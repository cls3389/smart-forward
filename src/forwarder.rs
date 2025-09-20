// 智能网络转发器 - 完整转发器实现
use crate::common::CommonManager;
use crate::config::{Config, ForwardRule};
use crate::firewall::FirewallScheduler;
use crate::utils::{get_standard_stats, get_stats_with_target, ConnectionStats};
use anyhow::Result;
use async_trait::async_trait;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
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
    #[allow(dead_code)]
    fn get_stats(&self) -> HashMap<String, String>;
    #[allow(dead_code)]
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

        let listener = match TcpListener::bind(&self.listen_addr).await {
            Ok(listener) => {
                log::info!("TCP监听器 {} 绑定成功: {}", self.name, self.listen_addr);
                listener
            }
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "TCP监听器 {} 绑定失败 {}: {}",
                    self.name,
                    self.listen_addr,
                    e
                ));
            }
        };
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
                            if (Self::handle_connection(
                                stream,
                                &target_str,
                                buffer_size,
                                stats,
                                &rule_name,
                            )
                            .await)
                                .is_err()
                            {
                                // 连接处理失败，但不记录详细错误
                            }
                        });
                    }
                    Err(e) => {
                        // 监听错误，记录日志但继续运行
                        log::warn!("TCP监听器 {name} 接受连接失败: {e}");
                        // 短暂延迟后继续，避免快速重试
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        continue;
                    }
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
            tokio::net::TcpStream::connect(target),
        )
        .await
        {
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
            Self::forward_data(
                &mut client_read,
                &mut target_write,
                &mut client_buffer,
                &stats,
                true
            ),
            Self::forward_data(
                &mut target_read,
                &mut client_write,
                &mut target_buffer,
                &stats,
                false
            ),
        );

        // 简化错误处理，连接断开是正常现象，减少日志噪音
        if client_to_target.is_err() {
            // 连接断开不记录错误日志
        }
        if target_to_client.is_err() {
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
        let mut buffer = vec![0u8; 4096];
        let n = stream.read(&mut buffer).await?;

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

        // 构建HTTPS重定向URL，保持完整路径和参数
        let redirect_url = if path == "/" {
            format!("https://{host}")
        } else {
            format!("https://{host}{path}")
        };

        // 构建响应
        let response = format!(
            "HTTP/1.1 301 Moved Permanently\r\n\
             Location: {redirect_url}\r\n\
             Connection: close\r\n\
             Content-Length: 0\r\n\
             \r\n"
        );

        stream.write_all(response.as_bytes()).await?;
        info!("HTTP跳转: {method} {path} -> {redirect_url}");

        Ok(())
    }
}

#[async_trait]
impl Forwarder for HTTPForwarder {
    async fn start(&mut self) -> Result<()> {
        *self.running.write().await = true;

        let listener = match TcpListener::bind(&self.listen_addr).await {
            Ok(listener) => {
                info!("HTTP监听器绑定到: {}", self.listen_addr);
                listener
            }
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "HTTP监听器绑定失败 {}: {}",
                    self.listen_addr,
                    e
                ));
            }
        };
        let running = self.running.clone();
        let name = self.name.clone();

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

        info!("HTTP转发器启动成功: {name}");
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
// UDP 转发器 - 基于原版优化实现
// ================================
pub struct UDPForwarder {
    listen_addr: String,
    name: String,
    buffer_size: usize,
    target_addr: Arc<RwLock<String>>,
    stats: Arc<RwLock<ConnectionStats>>,
    running: Arc<RwLock<bool>>,
    sessions: Arc<RwLock<HashMap<std::net::SocketAddr, UdpSession>>>,
}

// UDP会话结构
struct UdpSession {
    upstream: Option<Arc<UdpSocket>>,
    target: std::net::SocketAddr,
    last_seen: std::time::Instant,
}

impl UdpSession {
    fn new() -> Self {
        Self {
            upstream: None,
            target: "0.0.0.0:0".parse().unwrap(),
            last_seen: std::time::Instant::now(),
        }
    }
}

