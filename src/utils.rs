use anyhow::Result;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use trust_dns_resolver::{
    config::{ResolverConfig, ResolverOpts},
    Resolver,
};

pub struct ConnectionStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connections: u32,
    pub start_time: Instant,
}

impl Default for ConnectionStats {
    fn default() -> Self {
        Self {
            bytes_sent: 0,
            bytes_received: 0,
            connections: 0,
            start_time: Instant::now(),
        }
    }
}

impl ConnectionStats {
    pub fn add_bytes_sent(&mut self, bytes: u64) {
        self.bytes_sent += bytes;
    }
    
    pub fn add_bytes_received(&mut self, bytes: u64) {
        self.bytes_received += bytes;
    }
    
    pub fn increment_connections(&mut self) {
        self.connections += 1;
    }
    
    pub fn get_uptime(&self) -> Duration {
        self.start_time.elapsed()
    }
}

// 新增：带超时和重试的TCP连接函数
pub async fn connect_with_timeout_and_retry(target: SocketAddr, max_retries: u32, timeout_secs: u64, retry_delay_secs: u64, log_prefix: &str) -> Result<tokio::net::TcpStream> {
    let mut retry_count = 0;
    
    while retry_count < max_retries {
        match tokio::time::timeout(
            tokio::time::Duration::from_secs(timeout_secs),
            tokio::net::TcpStream::connect(target)
        ).await {
            Ok(Ok(stream)) => {
                if retry_count > 0 {
                    log::info!("{} 连接重试成功，重试次数: {}", log_prefix, retry_count);
                }
                return Ok(stream);
            }
            Ok(Err(e)) => {
                retry_count += 1;
                if retry_count < max_retries {
                    log::warn!("{} 连接到目标失败 {} (第{}次重试): {}，将在{}秒后重试", 
                        log_prefix, target, retry_count, e, retry_delay_secs);
                    tokio::time::sleep(tokio::time::Duration::from_secs(retry_delay_secs)).await;
                } else {
                    log::error!("{} 连接到目标失败 {} (已重试{}次): {}", 
                        log_prefix, target, max_retries, e);
                    return Err(anyhow::anyhow!("连接目标失败: {}", e));
                }
            }
            Err(_) => {
                retry_count += 1;
                if retry_count < max_retries {
                    log::warn!("{} 连接到目标超时 {} (第{}次重试)，将在{}秒后重试", 
                        log_prefix, target, retry_count, retry_delay_secs);
                    tokio::time::sleep(tokio::time::Duration::from_secs(retry_delay_secs)).await;
                } else {
                    log::error!("{} 连接到目标超时 {} (已重试{}次)", 
                        log_prefix, target, max_retries);
                    return Err(anyhow::anyhow!("连接目标超时")); 
                }
            }
        }
    }
    
    Err(anyhow::anyhow!("无法建立到目标的连接"))
}

pub async fn resolve_target(target: &str) -> Result<SocketAddr> {
    // 1. 尝试直接解析为SocketAddr (IP:PORT格式)
    if let Ok(addr) = target.parse::<SocketAddr>() {
        return Ok(addr);
    }
    
    // 2. 解析 hostname:port 格式
    let parts: Vec<&str> = target.split(':').collect();
    match parts.len() {
        1 => {
            // 纯域名 - 解析TXT记录获取IP:PORT
            let hostname = parts[0];
            resolve_txt_record_with_aliyun_dns(hostname).await
        }
        2 => {
            // 域名:port 格式 - 解析A/AAAA记录，然后拼接端口
            let hostname = parts[0];
            let port: u16 = parts[1].parse().map_err(|e| {
                anyhow::anyhow!("无效的端口号 {}: {}", parts[1], e)
            })?;
            resolve_domain_with_aliyun_dns(hostname, port).await
        }
        _ => {
            anyhow::bail!("无效的目标格式: {}", target);
        }
    }
}

