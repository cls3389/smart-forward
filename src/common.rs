use crate::config::Config;
use crate::utils::resolve_target;
use anyhow::Result;
use dashmap::DashMap;
use log::{error, info, warn};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct TargetInfo {
    pub original: String,
    pub resolved: SocketAddr,
    pub healthy: bool,
    pub last_check: Instant,
    pub fail_count: u32,
    pub latency: Option<Duration>,
}

#[derive(Debug)]
pub struct RuleInfo {
    pub rule: Config,
    pub targets: Vec<TargetInfo>,
    pub selected_target: Option<TargetInfo>,
    pub last_update: Instant,
}

#[derive(Clone)]
pub struct CommonManager {
    config: Config,
    target_cache: Arc<DashMap<String, TargetInfo>>,
    rule_infos: Arc<RwLock<DashMap<String, RuleInfo>>>,
}

impl CommonManager {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            target_cache: Arc::new(DashMap::new()),
            rule_infos: Arc::new(RwLock::new(DashMap::new())),
        }
    }
    
    // 判断目标地址是否与本地接口在同一网段
    fn is_same_subnet(local_ips: &[Ipv4Addr], target_ip: IpAddr) -> bool {
        if let IpAddr::V4(target_ipv4) = target_ip {
            for &local_ip in local_ips {
                // 简单的网段判断：检查是否都是192.168.x.x或10.x.x.x或172.16-31.x.x
                let local_octets = local_ip.octets();
                let target_octets = target_ipv4.octets();
                
                // 192.168.x.x网段
                if local_octets[0] == 192 && local_octets[1] == 168 &&
                   target_octets[0] == 192 && target_octets[1] == 168 {
                    return true;
                }
                
                // 10.x.x.x网段
                if local_octets[0] == 10 && target_octets[0] == 10 {
                    return true;
                }
                
                // 172.16-31.x.x网段
                if local_octets[0] == 172 && local_octets[1] >= 16 && local_octets[1] <= 31 &&
                   target_octets[0] == 172 && target_octets[1] >= 16 && target_octets[1] <= 31 {
                    return true;
                }
            }
        } else {
            // 对于IPv6，简单判断是否为回环地址
            return target_ip.is_loopback();
        }
        
        false
    }
    
    pub async fn initialize(&self) -> Result<()> {
        // 1. 定义本地网络接口地址（简化实现，使用常见的内网地址）
        let local_interfaces = vec![
            Ipv4Addr::new(192, 168, 0, 1), 
            Ipv4Addr::new(10, 0, 0, 1),
            Ipv4Addr::new(172, 16, 0, 1)
        ];
        
        // 2. DNS解析阶段：解析所有目标地址
        for rule in &self.config.rules {
            if let Err(e) = self.initialize_rule_targets(&local_interfaces, rule).await {
                error!("规则 {} DNS解析失败: {}", rule.name, e);
            }
        }
        
        // 3. 初始健康检查阶段：批量并发检查所有目标
        // 使用快速健康检查（缩短超时时间）
        let health_check_result = Self::quick_batch_health_check(&self.target_cache, &local_interfaces).await;
        info!("初始快速健康检查完成: {}", health_check_result);
        
        // 4. 选择最优地址阶段：为每个规则选择最佳目标
        Self::update_rule_targets(&self.rule_infos, &self.target_cache).await;
        
        // 5. 验证初始化结果
        let rule_infos = self.rule_infos.read().await;
        let mut available_rules = 0;
        
        for entry in rule_infos.iter() {
            let rule_name = entry.key();
            let rule_info = entry.value();
            
            if let Some(target) = &rule_info.selected_target {
                info!("规则 {}: {} -> {}", rule_name, target.original, target.resolved);
                available_rules += 1;
            } else {
                warn!("规则 {}: 没有可用的目标地址", rule_name);
            }
        }
        
        info!("启动完成: {} 个规则可用", available_rules);
        
        // 6. 启动持续健康检查任务
        self.start_health_check_task().await;
        
        Ok(())
    }
    
    async fn initialize_rule_targets(&self, local_interfaces: &[Ipv4Addr], rule: &crate::config::ForwardRule) -> Result<()> {
        let mut targets = Vec::new();
        
        for (_priority, target_str) in rule.targets.iter().enumerate() {
            match resolve_target(target_str).await {
                Ok(resolved_addr) => {
                    let target_info = TargetInfo {
                        original: target_str.clone(),
                        resolved: resolved_addr,
                        healthy: true,
                        last_check: Instant::now(),
                        fail_count: 0,
                        latency: None,
                    };
                    
                    // 判断是否为内网地址
                    let is_local = Self::is_same_subnet(local_interfaces, resolved_addr.ip());
                    
                    targets.push(target_info.clone());
                    self.target_cache.insert(target_str.clone(), target_info);
                    
                    info!("目标 {} 解析为 {} ({})", target_str, resolved_addr, 
                          if is_local { "内网地址" } else { "外网地址" });
                }
                Err(e) => {
                    error!("无法解析目标 {}: {}", target_str, e);
                }
            }
        }
        
        let rule_info = RuleInfo {
            rule: self.config.clone(),
            targets,
            selected_target: None,
            last_update: Instant::now(),
        };
        
        self.rule_infos.write().await.insert(rule.name.clone(), rule_info);
        Ok(())
    }
    
    async fn start_health_check_task(&self) {
        let target_cache = self.target_cache.clone();
        let rule_infos = self.rule_infos.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(45)); // 统一使用配置的检查间隔
            let mut _check_count = 0;
            
            info!("启动定期健康检查任务，间隔45秒");
            
            let mut last_status = None;
            let mut is_network_down = false;
            
            loop {
                if !is_network_down {
                    interval.tick().await;
                }

                
                // 1. 首先进行DNS检查，更新所有目标地址的解析结果
                Self::update_dns_resolutions(&target_cache).await;
                
                // 2. 等待5秒后进行健康检查，避免与DNS检查冲突
                tokio::time::sleep(Duration::from_secs(5)).await;
                
                // 3. 基于最新的DNS解析结果进行健康检查
                let current_status = Self::batch_health_check(&target_cache).await;
                
                // 4. 最后更新规则目标选择（基于健康检查结果）
                Self::update_rule_targets(&rule_infos, &target_cache).await;
                
                // 检查网络状态变化
                if last_status != Some(current_status.clone()) {
                    info!("网络状态: {}", current_status);
                    last_status = Some(current_status.clone());
                    
                    // 检查是否断网 - 使用ping DNS服务器判断
                    let network_status = Self::check_network_status().await;
                    if !network_status && !is_network_down {
                        is_network_down = true;
                        info!("检测到断网，暂停健康检测");
                    } else if network_status && is_network_down {
                        is_network_down = false;
                        info!("网络恢复，立即开始健康检测");
                    }
                }
            }
        });
    }
    
    // DNS解析更新 - 定期检查DNS变化并更新target_cache
    async fn update_dns_resolutions(target_cache: &Arc<DashMap<String, TargetInfo>>) {
        let targets: Vec<_> = target_cache.iter().map(|entry| {
            (entry.key().clone(), entry.value().clone())
        }).collect();
        
        // 并发解析所有域名目标
        let mut dns_tasks = Vec::new();
        for (target_str, target_info) in targets {
            // 只对域名进行DNS解析，跳过IP地址
            if !target_str.parse::<std::net::IpAddr>().is_ok() && target_str.contains('.') {
                let task = tokio::spawn(async move {
                    match resolve_target(&target_str).await {
                        Ok(new_resolved) => {
                            if new_resolved != target_info.resolved {
                                info!("目标 {} DNS解析变化: {} -> {}", target_str, target_info.resolved, new_resolved);
                                Some((target_str, target_info, new_resolved))
                            } else {
                                None // DNS没有变化
                            }
                        }
                        Err(e) => {
                            warn!("DNS解析失败 {}: {}", target_str, e);
                            None
                        }
                    }
                });
                dns_tasks.push(task);
            }
        }
        
        // 等待所有DNS解析完成并更新缓存
        for task in dns_tasks {
            if let Ok(Some((target_str, mut target_info, new_resolved))) = task.await {
                target_info.resolved = new_resolved;
                target_info.last_check = Instant::now();
                // DNS变化时重置健康状态，让健康检查重新评估
                target_info.healthy = true;
                target_info.fail_count = 0;
                target_cache.insert(target_str, target_info);
            }
        }
    }
    
    // 检查网络状态 - ping DNS服务器
    async fn check_network_status() -> bool {
        let dns_servers = vec![
            "223.5.5.5:53",  // 阿里云DNS
            "223.6.6.6:53",  // 阿里云DNS备用
            "8.8.8.8:53",    // Google DNS
            "114.114.114.114:53", // 114 DNS
        ];
        
        for dns_server in dns_servers {
            if let Ok(_) = tokio::time::timeout(
                Duration::from_secs(3),
                tokio::net::TcpStream::connect(dns_server)
            ).await {
                return true; // 至少有一个DNS服务器可达
            }
        }
        
        false // 所有DNS服务器都不可达
    }
    
    // 快速健康检查 - 启动时使用，缩短超时时间
    async fn quick_batch_health_check(target_cache: &Arc<DashMap<String, TargetInfo>>, local_interfaces: &[Ipv4Addr]) -> String {
        let mut targets: Vec<_> = target_cache.iter().map(|entry| {
            (entry.key().clone(), entry.value().clone())
        }).collect();
        
        // 按网段优先级排序：内网地址优先检查
        targets.sort_by(|(_, a_info), (_, b_info)| {
            let a_is_local = is_same_subnet(local_interfaces, a_info.resolved.ip());
            let b_is_local = is_same_subnet(local_interfaces, b_info.resolved.ip());
            
            // 内网地址优先
            b_is_local.cmp(&a_is_local)
        });
        
        // 并发执行健康检查，使用更短的超时时间
        let mut tasks = Vec::new();
        for (target_str, target_info) in targets {
            let task = tokio::spawn(async move {
                let start = Instant::now();
                // 根据目标地址的端口号猜测协议类型
                let protocol_type = if target_str.contains(":") {
                    let parts: Vec<&str> = target_str.split(':').collect();
                    if parts.len() == 2 {
                        if let Ok(port) = parts[1].parse::<u16>() {
                            // 根据端口号猜测协议类型
                            match port {
                                53 | 123 | 161 | 162 | 500 | 514 | 520 | 1900 => "udp", // 常见UDP端口
                                _ => "tcp" // 默认使用TCP
                            }
                        } else {
                            "tcp" // 默认使用TCP
                        }
                    } else {
                        "tcp" // 默认使用TCP
                    }
                } else {
                    "tcp" // 默认使用TCP
                };
                
                // 根据协议类型选择测试方法，使用更短的超时时间（1秒）
                let result = if protocol_type == "udp" {
                    // UDP测试使用更短的超时时间
                    tokio::time::timeout(
                        Duration::from_secs(1),
                        crate::utils::test_udp_connection(&target_str)
                    ).await.unwrap_or(Err(anyhow::anyhow!("UDP连接测试超时")))
                } else {
                    // TCP测试使用更短的超时时间
                    tokio::time::timeout(
                        Duration::from_secs(1),
                        crate::utils::test_connection(&target_str)
                    ).await.unwrap_or(Err(anyhow::anyhow!("TCP连接测试超时")))
                };
                
                let check_time = start.elapsed();
                (target_str, target_info, result, check_time)
            });
            tasks.push(task);
        }
        
        // 等待所有检查完成并统计结果
        let mut success_count = 0;
        let mut fail_count = 0;
        let mut status_changes = Vec::new();
        
        for task in tasks {
            if let Ok((target_str, mut target_info, result, _check_time)) = task.await {
                let old_healthy = target_info.healthy;
                
                match result {
                    Ok(latency) => {
                        target_info.healthy = true;
                        target_info.latency = Some(latency);
                        target_info.fail_count = 0; // 成功时重置失败计数
                        target_info.last_check = Instant::now();
                        success_count += 1;
                        
                        // 如果之前不健康，现在恢复了
                        if !old_healthy {
                            status_changes.push(format!("{} 恢复", target_str));
                        }
                    }
                    Err(_e) => {
                        target_info.fail_count += 1;
                        target_info.last_check = Instant::now();
                        
                        // 简单的失败阈值：失败2次就标记为不健康
                        const FAIL_THRESHOLD: u32 = 2;
                        if target_info.fail_count >= FAIL_THRESHOLD {
                            if old_healthy {
                                target_info.healthy = false;
                                status_changes.push(format!("{} 异常（连续失败{}次）", target_str, target_info.fail_count));
                            }
                        }
                        
                        // 统计时仍然按当前健康状态计算
                        if target_info.healthy {
                            success_count += 1;
                        } else {
                            fail_count += 1;
                        }
                    }
                }
                
                target_cache.insert(target_str, target_info);
            }
        }
        
        // 生成状态摘要
        let healthy_addresses = success_count;
        let unhealthy_addresses = fail_count;
        
        if !status_changes.is_empty() {
            format!("{} 个地址健康，{} 个地址异常 [{}]", healthy_addresses, unhealthy_addresses, status_changes.join(", "))
        } else {
            format!("{} 个地址健康，{} 个地址异常", healthy_addresses, unhealthy_addresses)
        }
    }
    
    // 标准健康检查 - 定期检查使用
    async fn batch_health_check(target_cache: &Arc<DashMap<String, TargetInfo>>) -> String {
        let targets: Vec<_> = target_cache.iter().map(|entry| {
            (entry.key().clone(), entry.value().clone())
        }).collect();
        
        // 并发执行健康检查
        let mut tasks = Vec::new();
        for (target_str, target_info) in targets {
            let task = tokio::spawn(async move {
                let start = Instant::now();
                // 根据目标地址的端口号猜测协议类型
                let protocol_type = if target_str.contains(":") {
                    let parts: Vec<&str> = target_str.split(':').collect();
                    if parts.len() == 2 {
                        if let Ok(port) = parts[1].parse::<u16>() {
                            // 根据端口号猜测协议类型
                            match port {
                                53 | 123 | 161 | 162 | 500 | 514 | 520 | 1900 => "udp", // 常见UDP端口
                                _ => "tcp" // 默认使用TCP
                            }
                        } else {
                            "tcp" // 默认使用TCP
                        }
                    } else {
                        "tcp" // 默认使用TCP
                    }
                } else {
                    "tcp" // 默认使用TCP
                };
                
                // 根据协议类型选择测试方法
                let result = if protocol_type == "udp" {
                    crate::utils::test_udp_connection(&target_str).await
                } else {
                    crate::utils::test_connection(&target_str).await
                };
                
                let check_time = start.elapsed();
                (target_str, target_info, result, check_time)
            });
            tasks.push(task);
        }
        
        // 等待所有检查完成并统计结果
        let mut success_count = 0;
        let mut fail_count = 0;
        let mut status_changes = Vec::new();
        
        for task in tasks {
            if let Ok((target_str, mut target_info, result, _check_time)) = task.await {
                let old_healthy = target_info.healthy;
                
                match result {
                    Ok(latency) => {
                        target_info.healthy = true;
                        target_info.latency = Some(latency);
                        target_info.fail_count = 0; // 成功时重置失败计数
                        target_info.last_check = Instant::now();
                        success_count += 1;
                        
                        // 如果之前不健康，现在恢复了
                        if !old_healthy {
                            status_changes.push(format!("{} 恢复", target_str));
                        }
                    }
                    Err(_e) => {
                        target_info.fail_count += 1;
                        target_info.last_check = Instant::now();
                        
                        // 简单的失败阈值：失败2次就标记为不健康
                        const FAIL_THRESHOLD: u32 = 2;
                        if target_info.fail_count >= FAIL_THRESHOLD {
                            if old_healthy {
                                target_info.healthy = false;
                                status_changes.push(format!("{} 异常（连续失败{}次）", target_str, target_info.fail_count));
                            }
                        }
                        
                        // 统计时仍然按当前健康状态计算
                        if target_info.healthy {
                            success_count += 1;
                        } else {
                            fail_count += 1;
                        }
                    }
                }
                
                target_cache.insert(target_str, target_info);
            }
        }
        
        // 生成状态摘要
        let healthy_addresses = success_count;
        let unhealthy_addresses = fail_count;
        
        if !status_changes.is_empty() {
            format!("{} 个地址健康，{} 个地址异常 [{}]", healthy_addresses, unhealthy_addresses, status_changes.join(", "))
        } else {
            format!("{} 个地址健康，{} 个地址异常", healthy_addresses, unhealthy_addresses)
        }
    }
    
    async fn update_rule_targets(
        rule_infos: &Arc<RwLock<DashMap<String, RuleInfo>>>,
        target_cache: &Arc<DashMap<String, TargetInfo>>,
    ) {
        let rule_infos_write = rule_infos.write().await;
        
        for mut entry in rule_infos_write.iter_mut() {
            let rule_name = entry.key().clone();
            let rule_info = entry.value_mut();
            
            // 获取当前规则的目标列表
            let rule_targets = if let Some(rule) = rule_info.rule.rules.iter().find(|r| r.name == rule_name) {
                &rule.targets
            } else {
                continue;
            };
            
            // 更新目标信息
            let mut updated_targets = Vec::new();
            for target_str in rule_targets {
                if let Some(target_info) = target_cache.get(target_str) {
                    updated_targets.push(target_info.clone());
                }
            }
            
            // 选择最佳目标（基于健康状态、延迟和优先级）
            let new_selected_target = select_best_target_with_stickiness(&updated_targets, rule_info.selected_target.as_ref());
            
            // 检查是否需要更新目标
            let should_update = match (&rule_info.selected_target, &new_selected_target) {
                (None, Some(_)) => {
                    // 之前没有目标，现在有了
                    if let Some(target) = new_selected_target.as_ref() {
                        info!("规则 {}: {} -> {}", 
                            rule_name, target.original, target.resolved);
                    }
                    true
                }
                (Some(old), Some(new)) => {
                    // 比较新旧目标是否相同
                    if old.resolved != new.resolved {
                        info!("规则 {} 切换: {} -> {}", 
                            rule_name, old.resolved, new.resolved);
                        true
                    } else {
                        // 地址相同，不更新
                        false
                    }
                }
                (Some(_old), None) => {
                    // 之前有目标，现在没有了
                    warn!("规则 {} 不可用", rule_name);
                    true
                }
                (None, None) => {
                    // 都没有目标，不更新
                    false
                }
            };
            
            // 更新规则信息
            rule_info.targets = updated_targets;
            rule_info.last_update = Instant::now();
            
            if should_update {
                rule_info.selected_target = new_selected_target.clone();
                

            }
        }
    }
    
    pub async fn get_best_target(&self, rule_name: &str) -> Result<SocketAddr> {
        let rule_infos = self.rule_infos.read().await;
        
        if let Some(rule_info) = rule_infos.get(rule_name) {
            if let Some(target) = &rule_info.selected_target {
                return Ok(target.resolved);
            }
        }
        
        anyhow::bail!("没有可用的目标: {}", rule_name)
    }
    
    pub async fn get_best_target_string(&self, rule_name: &str) -> Result<String> {
        let addr = self.get_best_target(rule_name).await?;
        Ok(addr.to_string())
    }
}