impl UDPForwarder {
    pub fn new(listen_addr: &str, name: &str, buffer_size: usize) -> Self {
        Self {
            listen_addr: listen_addr.to_string(),
            name: name.to_string(),
            buffer_size,
            target_addr: Arc::new(RwLock::new(String::new())),
            stats: Arc::new(RwLock::new(ConnectionStats::default())),
            running: Arc::new(RwLock::new(false)),
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start_with_target(&mut self, target: &str) -> Result<()> {
        *self.target_addr.write().await = target.to_string();
        *self.running.write().await = true;

        let socket = match UdpSocket::bind(&self.listen_addr).await {
            Ok(socket) => {
                log::info!("UDP监听器绑定成功: {}", self.listen_addr);
                socket
            }
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "UDP监听器绑定失败 {}: {}",
                    self.listen_addr,
                    e
                ));
            }
        };

        // 启动主转发循环
        let stats = self.stats.clone();
        let running = self.running.clone();
        let target_addr = self.target_addr.clone();
        let sessions = self.sessions.clone();
        let buffer_size = self.buffer_size;
        let name = self.name.clone();

        tokio::spawn(async move {
            Self::udp_forward_loop(
                socket,
                buffer_size,
                name,
                stats,
                running,
                target_addr,
                sessions,
            )
            .await;
        });

        // 启动会话清理任务
        let sessions_cleanup = self.sessions.clone();
        let running_cleanup = self.running.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                if !*running_cleanup.read().await {
                    break;
                }
                interval.tick().await;

