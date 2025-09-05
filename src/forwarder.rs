// 智能网络转发器 - 完整转发器实现
use crate::config::{Config, ForwardRule};
use crate::common::CommonManager;
use crate::utils::{ConnectionStats, get_standard_stats, get_stats_with_target};
use anyhow::Result;
use async_trait::async_trait;
use log::{error, info, warn};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::RwLock;

// ================================
// 转发器特征定义
// ================================
#[async_trait]
pub trait Forwarder: Send + Sync {
    async fn start(&mut self) -> Result<()>;
    async fn stop(&mut self);
    fn is_running(&self) -> bool;
    fn get_stats(&self) -> HashMap<String, String>;
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

// ================================
// TCP 转发器
// ================================
pub struct TCPForwarder {
    listen_addr: String,
    name: String,
    buffer_size: usize,
    target_addr: Arc<RwLock<String>>,
    stats: Arc<RwLock<ConnectionStats>>,
    running: Arc<RwLock<bool>>,
}

impl TCPForwarder {
    pub fn new(listen_addr: &str, name: &str, buffer_size: usize) -> Self {
        Self {
            listen_addr: listen_addr.to_string(),
            name: name.to_string(),
            buffer_size,
            target_addr: Arc::new(RwLock::new(String::new())),
            stats: Arc::new(RwLock::new(ConnectionStats::default())),
            running: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn start_with_target(&mut self, target: &str) -> Result<()> {
        *self.target_addr.write().await = target.to_string();
        *self.running.write().await = true;
        
        let listener = TcpListener::bind(&self.listen_addr).await?;
        let target_addr = self.target_addr.clone();
        let stats = self.stats.clone();
        let running = self.running.clone();
        let name = self.name.clone();
        let buffer_size = self.buffer_size;
        
        tokio::spawn(async move {
            while *running.read().await {
                match listener.accept().await {
                    Ok((stream, _)) => {
                        let target_str = target_addr.read().await.clone();
                        let stats = stats.clone();
                        let rule_name = name.clone();
                        
                        tokio::spawn(async move {
                            if let Err(_) = Self::handle_connection(stream, &target_str, buffer_size, stats, &rule_name).await {
                                // 连接处理失败，但不记录详细错误
                            }
                        });
                    }
                    Err(_) => break,
                }
            }
        });
        
        Ok(())
    }

    pub async fn update_target(&mut self, new_target: &str) -> Result<()> {
        *self.target_addr.write().await = new_target.to_string();
        Ok(())
    }

    async fn handle_connection(
        mut client_stream: TcpStream,
        target_addr: &str,
        buffer_size: usize,
        stats: Arc<RwLock<ConnectionStats>>,
        _rule_name: &str,
    ) -> Result<()> {
        let target: std::net::SocketAddr = crate::utils::resolve_target(target_addr).await?;
        
        stats.write().await.increment_connections();
        
        // 优化TCP：降低延迟
        let _ = client_stream.set_nodelay(true);

        // 直接连接，不重试（让健康检查快速切换到正确地址）
        let mut target_stream = match tokio::time::timeout(
            tokio::time::Duration::from_secs(5),
            tokio::net::TcpStream::connect(target)
        ).await {
            Ok(Ok(stream)) => stream,
            Ok(Err(e)) => return Err(anyhow::anyhow!("连接目标失败: {}", e)),
            Err(_) => return Err(anyhow::anyhow!("连接目标超时")),
        };

        // 目标侧同样禁用Nagle算法
        let _ = target_stream.set_nodelay(true);
        
        let (mut client_read, mut client_write) = client_stream.split();
        let (mut target_read, mut target_write) = target_stream.split();
        
        let mut client_buffer = vec![0u8; buffer_size];
        let mut target_buffer = vec![0u8; buffer_size];
        
        let (client_to_target, target_to_client) = tokio::join!(
            Self::forward_data(&mut client_read, &mut target_write, &mut client_buffer, &stats, true),
            Self::forward_data(&mut target_read, &mut client_write, &mut target_buffer, &stats, false),
        );
        
        // 简化错误处理，连接断开是正常现象，减少日志噪音
        if let Err(_) = client_to_target {
            // 连接断开不记录错误日志
        }
        if let Err(_) = target_to_client {
            // 连接断开不记录错误日志
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
        let mut total_bytes = 0u64;
        loop {
            let n = reader.read(buffer).await?;
            if n == 0 {
                break;
            }
            
            writer.write_all(&buffer[..n]).await?;
            total_bytes += n as u64;
        }
        
        // 批量更新统计信息，减少锁竞争
        if total_bytes > 0 {
            if is_sent {
                stats.write().await.add_bytes_sent(total_bytes);
            } else {
                stats.write().await.add_bytes_received(total_bytes);
            }
        }
        
        Ok(())
    }
    
    pub fn get_stats(&self) -> HashMap<String, String> {
        let stats = self.stats.blocking_read();
        get_standard_stats(&stats)
    }
}

#[async_trait]
impl Forwarder for TCPForwarder {
    async fn start(&mut self) -> Result<()> {
        Err(anyhow::anyhow!("TCP转发器需要使用start_with_target方法"))
    }
    
    async fn stop(&mut self) {
        *self.running.write().await = false;
    }
    
    fn is_running(&self) -> bool {
        *self.running.blocking_read()
    }
    
    fn get_stats(&self) -> HashMap<String, String> {
        Self::get_stats(self)
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

// ================================
// HTTP 转发器
// ================================
pub struct HTTPForwarder {
    listen_addr: String,
    name: String,
    running: Arc<RwLock<bool>>,
}

impl HTTPForwarder {
    pub fn new(listen_addr: &str, name: &str, _buffer_size: usize) -> Self {
        Self {
            listen_addr: listen_addr.to_string(),
            name: name.to_string(),
            running: Arc::new(RwLock::new(false)),
        }
    }

    async fn handle_http_redirect(mut stream: TcpStream) -> Result<()> {
        let mut buffer = [0; 1024];
        let n = stream.read(&mut buffer).await?;
        
        if n > 0 {
            let request = String::from_utf8_lossy(&buffer[..n]);
            if let Some(host_line) = request.lines().find(|line| line.to_lowercase().starts_with("host:")) {
                let host = host_line.split(':').nth(1).unwrap_or("").trim();
                let https_url = format!("https://{}", host);
                
                let response = format!(
                    "HTTP/1.1 301 Moved Permanently\r\n\
                     Location: {}\r\n\
                     Connection: close\r\n\
                     Content-Length: 0\r\n\
                     \r\n",
                    https_url
                );
                
                stream.write_all(response.as_bytes()).await?;
            }
        }
        
        Ok(())
    }
}

#[async_trait]
impl Forwarder for HTTPForwarder {
    async fn start(&mut self) -> Result<()> {
        *self.running.write().await = true;
        
        let listener = TcpListener::bind(&self.listen_addr).await?;
        let running = self.running.clone();
        
        tokio::spawn(async move {
            while *running.read().await {
                match listener.accept().await {
                    Ok((stream, _)) => {
                        tokio::spawn(async move {
                            let _ = Self::handle_http_redirect(stream).await;
                        });
                    }
                    Err(_) => break,
                }
            }
        });
        
        Ok(())
    }
    
    async fn stop(&mut self) {
        *self.running.write().await = false;
    }
    
    fn is_running(&self) -> bool {
        *self.running.blocking_read()
    }
    
    fn get_stats(&self) -> HashMap<String, String> {
        let mut stats = HashMap::new();
        stats.insert("name".to_string(), self.name.clone());
        stats.insert("type".to_string(), "HTTP Redirect".to_string());
        stats.insert("running".to_string(), self.is_running().to_string());
        stats
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

// ================================
// UDP 转发器
// ================================
pub struct UDPForwarder {
    listen_addr: String,
    buffer_size: usize,
    target_addr: Arc<RwLock<String>>,
    stats: Arc<RwLock<ConnectionStats>>,
    running: Arc<RwLock<bool>>,
}

impl UDPForwarder {
    pub fn new(listen_addr: &str, _name: &str, buffer_size: usize) -> Self {
        Self {
            listen_addr: listen_addr.to_string(),
            buffer_size,
            target_addr: Arc::new(RwLock::new(String::new())),
            stats: Arc::new(RwLock::new(ConnectionStats::default())),
            running: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn start_with_target(&mut self, target: &str) -> Result<()> {
        *self.target_addr.write().await = target.to_string();
        *self.running.write().await = true;
        
        let socket = UdpSocket::bind(&self.listen_addr).await?;
        let target_addr = self.target_addr.clone();
        let stats = self.stats.clone();
        let running = self.running.clone();
        let buffer_size = self.buffer_size;
        
        tokio::spawn(async move {
            let mut buffer = vec![0u8; buffer_size];
            
            while *running.read().await {
                match socket.recv_from(&mut buffer).await {
                    Ok((size, _client_addr)) => {
                        let target_str = target_addr.read().await.clone();
                        let stats = stats.clone();
                        
                        // 简化的UDP转发：直接转发，不维护会话
                        if let Ok(target) = crate::utils::resolve_target(&target_str).await {
                            if let Ok(upstream_socket) = UdpSocket::bind("0.0.0.0:0").await {
                                let _ = upstream_socket.send_to(&buffer[..size], target).await;
                                stats.write().await.add_bytes_sent(size as u64);
                                stats.write().await.increment_connections();
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        });
        
        Ok(())
    }

    pub async fn update_target(&mut self, new_target: &str) -> Result<()> {
        *self.target_addr.write().await = new_target.to_string();
        Ok(())
    }
    
    pub fn get_stats(&self) -> HashMap<String, String> {
        let stats = self.stats.blocking_read();
        get_stats_with_target(&stats, &self.target_addr.blocking_read())
    }
}

#[async_trait]
impl Forwarder for UDPForwarder {
    async fn start(&mut self) -> Result<()> {
        Err(anyhow::anyhow!("UDP转发器需要使用start_with_target方法"))
    }
    
    async fn stop(&mut self) {
        *self.running.write().await = false;
    }
    
    fn is_running(&self) -> bool {
        *self.running.blocking_read()
    }
    
    fn get_stats(&self) -> HashMap<String, String> {
        Self::get_stats(self)
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

// ================================
// 统一转发器
// ================================
pub struct UnifiedForwarder {
    rule: ForwardRule,
    listen_addr: String,
    target_addr: String,
    tcp_forwarder: Option<TCPForwarder>,
    http_forwarder: Option<HTTPForwarder>,
    udp_forwarder: Option<UDPForwarder>,
    running: Arc<RwLock<bool>>,
    last_update: Arc<RwLock<Instant>>,
}

impl UnifiedForwarder {
    pub fn new_with_target(rule: &ForwardRule, listen_addr: &str, target_addr: &str) -> Self {
        Self {
            rule: rule.clone(),
            listen_addr: listen_addr.to_string(),
            target_addr: target_addr.to_string(),
            tcp_forwarder: None,
            http_forwarder: None,
            udp_forwarder: None,
            running: Arc::new(RwLock::new(false)),
            last_update: Arc::new(RwLock::new(Instant::now())),
        }
    }

    pub async fn update_target(&mut self, new_target: &str) -> Result<()> {
        if self.target_addr != new_target {
            self.target_addr = new_target.to_string();
            *self.last_update.write().await = Instant::now();
            
            // 更新各转发器的目标地址
            if let Some(ref mut tcp) = self.tcp_forwarder {
                tcp.update_target(new_target).await?;
            }
            if let Some(ref mut udp) = self.udp_forwarder {
                udp.update_target(new_target).await?;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl Forwarder for UnifiedForwarder {
    async fn start(&mut self) -> Result<()> {
        *self.running.write().await = true;
        
        let protocols = if let Some(ref protocols) = self.rule.protocols {
            protocols.clone()
        } else if let Some(ref protocol) = self.rule.protocol {
            vec![protocol.clone()]
        } else {
            vec!["tcp".to_string()]
        };
        
        for protocol in &protocols {
            match protocol.as_str() {
                "tcp" => {
                    if self.tcp_forwarder.is_none() {
                        let mut tcp_forwarder = TCPForwarder::new(
                            &self.listen_addr,
                            &format!("{}_TCP", self.rule.name),
                            self.rule.get_effective_buffer_size(8192),
                        );
                        tcp_forwarder.start_with_target(&self.target_addr).await?;
                        self.tcp_forwarder = Some(tcp_forwarder);
                    }
                }
                "udp" => {
                    if self.udp_forwarder.is_none() {
                        let mut udp_forwarder = UDPForwarder::new(
                            &self.listen_addr,
                            &format!("{}_UDP", self.rule.name),
                            self.rule.get_effective_buffer_size(8192),
                        );
                        udp_forwarder.start_with_target(&self.target_addr).await?;
                        self.udp_forwarder = Some(udp_forwarder);
                    }
                }
                "http" => {
                    if self.http_forwarder.is_none() {
                        let mut http_forwarder = HTTPForwarder::new(
                            &self.listen_addr,
                            &format!("{}_HTTP", self.rule.name),
                            self.rule.get_effective_buffer_size(8192),
                        );
                        http_forwarder.start().await?;
                        self.http_forwarder = Some(http_forwarder);
                    }
                }
                _ => {}
            }
        }
        
        Ok(())
    }
    
    async fn stop(&mut self) {
        *self.running.write().await = false;
        
        if let Some(ref mut tcp) = self.tcp_forwarder {
            tcp.stop().await;
        }
        if let Some(ref mut udp) = self.udp_forwarder {
            udp.stop().await;
        }
        if let Some(ref mut http) = self.http_forwarder {
            http.stop().await;
        }
    }
    
    fn is_running(&self) -> bool {
        *self.running.blocking_read()
    }
    
    fn get_stats(&self) -> HashMap<String, String> {
        let mut stats = HashMap::new();
        stats.insert("rule_name".to_string(), self.rule.name.clone());
        stats.insert("target_addr".to_string(), self.target_addr.clone());
        let protocols_str = if let Some(ref protocols) = self.rule.protocols {
            protocols.join("+")
        } else if let Some(ref protocol) = self.rule.protocol {
            protocol.clone()
        } else {
            "tcp".to_string()
        };
        stats.insert("protocols".to_string(), protocols_str);
        stats.insert("running".to_string(), self.is_running().to_string());
        
        if let Some(ref tcp) = self.tcp_forwarder {
            let tcp_stats = tcp.get_stats();
            for (k, v) in tcp_stats {
                stats.insert(format!("tcp_{}", k), v);
            }
        }
        
        if let Some(ref udp) = self.udp_forwarder {
            let udp_stats = udp.get_stats();
            for (k, v) in udp_stats {
                stats.insert(format!("udp_{}", k), v);
            }
        }
        
        stats
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

// ================================
// 智能转发器管理器
// ================================
pub struct SmartForwarder {
    config: Config,
    common_manager: CommonManager,
    forwarders: Arc<RwLock<HashMap<String, Box<dyn Forwarder + Send + Sync>>>>,
    dynamic_update_started: Arc<RwLock<bool>>,
}

impl SmartForwarder {
    pub fn new(config: Config, common_manager: CommonManager) -> Self {
        Self {
            config,
            common_manager,
            forwarders: Arc::new(RwLock::new(HashMap::new())),
            dynamic_update_started: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        // 初始化公共管理器
        self.common_manager.initialize().await?;
        
        // 简化的初始化信息
        info!("智能转发器初始化完成，开始启动转发规则...");
        
        Ok(())
    }

    pub async fn start(&mut self) -> Result<()> {
        let rules = self.config.rules.clone();
        for rule in &rules {
            self.start_forwarder(rule).await?;
        }
        
        // 启动动态更新任务
        if !*self.dynamic_update_started.read().await {
            self.start_dynamic_update_task().await;
            *self.dynamic_update_started.write().await = true;
        }
        
        Ok(())
    }

    async fn start_forwarder(&mut self, rule: &ForwardRule) -> Result<()> {
        let listen_addr = rule.get_listen_addr(&self.config.network.listen_addr);
        
        // 获取最佳目标
        if let Ok(best_target) = self.common_manager.get_best_target(&rule.name).await {
            let target_addr = best_target.to_string();
            
            info!("规则 {} 启动: {} -> {}", rule.name, listen_addr, target_addr);
            
            // 创建统一转发器
            let mut unified_forwarder = UnifiedForwarder::new_with_target(rule, &listen_addr, &target_addr);
            unified_forwarder.start().await?;
            
            self.forwarders.write().await.insert(
                rule.name.clone(),
                Box::new(unified_forwarder),
            );
        } else {
            warn!("规则 {} 没有可用的目标地址", rule.name);
        }
        
        Ok(())
    }

    async fn start_dynamic_update_task(&self) {
        let forwarders = self.forwarders.clone();
        let common_manager = self.common_manager.clone();
        let rules = self.config.rules.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(15));
            
            loop {
                interval.tick().await;
                
                for rule in &rules {
                    if let Ok(best_target) = common_manager.get_best_target(&rule.name).await {
                        let target_addr = best_target.to_string();
                        
                        let mut forwarders_guard = forwarders.write().await;
                        if let Some(forwarder) = forwarders_guard.get_mut(&rule.name) {
                            if let Some(unified) = forwarder.as_any_mut().downcast_mut::<UnifiedForwarder>() {
                                if let Err(e) = unified.update_target(&target_addr).await {
                                    error!("规则 {} 更新目标失败: {}", rule.name, e);
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    pub async fn stop(&mut self) {
        let mut forwarders = self.forwarders.write().await;
        for (name, forwarder) in forwarders.iter_mut() {
            info!("停止转发器: {}", name);
            forwarder.stop().await;
        }
        forwarders.clear();
    }

    pub async fn get_stats(&self) -> HashMap<String, HashMap<String, String>> {
        let mut all_stats = HashMap::new();
        let forwarders = self.forwarders.read().await;
        
        for (name, forwarder) in forwarders.iter() {
            all_stats.insert(name.clone(), forwarder.get_stats());
        }
        
        all_stats
    }
}
