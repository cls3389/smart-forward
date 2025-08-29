use anyhow::Result;
use async_trait::async_trait;
use log::{error, info};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use crate::utils::ConnectionStats;
use crate::stats;
use std::collections::HashMap as StdHashMap;

pub struct UDPForwarder {
    listen_addr: String,
    name: String,
    buffer_size: usize,
    stats: Arc<RwLock<ConnectionStats>>,
    running: Arc<RwLock<bool>>,
    socket: Option<UdpSocket>,
    target_addr: Arc<RwLock<String>>,
    // 新增：会话映射（客户端 -> 上游会话）
    sessions: Arc<RwLock<StdHashMap<std::net::SocketAddr, UdpSession>>>,
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
            sessions: Arc::new(RwLock::new(StdHashMap::new())),
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
        let sessions_arc = self.sessions.clone();
        
        let socket = self.socket.take().ok_or_else(|| {
            anyhow::anyhow!("UDP socket未初始化")
        })?;
        
        tokio::spawn(async move {
            Self::udp_forward_loop(socket, buffer_size, name, stats, running.clone(), target_addr_arc, sessions_arc.clone()).await;
        });

        // 会话清理任务：移除 60 秒无活动的会话
        let sessions_cleanup = self.sessions.clone();
        let running_cleanup = self.running.clone();
        tokio::spawn(async move {
            const IDLE_SECS: u64 = 60;
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            loop {
                if !*running_cleanup.read().await { break; }
                interval.tick().await;
                let now = std::time::Instant::now();
                let mut to_remove = Vec::new();
                {
                    let sessions_read = sessions_cleanup.read().await;
                    for (client, sess) in sessions_read.iter() {
                        if now.duration_since(sess.last_seen).as_secs() > IDLE_SECS {
                            to_remove.push(*client);
                        }
                    }
                }
                if !to_remove.is_empty() {
                    let mut sessions_write = sessions_cleanup.write().await;
                    for client in to_remove {
                        sessions_write.remove(&client);
                    }
                    info!("UDP会话清理: 已执行清理，当前会话数 {}", sessions_write.len());
                }
            }
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
        sessions: Arc<RwLock<StdHashMap<std::net::SocketAddr, UdpSession>>>,
    ) {
        let mut buffer = vec![0u8; buffer_size];
        // 允许在回程任务中共享监听socket
        let socket = Arc::new(socket);
        
        // 添加客户端地址缓存，避免重复DNS解析
        let mut target_cache: std::collections::HashMap<String, (std::net::SocketAddr, std::time::Instant)> = std::collections::HashMap::new();
        
        loop {
            if !*running.read().await {
                break;
            }
            
            match socket.recv_from(&mut buffer).await {
                Ok((len, client_addr)) => {
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
                    
                    // 获取或创建客户端会话（为每个客户端创建单独的上游socket）
                    let mut sessions_guard = sessions.write().await;
                    let entry = sessions_guard.entry(client_addr).or_insert_with(|| {
                        info!("UDP会话创建: 客户端 {}", client_addr);
                        UdpSession::new(client_addr)
                    });
                    // 如果没有上游socket或目标变化，则重新连接
                    if entry.upstream.is_none() || entry.target != target {
                        match UdpSocket::bind("0.0.0.0:0").await {
                            Ok(up) => {
                                if let Err(e) = up.connect(target).await {
                                    error!("UDP转发器 {} 连接上游失败 {}: {}", name, target, e);
                                    continue;
                                }
                                let up = Arc::new(up);
                                // 启动回程任务：独立协程非阻塞读取并回发客户端
                                let up_reader = up.clone();
                                let client_addr_clone = client_addr;
                                let socket_clone = socket.clone();
                                let name_clone = name.clone();
                                let stats_clone = stats.clone();
                                tokio::spawn(async move {
                                    let mut resp_buf = vec![0u8; 4096];
                                    loop {
                                        match up_reader.recv(&mut resp_buf).await {
                                            Ok(resp_len) => {
                                                if resp_len == 0 { continue; }
                                                if let Err(e) = socket_clone.send_to(&resp_buf[..resp_len], client_addr_clone).await {
                                                    log::debug!("UDP转发器 {} 回程发送到客户端失败: {}", name_clone, e);
                                                } else {
                                                    stats_clone.write().await.add_bytes_sent(resp_len as u64);
                                                }
                                            }
                                            Err(e) => {
                                                log::debug!("UDP转发器 {} 回程接收失败: {}", name_clone, e);
                                                break;
                                            }
                                        }
                                    }
                                });
                                entry.upstream = Some(up);
                                entry.target = target;
                                entry.last_seen = std::time::Instant::now();
                                // 回程在发送后以短超时同步尝试一次读取，由主循环处理，避免克隆socket
                            }
                            Err(e) => {
                                error!("UDP转发器 {} 分配上游socket失败: {}", name, e);
                                continue;
                            }
                        }
                    }
                    // 发送数据到上游
                    if let Some(up) = &entry.upstream {
                        if let Err(e) = up.send(&buffer[..len]).await {
                            log::debug!("UDP转发器 {} 发送到上游失败: {}", name, e);
                        } else {
                            entry.last_seen = std::time::Instant::now();
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
        // 清理会话
        self.sessions.write().await.clear();
    }
    
    pub fn get_stats(&self) -> HashMap<String, String> {
        let stats = self.stats.blocking_read();
        stats::get_stats_with_target(&stats, &self.target_addr.blocking_read())
    }
}

// 会话结构：为每个客户端维护独立的上游socket与状态
struct UdpSession {
    #[allow(dead_code)]
    client: std::net::SocketAddr,
    upstream: Option<Arc<UdpSocket>>,
    target: std::net::SocketAddr,
    last_seen: std::time::Instant,
}

impl UdpSession {
    fn new(client: std::net::SocketAddr) -> Self {
        Self {
            client,
            upstream: None,
            target: "0.0.0.0:0".parse().unwrap(),
            last_seen: std::time::Instant::now(),
        }
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