                let now = std::time::Instant::now();
                let mut to_remove = Vec::new();
                {
                    let sessions_read = sessions_cleanup.read().await;
                    for (client, sess) in sessions_read.iter() {
                        if now.duration_since(sess.last_seen).as_secs() > 60 {
                            to_remove.push(*client);
                        }
                    }
                }
                if !to_remove.is_empty() {
                    let mut sessions_write = sessions_cleanup.write().await;
                    for client in to_remove {
                        sessions_write.remove(&client);
                    }
                }
            }
        });

        Ok(())
    }

    async fn udp_forward_loop(
        socket: UdpSocket,
        buffer_size: usize,
        _name: String,
        stats: Arc<RwLock<ConnectionStats>>,
        running: Arc<RwLock<bool>>,
        target_addr: Arc<RwLock<String>>,
        sessions: Arc<RwLock<HashMap<std::net::SocketAddr, UdpSession>>>,
    ) {
        let mut buffer = vec![0u8; buffer_size];
        let socket = Arc::new(socket);
        let mut target_cache: HashMap<String, (std::net::SocketAddr, std::time::Instant)> =
            HashMap::new();

        loop {
            if !*running.read().await {
                break;
            }

            match socket.recv_from(&mut buffer).await {
                Ok((len, client_addr)) => {
                    stats.write().await.add_bytes_received(len as u64);

                    let target_addr_str = target_addr.read().await.clone();

                    // DNS缓存：5分钟有效期
                    let target = if let Some((cached_target, timestamp)) =
                        target_cache.get(&target_addr_str)
                    {
                        if timestamp.elapsed().as_secs() < 300 {
                            *cached_target
                        } else {
                            target_cache.remove(&target_addr_str);
                            match crate::utils::resolve_target(&target_addr_str).await {
                                Ok(addr) => {
                                    target_cache.insert(
                                        target_addr_str.clone(),
                                        (addr, std::time::Instant::now()),
                                    );
                                    addr
                                }
                                Err(_) => continue,
                            }
                        }
                    } else {
                        match crate::utils::resolve_target(&target_addr_str).await {
                            Ok(addr) => {
                                target_cache.insert(
                                    target_addr_str.clone(),
                                    (addr, std::time::Instant::now()),
                                );
                                addr
                            }
                            Err(_) => continue,
                        }
                    };

                    // 获取或创建会话
                    let mut sessions_guard = sessions.write().await;
                    let entry = sessions_guard
                        .entry(client_addr)
                        .or_insert_with(UdpSession::new);

                    // 如果没有上游socket或目标变化，重新连接
                    if entry.upstream.is_none() || entry.target != target {
                        if let Ok(upstream) = UdpSocket::bind("0.0.0.0:0").await {
                            if upstream.connect(target).await.is_ok() {
                                let upstream = Arc::new(upstream);

                                // 启动回程任务
                                let upstream_reader = upstream.clone();
                                let socket_clone = socket.clone();
                                let stats_clone = stats.clone();
                                tokio::spawn(async move {
                                    let mut resp_buf = vec![0u8; 4096];
                                    while let Ok(resp_len) =
                                        upstream_reader.recv(&mut resp_buf).await
                                    {
                                        if resp_len > 0 {
                                            let _ = socket_clone
                                                .send_to(&resp_buf[..resp_len], client_addr)
                                                .await;
                                            stats_clone
                                                .write()
                                                .await
                                                .add_bytes_sent(resp_len as u64);
                                        }
                                    }
                                });

                                entry.upstream = Some(upstream);
                                entry.target = target;
                            }
                        }
                    }
                    entry.last_seen = std::time::Instant::now();

                    // 转发数据
                    if let Some(ref upstream) = entry.upstream {
                        let _ = upstream.send(&buffer[..len]).await;
                        stats.write().await.add_bytes_sent(len as u64);
                    }
                }
                Err(_) => {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
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

        // 使用规则的 get_protocols() 方法获取协议列表
        let protocols = self.rule.get_protocols();

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
                stats.insert(format!("tcp_{k}"), v);
            }
        }

        if let Some(ref udp) = self.udp_forwarder {
            let udp_stats = udp.get_stats();
            for (k, v) in udp_stats {
                stats.insert(format!("udp_{k}"), v);
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
    firewall_scheduler: Option<FirewallScheduler>,
}

impl SmartForwarder {
    pub fn new(
        config: Config,
        common_manager: CommonManager,
        firewall_scheduler: Option<FirewallScheduler>,
    ) -> Self {
        Self {
            config,
            common_manager,
            forwarders: Arc::new(RwLock::new(HashMap::new())),
            dynamic_update_started: Arc::new(RwLock::new(false)),
            firewall_scheduler,
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
        let mut success_count = 0;
        let is_kernel_mode = self.firewall_scheduler.is_some();

        // 检查是否需要自动启用HTTP跳转服务
        let has_443 = rules.iter().any(|r| r.listen_port == 443);
        let has_80 = rules.iter().any(|r| r.listen_port == 80);

        // 如果配置了443但没有配置80，自动启用HTTP跳转
        if has_443 && !has_80 {
            if let Err(e) = self.start_auto_http_redirect().await {
                warn!("自动HTTP跳转服务启动失败: {e}");
            } else {
                success_count += 1;
            }
        }

        // 根据转发模式决定是否启动用户态转发器
        if is_kernel_mode {
            info!("🚀 内核态转发模式：跳过用户态转发器启动，使用内核DNAT/SNAT");
            success_count = rules.len(); // 内核态转发由FirewallScheduler处理
        } else {
            info!("📡 用户态转发模式：启动应用层转发器");
            for rule in &rules {
                match self.start_forwarder(rule).await {
                    Ok(_) => {
                        success_count += 1;
                    }
                    Err(e) => {
                        error!("规则 {} 启动失败: {}", rule.name, e);
                        // 继续处理其他规则，不退出
                    }
                }
            }
        }

        // 计算实际启动的规则数量，区分配置规则和自动服务
        let configured_rules_started = success_count - if has_443 && !has_80 { 1 } else { 0 };

        if has_443 && !has_80 && success_count > configured_rules_started {
            info!(
                "启动完成: {} 个规则可用 (配置 {} 个规则 + 自动HTTP跳转服务)",
                success_count, configured_rules_started
            );
        } else {
            info!("启动完成: {} 个规则可用", success_count);
        }

        // 启动动态更新任务
        if !*self.dynamic_update_started.read().await {
            self.start_dynamic_update_task().await;
            *self.dynamic_update_started.write().await = true;
        }

        // 如果没有任何规则启动成功，返回错误
        if success_count == 0 {
            return Err(anyhow::anyhow!(
                "没有规则成功启动，请检查配置和端口占用情况"
            ));
        }

        Ok(())
    }

    async fn start_auto_http_redirect(&mut self) -> Result<()> {
        let listen_addr = format!("{}:80", self.config.network.listen_addr);

        // 检查80端口是否被占用
        if tokio::net::TcpListener::bind(&listen_addr).await.is_err() {
            warn!("端口80被占用，无法启动自动HTTP跳转服务");
            return Ok(()); // 不返回错误，只是跳过
        }

        info!("检测到HTTPS配置但无HTTP配置，自动启用HTTP跳转服务");

        let mut http_forwarder = HTTPForwarder::new(&listen_addr, "AutoHTTP", 4096);
        http_forwarder.start().await?;

        // 将HTTP转发器添加到管理列表中
        self.forwarders
            .write()
            .await
            .insert("AutoHTTP".to_string(), Box::new(http_forwarder));

        info!("自动HTTP跳转服务启动成功");
        Ok(())
    }

    async fn start_forwarder(&mut self, rule: &ForwardRule) -> Result<()> {
        let listen_addr = rule.get_listen_addr(&self.config.network.listen_addr);

        // 获取最佳目标
        if let Ok(best_target) = self.common_manager.get_best_target(&rule.name).await {
            let target_addr = best_target.to_string();

            info!(
                "规则 {} 启动: {} -> {}",
                rule.name, listen_addr, target_addr
            );

            // 创建统一转发器
            let mut unified_forwarder =
                UnifiedForwarder::new_with_target(rule, &listen_addr, &target_addr);
            match unified_forwarder.start().await {
                Ok(_) => {
                    self.forwarders
                        .write()
                        .await
                        .insert(rule.name.clone(), Box::new(unified_forwarder));
                }
                Err(e) => {
                    error!("规则 {} 启动失败: {}", rule.name, e);
                    // 不返回错误，继续处理其他规则
                }
            }
        } else {
            warn!("规则 {} 没有可用的目标地址", rule.name);
        }

        Ok(())
    }

    async fn start_dynamic_update_task(&self) {
        let forwarders = self.forwarders.clone();
        let common_manager = self.common_manager.clone();
        let rules = self.config.rules.clone();
        let has_firewall_scheduler = self.firewall_scheduler.is_some();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(15));

            loop {
                interval.tick().await;

                // 如果启用了内核态转发，优先同步防火墙规则
                if has_firewall_scheduler {
                    // 注意：这里我们无法直接访问firewall_scheduler，因为它被移动到了SmartForwarder中
                    // 在实际实现中，我们需要重新设计这部分架构
                    debug!("内核态模式：防火墙规则同步由FirewallScheduler处理");
                }

                // 更新用户态转发器（如果存在）
                for rule in &rules {
                    if let Ok(best_target) = common_manager.get_best_target(&rule.name).await {
                        let target_addr = best_target.to_string();

                        let mut forwarders_guard = forwarders.write().await;
                        if let Some(forwarder) = forwarders_guard.get_mut(&rule.name) {
                            if let Some(unified) =
                                forwarder.as_any_mut().downcast_mut::<UnifiedForwarder>()
                            {
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
            info!("停止转发器: {name}");
            forwarder.stop().await;
        }
        forwarders.clear();
    }

    #[allow(dead_code)]
    pub async fn get_stats(&self) -> HashMap<String, HashMap<String, String>> {
        let mut all_stats = HashMap::new();
        let forwarders = self.forwarders.read().await;

        for (name, forwarder) in forwarders.iter() {
            all_stats.insert(name.clone(), forwarder.get_stats());
        }

        all_stats
    }
}
