use anyhow::Result;
use async_trait::async_trait;
use log::{debug, error, info};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use crate::utils::ConnectionStats;

pub struct HTTPForwarder {
    listen_addr: String,
    name: String,

    stats: Arc<RwLock<ConnectionStats>>,
    running: Arc<RwLock<bool>>,
    listener: Option<TcpListener>,
}

impl HTTPForwarder {
    pub fn new(listen_addr: &str, name: &str, _buffer_size: usize) -> Self {
        Self {
            listen_addr: listen_addr.to_string(),
            name: name.to_string(),

            stats: Arc::new(RwLock::new(ConnectionStats::default())),
            running: Arc::new(RwLock::new(false)),
            listener: None,
        }
    }
    
    async fn handle_http_redirect(
        mut client_stream: TcpStream,
        stats: Arc<RwLock<ConnectionStats>>,
    ) -> Result<()> {
        // 读取HTTP请求
        let mut buffer = vec![0u8; 4096];
        let n = client_stream.read(&mut buffer).await?;
        
        if n == 0 {
            return Ok(());
        }
        
        let request = String::from_utf8_lossy(&buffer[..n]);
        let lines: Vec<&str> = request.lines().collect();
        
        if lines.is_empty() {
            return Ok(());
        }
        
        // 解析请求行
        let request_line = lines[0];
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        
        if parts.len() < 2 {
            return Ok(());
        }
        
        let method = parts[0];
        let path = parts[1];
        
        // 解析Host头
        let mut host = "localhost";
        for line in &lines[1..] {
            if line.to_lowercase().starts_with("host:") {
                host = line.split(':').nth(1).unwrap_or("localhost").trim();
                break;
            }
        }
        
        // 构建HTTPS重定向URL
        let redirect_url = if path == "/" {
            format!("https://{}", host)
        } else {
            format!("https://{}{}", host, path)
        };
        
        // 构建重定向响应
        let response = format!(
            "HTTP/1.1 301 Moved Permanently\r\n\
             Location: {}\r\n\
             Content-Type: text/html; charset=utf-8\r\n\
             Content-Length: {}\r\n\
             Connection: close\r\n\
             \r\n\
             <!DOCTYPE html>\r\n\
             <html>\r\n\
             <head><title>301 Moved Permanently</title></head>\r\n\
             <body>\r\n\
             <h1>Moved Permanently</h1>\r\n\
             <p>The document has moved <a href=\"{}\">here</a>.</p>\r\n\
             </body>\r\n\
             </html>",
            redirect_url,
            95 + redirect_url.len(), // Content-Length
            redirect_url
        );
        
        // 发送响应
        client_stream.write_all(response.as_bytes()).await?;
        client_stream.flush().await?;
        
        // 更新统计信息
        stats.write().await.add_bytes_sent(response.len() as u64);
        stats.write().await.add_bytes_received(n as u64);
        
        info!("HTTP跳转: {} {} -> {}", method, path, redirect_url);
        
        Ok(())
    }
}

#[async_trait]
impl super::Forwarder for HTTPForwarder {
    async fn start(&mut self) -> Result<()> {
        info!("启动HTTP转发器: {}", self.name);
        
        // 创建监听器
        let listener = TcpListener::bind(&self.listen_addr).await?;
        info!("HTTP监听器绑定到: {}", self.listen_addr);
        
        self.listener = Some(listener);
        *self.running.write().await = true;
        
        // 启动接受连接的循环
        let stats = self.stats.clone();
        let running = self.running.clone();
        let name = self.name.clone();
        let listener = self.listener.take().ok_or_else(|| {
            anyhow::anyhow!("HTTP监听器未初始化")
        })?;
        
        tokio::spawn(async move {
            loop {
                // 检查是否还在运行
                if !*running.read().await {
                    break;
                }
                
                match listener.accept().await {
                    Ok((client_stream, client_addr)) => {
                        debug!("接受HTTP客户端连接: {} -> {}", client_addr, name);
                        
                        // 增加连接计数
                        stats.write().await.increment_connections();
                        
                        // 为每个连接创建新的任务
                        let stats_clone = stats.clone();
                        
                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_http_redirect(client_stream, stats_clone).await {
                                error!("处理HTTP连接失败: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        if *running.read().await {
                            error!("接受HTTP连接失败: {}", e);
                        }
                        break;
                    }
                }
            }
        });
        
        info!("HTTP转发器启动成功: {}", self.name);
        Ok(())
    }
    
    async fn stop(&mut self) {
        info!("停止HTTP转发器: {}", self.name);
        *self.running.write().await = false;
        
        // 关闭监听器
        if let Some(listener) = self.listener.take() {
            drop(listener); // 这会关闭监听器
        }
        
        info!("HTTP转发器已停止: {}", self.name);
    }
    
    fn is_running(&self) -> bool {
        *self.running.blocking_read()
    }
    
    fn get_stats(&self) -> HashMap<String, String> {
        let stats = self.stats.blocking_read();
        let mut result = HashMap::new();
        
        result.insert("connections".to_string(), stats.connections.to_string());
        result.insert("bytes_sent".to_string(), stats.bytes_sent.to_string());
        result.insert("bytes_received".to_string(), stats.bytes_received.to_string());
        result.insert("uptime".to_string(), format!("{:?}", stats.get_uptime()));
        
        result
    }
    
    // 新增：实现as_any方法
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    // 新增：实现as_any_mut方法
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
