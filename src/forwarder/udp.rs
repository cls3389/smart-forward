use anyhow::Result;
use async_trait::async_trait;
use log::{error, info};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use crate::utils::ConnectionStats;
use crate::stats;

pub struct UDPForwarder {
    listen_addr: String,
    name: String,
    buffer_size: usize,
    stats: Arc<RwLock<ConnectionStats>>,
    running: Arc<RwLock<bool>>,
    socket: Option<UdpSocket>,
    target_addr: Arc<RwLock<String>>,
}

impl UDPForwarder {
    pub fn new(listen_addr: &str, name: &str, buffer_size: usize) -> Self {
        Self {
            listen_addr: listen_addr.to_string(),
            name: name.to_string(),
            buffer_size,
            stats: Arc::new(RwLock::new(ConnectionStats::default())),
            running: Arc::new(RwLock::new(false)),
            socket: None,
            target_addr: Arc::new(RwLock::new(String::new())),
        }
    }
    
    pub async fn start_with_target(&mut self, target_addr: &str) -> Result<()> {
        *self.target_addr.write().await = target_addr.to_string();
        
        let socket = UdpSocket::bind(&self.listen_addr).await?;
        self.socket = Some(socket);
        *self.running.write().await = true;
        
        info!("UDP转发器启动: {} -> {}", self.listen_addr, target_addr);
        
        let stats = self.stats.clone();
        let running = self.running.clone();
        let buffer_size = self.buffer_size;
        let name = self.name.clone();
        let target_addr_arc = self.target_addr.clone();
        
        let socket = self.socket.take().ok_or_else(|| {
            anyhow::anyhow!("UDP socket未初始化")
        })?;
        
        tokio::spawn(async move {
            Self::udp_forward_loop(socket, buffer_size, name, stats, running, target_addr_arc).await;
        });
        
        Ok(())
    }
    
    async fn udp_forward_loop(
        socket: UdpSocket,
        buffer_size: usize,
        name: String,
        stats: Arc<RwLock<ConnectionStats>>,
        running: Arc<RwLock<bool>>,
        target_addr: Arc<RwLock<String>>,
    ) {
        let mut buffer = vec![0u8; buffer_size];
        
        // 添加客户端地址缓存，避免重复DNS解析
        let mut target_cache: std::collections::HashMap<String, (std::net::SocketAddr, std::time::Instant)> = std::collections::HashMap::new();
        
        loop {
            if !*running.read().await {
                break;
            }
            
            match socket.recv_from(&mut buffer).await {
                Ok((len, _client_addr)) => {
                    stats.write().await.add_bytes_received(len as u64);
                    
                    let target_addr_str = target_addr.read().await.clone();
                    
                    // 检查缓存中是否有目标地址且未过期（缓存10秒）
                    let target = if let Some((cached_target, timestamp)) = target_cache.get(&target_addr_str) {
                        if timestamp.elapsed().as_secs() < 10 {
                            *cached_target
                        } else {
                            // 缓存过期，重新解析
                            target_cache.remove(&target_addr_str);
                            
                            // 重新解析
                            match crate::utils::resolve_target(&target_addr_str).await {
                                Ok(addr) => {
                                    target_cache.insert(target_addr_str.clone(), (addr, std::time::Instant::now()));
                                    addr
                                },
                                Err(e) => {
                                    error!("UDP转发器 {} 解析目标地址失败 {}: {}", name, target_addr_str, e);
                                    continue;
                                }
                            }
                        }
                    } else {
                        // 缓存中没有，重新解析
                        match crate::utils::resolve_target(&target_addr_str).await {
                            Ok(addr) => {
                                target_cache.insert(target_addr_str.clone(), (addr, std::time::Instant::now()));
                                addr
                            },
                            Err(e) => {
                                error!("UDP转发器 {} 解析目标地址失败 {}: {}", name, target_addr_str, e);
                                continue;
                            }
                        }
                    };
                    
                    // 添加发送重试机制，减少重试次数
                    let data = &buffer[..len];
                    let mut send_success = false;
                    let max_retries = 1; // 减少重试次数到1次
                    let mut retry_count = 0;
                    
                    while retry_count < max_retries && !send_success {
                        match socket.send_to(data, target).await {
                            Ok(sent_len) => {
                                stats.write().await.add_bytes_sent(sent_len as u64);
                                send_success = true;
                                if retry_count > 0 {
                                    info!("UDP转发器 {} 重试发送成功，重试次数: {}", name, retry_count);
                                }
                            }
                            Err(e) => {
                                retry_count += 1;
                                if retry_count < max_retries {
                                    error!("UDP转发器 {} 发送数据失败 (第{}次重试): {}，将在50毫秒后重试", 
                                        name, retry_count, e);
                                    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                                } else {
                                    error!("UDP转发器 {} 发送数据失败 (已重试{}次): {}", 
                                        name, max_retries, e);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    if *running.read().await {
                        error!("UDP转发器 {} 接收数据失败: {}", name, e);
                        // 添加短暂延迟避免忙等待
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    } else {
                        // 正常停止
                        break;
                    }
                }
            }
        }
    }
    
    pub async fn update_target(&mut self, new_target: &str) -> Result<()> {
        let old_target = self.target_addr.read().await.clone();
        if old_target != new_target {
            info!("UDP转发器 {} 更新目标地址: {} -> {}", self.name, old_target, new_target);
            *self.target_addr.write().await = new_target.to_string();
        }
        Ok(())
    }
    
    pub async fn stop(&mut self) {
        info!("停止UDP转发器: {}", self.name);
        *self.running.write().await = false;
        self.socket.take();
    }
    
    pub fn get_stats(&self) -> HashMap<String, String> {
        let stats = self.stats.blocking_read();
        stats::get_stats_with_target(&stats, &self.target_addr.blocking_read())
    }
}

#[async_trait]
impl super::Forwarder for UDPForwarder {
    async fn start(&mut self) -> Result<()> {
        self.start_with_target("127.0.0.1:8080").await
    }
    
    async fn stop(&mut self) {
        self.stop().await;
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