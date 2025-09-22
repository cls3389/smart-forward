use anyhow::Result;
use hickory_resolver::{
    config::{ResolverConfig, ResolverOpts},
    Resolver,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use crate::config::DnsConfig;

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

pub async fn resolve_target(target: &str, dns_config: &DnsConfig) -> Result<SocketAddr> {
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
            resolve_with_dns(hostname, None, dns_config).await
        }
        2 => {
            // 域名:port 格式 - 解析A/AAAA记录，然后拼接端口
            let hostname = parts[0];
            let port: u16 = parts[1]
                .parse()
                .map_err(|e| anyhow::anyhow!("无效的端口号 {}: {}", parts[1], e))?;
            resolve_with_dns(hostname, Some(port), dns_config).await
        }
        _ => {
            anyhow::bail!("无效的目标格式: {}", target);
        }
    }
}

// 统一的DNS解析函数 - 支持A/AAAA和TXT记录
async fn resolve_with_dns(
    hostname: &str,
    port: Option<u16>,
    dns_config: &DnsConfig,
) -> Result<SocketAddr> {
    let hostname = hostname.to_string();
    let dns_config = dns_config.clone();

    let result = tokio::task::spawn_blocking(move || {
        // 创建DNS解析器
        let mut config = ResolverConfig::new();
        for dns_server in &dns_config.servers {
            if let Ok(addr) = dns_server.parse::<SocketAddr>() {
                config.add_name_server(hickory_resolver::config::NameServerConfig::new(
                    addr,
                    hickory_resolver::config::Protocol::Udp,
                ));
            }
        }

        let mut opts = ResolverOpts::default();
        opts.timeout = Duration::from_secs(dns_config.timeout.unwrap_or(2));
        opts.attempts = dns_config.attempts.unwrap_or(2);
        let resolver = Resolver::new(config, opts)?;

        match port {
            Some(p) => {
                // 有端口：解析A/AAAA记录，拼接端口
                match resolver.lookup_ip(&hostname) {
                    Ok(response) => {
                        if let Some(addr) = response.iter().find(|addr| addr.is_ipv4()) {
                            Ok(SocketAddr::new(addr, p))
                        } else if let Some(addr) = response.iter().next() {
                            Ok(SocketAddr::new(addr, p))
                        } else {
                            anyhow::bail!("没有找到可用的IP地址: {}", hostname)
                        }
                    }
                    Err(e) => anyhow::bail!("DNS解析失败 {}: {}", hostname, e),
                }
            }
            None => {
                // 无端口：解析TXT记录获取IP:PORT
                match resolver.txt_lookup(&hostname) {
                    Ok(txt_response) => {
                        for txt in txt_response.iter() {
                            for txt_data in txt.iter() {
                                let txt_string = String::from_utf8_lossy(txt_data);
                                let clean_txt = txt_string.trim_matches('"').trim();
                                if let Ok(addr) = clean_txt.parse::<SocketAddr>() {
                                    return Ok(addr);
                                }
                            }
                        }
                        anyhow::bail!("TXT记录中没有找到有效的IP:PORT格式: {}", hostname)
                    }
                    Err(e) => anyhow::bail!("TXT记录查询失败 {}: {}", hostname, e),
                }
            }
        }
    })
    .await?;

    result
}

pub async fn test_connection(target: &str, dns_config: &DnsConfig) -> Result<Duration> {
    let addr = resolve_target(target, dns_config).await?;
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
