use anyhow::Result;
use async_trait::async_trait;
use log::{error, info};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use crate::utils::ConnectionStats;
use crate::stats;

pub struct TCPForwarder {
    listen_addr: String,
    name: String,
    buffer_size: usize,
    stats: Arc<RwLock<ConnectionStats>>,
    running: Arc<RwLock<bool>>,
    listener: Option<TcpListener>,
    target_addr: Arc<RwLock<String>>,
}

impl TCPForwarder {
    pub fn new(listen_addr: &str, name: &str, buffer_size: usize) -> Self {
        Self {
            listen_addr: listen_addr.to_string(),
            name: name.to_string(),
            buffer_size,
            stats: Arc::new(RwLock::new(ConnectionStats::default())),
            running: Arc::new(RwLock::new(false)),
            listener: None,
            target_addr: Arc::new(RwLock::new(String::new())),
        }
    }
    
    pub async fn start_with_target(&mut self, target_addr: &str) -> Result<()> {
        *self.target_addr.write().await = target_addr.to_string();
        
        let listener = TcpListener::bind(&self.listen_addr).await?;
        self.listener = Some(listener);
        *self.running.write().await = true;
        
        let stats = self.stats.clone();
        let running = self.running.clone();
        let buffer_size = self.buffer_size;
        let name = self.name.clone();
        let target_addr_arc = self.target_addr.clone();
        
        let listener = self.listener.take().ok_or_else(|| {
            anyhow::anyhow!("TCP监听器未初始化")
        })?;
        
        tokio::spawn(async move {
            loop {
                if !*running.read().await {
                    break;
                }
                
                match listener.accept().await {
                    Ok((client_stream, _client_addr)) => {
                        stats.write().await.increment_connections();
                        
                        let stats_clone = stats.clone();
                        let name_clone = name.clone();
                        let target_addr_clone = target_addr_arc.clone();
                        
                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_connection(
                                client_stream, 
                                buffer_size, 
                                stats_clone,
                                &name_clone,
                                target_addr_clone,
                            ).await {
                                error!("规则 {} 处理连接失败: {}", name_clone, e);
                            }
                        });
                    }
                    Err(e) => {
                        if *running.read().await {
                            error!("接受连接失败: {}", e);
                        }
                        break;
                    }
                }
            }
        });
        
        Ok(())
    }
    
    pub async fn update_target(&mut self, new_target: &str) -> Result<()> {
        let old_target = self.target_addr.read().await.clone();
        if old_target != new_target {
            info!("规则 {} 更新目标地址: {} -> {}", self.name, old_target, new_target);
            *self.target_addr.write().await = new_target.to_string();
        }
        Ok(())
    }
    
    async fn handle_connection(
        mut client_stream: TcpStream,
        buffer_size: usize,
        stats: Arc<RwLock<ConnectionStats>>,
        rule_name: &str,
        target_addr: Arc<RwLock<String>>,
    ) -> Result<()> {
        let target_addr_str = target_addr.read().await.clone();
        let target = match crate::utils::resolve_target(&target_addr_str).await {
            Ok(addr) => addr,
            Err(e) => {
                error!("规则 {} 解析目标地址失败 {}: {}", rule_name, target_addr_str, e);
                return Err(anyhow::anyhow!("解析目标地址失败: {}", e));
            }
        };
        
        // 使用公共的带超时和重试的连接函数，减少重试次数
        let mut target_stream = match crate::utils::connect_with_timeout_and_retry(
            target, 
            1,  // max_retries - 减少到1次重试
            3,  // timeout_secs - 3秒超时（从5秒减少到3秒）
            1,  // retry_delay_secs - 1秒重试延迟
            &format!("规则 {}", rule_name)
        ).await {
            Ok(stream) => stream,
            Err(e) => {
                return Err(e);
            }
        };
        
        let (mut client_read, mut client_write) = client_stream.split();
        let (mut target_read, mut target_write) = target_stream.split();
        
        let mut client_buffer = vec![0u8; buffer_size];
        let mut target_buffer = vec![0u8; buffer_size];
        
        let (client_to_target, target_to_client) = tokio::join!(
            Self::forward_data(&mut client_read, &mut target_write, &mut client_buffer, &stats, true),
            Self::forward_data(&mut target_read, &mut client_write, &mut target_buffer, &stats, false),
        );
        
        if let Err(e) = client_to_target {
            error!("规则 {} 客户端到目标转发错误: {}", rule_name, e);
        }
        if let Err(e) = target_to_client {
            error!("规则 {} 目标到客户端转发错误: {}", rule_name, e);
        }
        
        Ok(())
    }
    
    async fn forward_data<R, W>(
        reader: &mut R,
        writer: &mut W,
        buffer: &mut [u8],
        stats: &Arc<RwLock<ConnectionStats>>,
        is_sent: bool,
    ) -> Result<()>
    where
        R: tokio::io::AsyncRead + Unpin,
        W: tokio::io::AsyncWrite + Unpin,
    {
        loop {
            let n = reader.read(buffer).await?;
            if n == 0 {
                break;
            }
            
            writer.write_all(&buffer[..n]).await?;
            writer.flush().await?;
            
            if is_sent {
                stats.write().await.add_bytes_sent(n as u64);
            } else {
                stats.write().await.add_bytes_received(n as u64);
            }
        }
        Ok(())
    }
    
    pub fn get_stats(&self) -> HashMap<String, String> {
        let stats = self.stats.blocking_read();
        stats::get_standard_stats(&stats)
    }
}

#[async_trait]
impl super::Forwarder for TCPForwarder {
    async fn start(&mut self) -> Result<()> {
        Err(anyhow::anyhow!("TCP转发器需要目标地址，请使用 start_with_target"))
    }
    
    async fn stop(&mut self) {
        *self.running.write().await = false;
        
        if let Some(listener) = self.listener.take() {
            drop(listener);
        }
    }
    
    fn is_running(&self) -> bool {
        *self.running.blocking_read()
    }
    
    fn get_stats(&self) -> HashMap<String, String> {
        self.get_stats()
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}