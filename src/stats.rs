use std::collections::HashMap;
use crate::utils::ConnectionStats;

/// 获取标准统计信息的公共函数
pub fn get_standard_stats(stats: &ConnectionStats) -> HashMap<String, String> {
    let mut result = HashMap::new();
    
    result.insert("connections".to_string(), stats.connections.to_string());
    result.insert("bytes_sent".to_string(), stats.bytes_sent.to_string());
    result.insert("bytes_received".to_string(), stats.bytes_received.to_string());
    result.insert("uptime".to_string(), format!("{:?}", stats.get_uptime()));
    
    result
}

/// 获取带目标地址的统计信息
pub fn get_stats_with_target(stats: &ConnectionStats, target_addr: &str) -> HashMap<String, String> {
    let mut result = get_standard_stats(stats);
    result.insert("target_addr".to_string(), target_addr.to_string());
    result
}