// 故障转移选择目标 - 带地址粘性的选择算法
fn select_best_target_with_stickiness(targets: &[TargetInfo], current_target: Option<&TargetInfo>) -> Option<TargetInfo> {
    // 定义本地网络接口地址（简化实现，使用常见的内网地址）
    let local_interfaces = vec![
        Ipv4Addr::new(192, 168, 0, 1), 
        Ipv4Addr::new(10, 0, 0, 1),
        Ipv4Addr::new(172, 16, 0, 1)
    ];
    
    // 1. 首先过滤出健康的目标
    let healthy_targets: Vec<_> = targets.iter()
        .filter(|t| t.healthy)
        .collect();
    
    if healthy_targets.is_empty() {
        // 如果没有健康的目标，使用原有逻辑
        return select_best_target(targets);
    }
    
    // 2. 检查当前目标是否仍然健康
    if let Some(current) = current_target {
        if let Some(current_in_targets) = healthy_targets.iter().find(|t| t.resolved == current.resolved) {
            // 当前目标仍然健康，检查是否有更高优先级的目标可用
            
            // 为目标按配置顺序排序（配置中越靠前优先级越高）
            let mut sorted_by_priority = healthy_targets.clone();
            sorted_by_priority.sort_by(|a, b| {
                // 先按网段优先级排序（内网地址优先）
                let a_is_local = is_same_subnet(&local_interfaces, a.resolved.ip());
                let b_is_local = is_same_subnet(&local_interfaces, b.resolved.ip());
                
                let subnet_cmp = b_is_local.cmp(&a_is_local);
                if subnet_cmp != std::cmp::Ordering::Equal {
                    return subnet_cmp;
                }
                
                // 网段相同时，保持配置顺序（在targets数组中的位置）
                let a_pos = targets.iter().position(|t| t.resolved == a.resolved).unwrap_or(999);
                let b_pos = targets.iter().position(|t| t.resolved == b.resolved).unwrap_or(999);
                a_pos.cmp(&b_pos)
            });
            
            // 检查最高优先级的目标是否就是当前目标
            if let Some(highest_priority) = sorted_by_priority.first() {
                if highest_priority.resolved == current.resolved {
                    // 当前目标就是最高优先级且健康，保持不变
                    return Some((*current_in_targets).clone());
                } else {
                    // 有更高优先级的健康目标，需要切换
                    return Some((*highest_priority).clone());
                }
            }
        }
    }
    
    // 3. 当前目标不健康或不存在，选择最佳目标
    select_best_target(targets)
}

