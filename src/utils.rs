use anyhow::Result;
use hickory_resolver::{
    config::{ResolverConfig, ResolverOpts},
    Resolver,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

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
            let port: u16 = parts[1]
                .parse()
                .map_err(|e| anyhow::anyhow!("无效的端口号 {}: {}", parts[1], e))?;
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

        // 添加多个DNS服务器提高解析成功率和速度
        let dns_servers = [
            "223.5.5.5:53",    // 阿里云DNS
            "223.6.6.6:53",    // 阿里云DNS
            "8.8.8.8:53",      // Google DNS
            "1.1.1.1:53",      // Cloudflare DNS
        ];

        for dns_server in &dns_servers {
            if let Ok(addr) = dns_server.parse::<SocketAddr>() {
                config.add_name_server(hickory_resolver::config::NameServerConfig::new(
                    addr,
                    hickory_resolver::config::Protocol::Udp,
                ));
            }
        }

        let mut opts = ResolverOpts::default();
        opts.timeout = Duration::from_secs(2);  // 缩短超时时间到2秒
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
    })
    .await?;

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

        // 添加多个DNS服务器提高解析成功率和速度
        let dns_servers = [
            "223.5.5.5:53",    // 阿里云DNS
            "223.6.6.6:53",    // 阿里云DNS
            "8.8.8.8:53",      // Google DNS
            "1.1.1.1:53",      // Cloudflare DNS
        ];

        for dns_server in &dns_servers {
            if let Ok(addr) = dns_server.parse::<SocketAddr>() {
                config.add_name_server(hickory_resolver::config::NameServerConfig::new(
                    addr,
                    hickory_resolver::config::Protocol::Udp,
                ));
            }
        }

        let mut opts = ResolverOpts::default();
        opts.timeout = Duration::from_secs(2);  // 缩短超时时间到2秒
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
    })
    .await?;

    let socket_addr = result?;
    Ok(socket_addr)
}

pub async fn test_connection(target: &str) -> Result<Duration> {
    let addr = resolve_target(target).await?;
    let start = Instant::now();

    // 统一使用3秒超时时间，提高故障检测速度
    match tokio::time::timeout(Duration::from_secs(3), tokio::net::TcpStream::connect(addr)).await {
        Ok(Ok(_)) => Ok(start.elapsed()),
        Ok(Err(e)) => Err(anyhow::anyhow!("连接失败 {}: {}", target, e)),
        Err(_) => Err(anyhow::anyhow!("连接超时: {}", target)),
    }
}

// UDP连接测试函数
// 已移除: UDP连通性测试函数（不再使用，避免误判）

/// 获取标准统计信息的公共函数
pub fn get_standard_stats(stats: &ConnectionStats) -> HashMap<String, String> {
    let mut result = HashMap::new();

    result.insert("connections".to_string(), stats.connections.to_string());
    result.insert("bytes_sent".to_string(), stats.bytes_sent.to_string());
    result.insert(
        "bytes_received".to_string(),
        stats.bytes_received.to_string(),
    );
    result.insert("uptime".to_string(), format!("{:?}", stats.get_uptime()));

    // 增强的性能指标
    let uptime_secs = stats.get_uptime().as_secs() as f64;
    if uptime_secs > 0.0 {
        let avg_throughput_mbps =
            (stats.bytes_sent + stats.bytes_received) as f64 / (1024.0 * 1024.0) / uptime_secs;
        result.insert(
            "avg_throughput_mbps".to_string(),
            format!("{avg_throughput_mbps:.2}"),
        );
        result.insert(
            "connections_per_hour".to_string(),
            format!("{:.1}", stats.connections as f64 * 3600.0 / uptime_secs),
        );
    }

    result
}

/// 获取带目标地址的统计信息
pub fn get_stats_with_target(
    stats: &ConnectionStats,
    target_addr: &str,
) -> HashMap<String, String> {
    let mut result = get_standard_stats(stats);
    result.insert("target_addr".to_string(), target_addr.to_string());
    result
}