// 解析域名:PORT格式 - 解析A/AAAA记录，然后拼接端口
async fn resolve_domain_with_aliyun_dns(hostname: &str, port: u16) -> Result<SocketAddr> {
    // 使用tokio的spawn_blocking来运行同步DNS解析
    let hostname = hostname.to_string();
    let result = tokio::task::spawn_blocking(move || {
        // 创建阿里云DNS解析器
        let mut config = ResolverConfig::new();
        
        // 添加阿里云DNS服务器
        let aliyun_dns1: SocketAddr = "223.5.5.5:53".parse()?;
        let aliyun_dns2: SocketAddr = "223.6.6.6:53".parse()?;
        
        config.add_name_server(trust_dns_resolver::config::NameServerConfig::new(
            aliyun_dns1,
            trust_dns_resolver::config::Protocol::Udp,
        ));
        config.add_name_server(trust_dns_resolver::config::NameServerConfig::new(
            aliyun_dns2,
            trust_dns_resolver::config::Protocol::Udp,
        ));
        
        let mut opts = ResolverOpts::default();
        opts.timeout = Duration::from_secs(5);
        opts.attempts = 2;
        
        let resolver = Resolver::new(config, opts)?;
        
        // 使用阿里云DNS解析A和AAAA记录
        match resolver.lookup_ip(&hostname) {
            Ok(response) => {
                // 优先选择IPv4地址
                if let Some(addr) = response.iter().find(|addr| addr.is_ipv4()) {
                    let socket_addr = SocketAddr::new(addr, port);
                    Ok(socket_addr)
                } else if let Some(addr) = response.iter().next() {
                    let socket_addr = SocketAddr::new(addr, port);
                    Ok(socket_addr)
                } else {
                    anyhow::bail!("没有找到可用的IP地址: {}", hostname)
                }
            }
            Err(e) => {
                anyhow::bail!("DNS解析失败 {}: {}", hostname, e)
            }
        }
    }).await?;
    
    let socket_addr = result?;
    Ok(socket_addr)
}

// 解析纯域名TXT记录 - 从TXT记录中获取IP:PORT
async fn resolve_txt_record_with_aliyun_dns(hostname: &str) -> Result<SocketAddr> {
    // 使用tokio的spawn_blocking来运行同步DNS解析
    let hostname = hostname.to_string();
    let result = tokio::task::spawn_blocking(move || {
        // 创建阿里云DNS解析器
        let mut config = ResolverConfig::new();
        
        // 添加阿里云DNS服务器
        let aliyun_dns1: SocketAddr = "223.5.5.5:53".parse()?;
        let aliyun_dns2: SocketAddr = "223.6.6.6:53".parse()?;
        
        config.add_name_server(trust_dns_resolver::config::NameServerConfig::new(
            aliyun_dns1,
            trust_dns_resolver::config::Protocol::Udp,
        ));
        config.add_name_server(trust_dns_resolver::config::NameServerConfig::new(
            aliyun_dns2,
            trust_dns_resolver::config::Protocol::Udp,
        ));
        
        let mut opts = ResolverOpts::default();
        opts.timeout = Duration::from_secs(5);
        opts.attempts = 2;
        
        let resolver = Resolver::new(config, opts)?;
        
        // 查询TXT记录
        match resolver.txt_lookup(&hostname) {
            Ok(txt_response) => {
                for txt in txt_response.iter() {
                    for txt_data in txt.iter() {
                        let txt_string = String::from_utf8_lossy(txt_data);
                        
                        // 清理TXT记录内容（移除引号、空格等）
                        let clean_txt = txt_string.trim_matches('"').trim();
                        
                        // 尝试解析TXT记录中的IP:PORT格式
                        if let Ok(addr) = clean_txt.parse::<SocketAddr>() {
                            return Ok(addr);
                        }
                    }
                }
                anyhow::bail!("TXT记录中没有找到有效的IP:PORT格式: {}", hostname)
            }
            Err(e) => {
                anyhow::bail!("TXT记录查询失败 {}: {}", hostname, e)
            }
        }
    }).await?;
    
    let socket_addr = result?;
    Ok(socket_addr)
}

pub async fn test_connection(target: &str) -> Result<Duration> {
    let addr = resolve_target(target).await?;
    let start = Instant::now();
    
    // 增加超时时间到8秒，适应外网地址的延迟
    match tokio::time::timeout(Duration::from_secs(8), tokio::net::TcpStream::connect(addr)).await {
        Ok(Ok(_)) => Ok(start.elapsed()),
        Ok(Err(e)) => Err(anyhow::anyhow!("连接失败 {}: {}", target, e)),
        Err(_) => Err(anyhow::anyhow!("连接超时: {}", target)),
    }
}

// UDP连接测试函数
pub async fn test_udp_connection(target: &str) -> Result<Duration> {
    let addr = resolve_target(target).await?;
    let start = Instant::now();
    
    // 创建UDP socket并尝试发送数据，增加超时时间为5秒
    match tokio::time::timeout(Duration::from_secs(5), async {
        let socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;
        // 发送一个空的UDP包作为测试
        socket.send_to(&[], addr).await?;
        Ok::<(), anyhow::Error>(())
    }).await {
        Ok(Ok(_)) => Ok(start.elapsed()),
        Ok(Err(e)) => Err(anyhow::anyhow!("UDP连接测试失败 {}: {}", target, e)),
        Err(_) => Err(anyhow::anyhow!("UDP连接测试超时: {}", target)),
    }
}