// 故障转移选择目标 - 简化版选择算法（保留原有逻辑）
fn select_best_target(targets: &[TargetInfo]) -> Option<TargetInfo> {
    // 定义本地网络接口地址（简化实现，使用常见的内网地址）
    let local_interfaces = vec![
        Ipv4Addr::new(192, 168, 0, 1), 
        Ipv4Addr::new(10, 0, 0, 1),
        Ipv4Addr::new(172, 16, 0, 1)
    ];
    
    // 1. 首先过滤出健康的目标
    let healthy_targets: Vec<_> = targets.iter()
        .filter(|t| t.healthy)
        .collect();
    
    if healthy_targets.is_empty() {
        // 如果没有健康的目标，检查是否有最近失败次数较少的目标
        // 优先选择失败次数少的目标，避免连续失败过多的目标
        let mut sorted_targets: Vec<_> = targets.iter().collect();
        sorted_targets.sort_by_key(|t| t.fail_count);
        
        // 如果有目标且失败次数较少（小于阈值），返回失败次数最少的目标
        // 这样可以在初始化时更快地选择一个可能可用的目标
        if !sorted_targets.is_empty() && sorted_targets[0].fail_count < 5 {
            return Some(sorted_targets[0].clone());
        }
        
        // 如果所有目标都失败次数较多，仍然返回第一个目标（可能是刚启动时的情况）
        if !targets.is_empty() {
            return Some(targets[0].clone());
        }
        return None;
    }
    
    // 2. 如果只有一个健康目标，直接返回
    if healthy_targets.len() == 1 {
        return healthy_targets[0].clone().into();
    }
    
    // 3. 多个健康目标时，按以下优先级选择：
    // - 优先选择内网地址
    // - 优先选择延迟更低的目标
    // - 如果延迟相近，优先选择配置中排在前面的地址
    
    let mut sorted_targets = healthy_targets.clone();
    sorted_targets.sort_by(|a, b| {
        // 首先按网段优先级排序（内网地址优先）
        let a_is_local = is_same_subnet(&local_interfaces, a.resolved.ip());
        let b_is_local = is_same_subnet(&local_interfaces, b.resolved.ip());
        
        // 内网地址优先
        let subnet_cmp = b_is_local.cmp(&a_is_local);
        if subnet_cmp != std::cmp::Ordering::Equal {
            return subnet_cmp;
        }
        
        // 然后按延迟排序
        if let (Some(a_latency), Some(b_latency)) = (&a.latency, &b.latency) {
            return a_latency.cmp(b_latency);
        }
        
        // 延迟相同或没有延迟信息时，保持原有顺序（配置中的顺序）
        std::cmp::Ordering::Equal
    });
    
    Some(sorted_targets[0].clone())
}

// 判断目标地址是否与本地接口在同一网段（独立函数）
fn is_same_subnet(local_ips: &[Ipv4Addr], target_ip: IpAddr) -> bool {
    if let IpAddr::V4(target_ipv4) = target_ip {
        for &local_ip in local_ips {
            // 简单的网段判断：检查是否都是192.168.x.x或10.x.x.x或172.16-31.x.x
            let local_octets = local_ip.octets();
            let target_octets = target_ipv4.octets();
            
            // 192.168.x.x网段
            if local_octets[0] == 192 && local_octets[1] == 168 &&
               target_octets[0] == 192 && target_octets[1] == 168 {
                return true;
            }
            
            // 10.x.x.x网段
            if local_octets[0] == 10 && target_octets[0] == 10 {
                return true;
            }
            
            // 172.16-31.x.x网段
            if local_octets[0] == 172 && local_octets[1] >= 16 && local_octets[1] <= 31 &&
               target_octets[0] == 172 && target_octets[1] >= 16 && target_octets[1] <= 31 {
                return true;
            }
        }
    } else {
        // 对于IPv6，简单判断是否为回环地址
        return target_ip.is_loopback();
    }
    
    false
}