use crate::config::Config;
use crate::utils::resolve_target;
use anyhow::Result;
use dashmap::DashMap;
use log::{error, info, warn};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use local_ip_address::{local_ip, list_afinet_netifas};

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
    local_interfaces: Arc<Vec<Ipv4Addr>>, // 缓存本地网络接口
}

impl CommonManager {
    pub fn new(config: Config) -> Self {
        let local_interfaces = Arc::new(Self::get_local_interfaces());
        Self {
            config,
            target_cache: Arc::new(DashMap::new()),
            rule_infos: Arc::new(RwLock::new(DashMap::new())),
            local_interfaces,
        }
    }
    
    // 获取本地网络接口地址
    pub fn get_local_interfaces() -> Vec<Ipv4Addr> {
        let mut local_ips = Vec::new();
        
        // 方法1: 获取默认路由使用的IP地址
        if let Ok(ip) = local_ip() {
            if let IpAddr::V4(ipv4) = ip {
                local_ips.push(ipv4);
            }
        }
        
        // 方法2: 获取所有网络接口的IP地址
        if let Ok(network_interfaces) = list_afinet_netifas() {
            for (name, ip) in network_interfaces {
                if let IpAddr::V4(ipv4) = ip {
                    // 排除回环地址
                    if !ipv4.is_loopback() {
                        if !local_ips.contains(&ipv4) {
                            local_ips.push(ipv4);
                            info!("检测到网络接口 {}: {}", name, ipv4);
                        }
                    }
                }
            }
        }
        
        // 如果没有检测到任何地址，使用默认的内网地址作为fallback
        if local_ips.is_empty() {
            warn!("未检测到本地网络接口，使用默认内网地址");
            local_ips = vec![
                Ipv4Addr::new(192, 168, 0, 1), 
                Ipv4Addr::new(10, 0, 0, 1),
                Ipv4Addr::new(172, 16, 0, 1)
            ];
        } else {
            info!("检测到本地网络接口: {:?}", local_ips);
        }
        
        local_ips
    }
    
    // 判断目标地址是否与本地接口在同一网段
    fn is_same_subnet(local_ips: &[Ipv4Addr], target_ip: IpAddr) -> bool {
        if let IpAddr::V4(target_ipv4) = target_ip {
            for &local_ip in local_ips {
                // 基于实际网卡地址进行精确的网段匹配
                let local_octets = local_ip.octets();
                let target_octets = target_ipv4.octets();
                
                // 优先检查是否为相同的/24网段（前3个字节相同）
                if local_octets[0] == target_octets[0] && 
                   local_octets[1] == target_octets[1] && 
                   local_octets[2] == target_octets[2] {
                    return true;
                }
                
                // 对于常见的大型内网段，使用标准的子网掩码（但只有在同一个子网内才认为是内网）
                
                // 10.x.x.x/8 网段 - 但只有前两个字节相同才认为同网段
                if local_octets[0] == 10 && target_octets[0] == 10 &&
                   local_octets[1] == target_octets[1] {
                    return true;
                }
                
                // 172.16-31.x.x/12 网段 - 但只有前两个字节相同才认为同网段
                if local_octets[0] == 172 && local_octets[1] >= 16 && local_octets[1] <= 31 &&
                   target_octets[0] == 172 && target_octets[1] >= 16 && target_octets[1] <= 31 &&
                   local_octets[1] == target_octets[1] {
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
        // 1. 使用缓存的本地网络接口地址
        let local_interfaces = &self.local_interfaces;
        
        // 2. DNS解析阶段：解析所有目标地址
        for rule in &self.config.rules {
            if let Err(e) = self.initialize_rule_targets(local_interfaces, rule).await {
                error!("规则 {} DNS解析失败: {}", rule.name, e);
            }
        }
        
        // 3. 初始健康检查阶段：批量并发检查所有目标
        // 使用快速健康检查（缩短超时时间）
        let health_check_result = Self::quick_batch_health_check(&self.target_cache, &local_interfaces, &self.config).await;
        info!("初始快速健康检查完成: {}", health_check_result);
        
        // 4. 选择最优地址阶段：为每个规则选择最佳目标
        Self::update_rule_targets(&self.rule_infos, &self.target_cache, local_interfaces).await;
        
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
        let local_interfaces = self.local_interfaces.clone();
        let config = self.config.clone(); // 传递配置信息
        
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
                
                // 3. 基于最新的DNS解析结果进行健康检查（传递规则配置）
                let current_status = Self::batch_health_check(&target_cache, &config).await;
                
                // 4. 最后更新规则目标选择（基于健康检查结果）
                Self::update_rule_targets(&rule_infos, &target_cache, &local_interfaces).await;
                
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
    
    // 快速健康检查 - 启动时使用，缩短超时时间，根据规则配置智能选择协议
    async fn quick_batch_health_check(target_cache: &Arc<DashMap<String, TargetInfo>>, local_interfaces: &[Ipv4Addr], config: &Config) -> String {
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
        
        // 建立目标地址到规则的映射，用于决定健康检查协议
        let mut target_to_protocol = std::collections::HashMap::new();
        for rule in &config.rules {
            let protocols = rule.get_protocols();
            for target_str in &rule.targets {
                // 对于TCP+UDP规则，只检查TCP；对于纯UDP规则，检查UDP
                let check_protocol = if protocols.len() == 1 && protocols[0] == "udp" {
                    "udp" // 只有纯UDP规则才检查UDP
                } else {
                    "tcp" // 其他情况都检查TCP（包括TCP+UDP规则）
                };
                target_to_protocol.insert(target_str.clone(), check_protocol);
            }
        }
        
        // 并发执行健康检查，使用更短的超时时间
        let mut tasks = Vec::new();
        for (target_str, target_info) in targets {
            let local_interfaces = local_interfaces.to_vec(); // 克隆到task中
            let protocol_to_check = target_to_protocol.get(&target_str).copied().unwrap_or("tcp");
            
            let task = tokio::spawn(async move {
                let start = Instant::now();
                
                // 根据地址类型选择适合的超时时间
                let timeout_duration = if is_same_subnet(&local_interfaces, target_info.resolved.ip()) {
                    Duration::from_secs(3) // 内网地址使用3秒超时
                } else {
                    Duration::from_secs(8) // 外网地址使用8秒超时
                };
                
                // 根据规则配置决定健康检查协议
                let result = if protocol_to_check == "udp" {
                    // UDP测试使用较短的超时时间
                    tokio::time::timeout(
                        timeout_duration.min(Duration::from_secs(5)), // UDP最多5秒
                        crate::utils::test_udp_connection(&target_str)
                    ).await.unwrap_or(Err(anyhow::anyhow!("UDP连接测试超时")))
                } else {
                    // TCP测试使用动态超时时间
                    tokio::time::timeout(
                        timeout_duration,
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
    
    // 标准健康检查 - 定期检查使用，根据规则配置智能选择协议
    async fn batch_health_check(target_cache: &Arc<DashMap<String, TargetInfo>>, config: &Config) -> String {
        let targets: Vec<_> = target_cache.iter().map(|entry| {
            (entry.key().clone(), entry.value().clone())
        }).collect();
        
        // 建立目标地址到规则的映射，用于决定健康检查协议
        let mut target_to_protocol = std::collections::HashMap::new();
        for rule in &config.rules {
            let protocols = rule.get_protocols();
            for target_str in &rule.targets {
                // 对于TCP+UDP规则，只检查TCP；对于纯UDP规则，检查UDP
                let check_protocol = if protocols.len() == 1 && protocols[0] == "udp" {
                    "udp" // 只有纯UDP规则才检查UDP
                } else {
                    "tcp" // 其他情况都检查TCP（包括TCP+UDP规则）
                };
                target_to_protocol.insert(target_str.clone(), check_protocol);
            }
        }
        
        // 并发执行健康检查
        let mut tasks = Vec::new();
        for (target_str, target_info) in targets {
            let protocol_to_check = target_to_protocol.get(&target_str).copied().unwrap_or("tcp");
            
            let task = tokio::spawn(async move {
                let start = Instant::now();
                
                // 根据规则配置决定健康检查协议
                let result = if protocol_to_check == "udp" {
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
        local_interfaces: &[Ipv4Addr],
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
            let new_selected_target = select_best_target_with_stickiness(&updated_targets, rule_info.selected_target.as_ref(), local_interfaces);
            
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
fn select_best_target_with_stickiness(targets: &[TargetInfo], current_target: Option<&TargetInfo>, local_interfaces: &[Ipv4Addr]) -> Option<TargetInfo> {
    // 1. 首先过滤出健康的目标
    let healthy_targets: Vec<_> = targets.iter()
        .filter(|t| t.healthy)
        .collect();
    
    if healthy_targets.is_empty() {
        // 如果没有健康的目标，使用原有逻辑
        return select_best_target(targets, local_interfaces);
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
    select_best_target(targets, local_interfaces)
}

// 故障转移选择目标 - 智能选择算法（避免非同网段内网地址优先）
fn select_best_target(targets: &[TargetInfo], local_interfaces: &[Ipv4Addr]) -> Option<TargetInfo> {
    // 1. 首先过滤出健康的目标
    let healthy_targets: Vec<_> = targets.iter()
        .filter(|t| t.healthy)
        .collect();
    
    if healthy_targets.is_empty() {
        // 如果没有健康的目标，使用智能初始选择策略
        return select_initial_target_when_unhealthy(targets, local_interfaces);
    }
    
    // 2. 如果只有一个健康目标，直接返回
    if healthy_targets.len() == 1 {
        return healthy_targets[0].clone().into();
    }
    
    // 3. 多个健康目标时，使用优化的选择策略
    select_from_healthy_targets(&healthy_targets, local_interfaces)
}

// 当没有健康目标时的智能初始选择策略
fn select_initial_target_when_unhealthy(targets: &[TargetInfo], local_interfaces: &[Ipv4Addr]) -> Option<TargetInfo> {
    if targets.is_empty() {
        return None;
    }
    
    // 按智能优先级排序，但在相同优先级内严格按配置顺序
    let mut sorted_targets: Vec<_> = targets.iter().enumerate().collect();
    sorted_targets.sort_by(|(a_idx, a), (b_idx, b)| {
        let a_priority = get_target_priority(&a, local_interfaces);
        let b_priority = get_target_priority(&b, local_interfaces);
        
        // 首先按智能优先级排序（数字越小优先级越高）
        let priority_cmp = a_priority.cmp(&b_priority);
        if priority_cmp != std::cmp::Ordering::Equal {
            return priority_cmp;
        }
        
        // 相同智能优先级时，严格按配置顺序（索引越小优先级越高）
        let config_cmp = a_idx.cmp(b_idx);
        if config_cmp != std::cmp::Ordering::Equal {
            return config_cmp;
        }
        
        // 配置顺序相同时，选择失败次数较少的
        a.fail_count.cmp(&b.fail_count)
    });
    
    // 返回优先级最高且失败次数相对较少的目标
    if let Some((_, best)) = sorted_targets.first() {
        if best.fail_count < 10 { // 避免选择失败次数过多的目标
            return Some((*best).clone());
        }
    }
    
    // 如果所有目标失败次数都很多，仍然返回优先级最高的
    Some(sorted_targets[0].1.clone())
}

// 从健康目标中选择最佳目标
fn select_from_healthy_targets(healthy_targets: &[&TargetInfo], local_interfaces: &[Ipv4Addr]) -> Option<TargetInfo> {
    // 需要在原始目标列表中找到配置位置，这里使用一个简化的方法
    let mut sorted_targets = healthy_targets.to_vec();
    sorted_targets.sort_by(|a, b| {
        // 先按网段优先级排序
        let a_is_local = is_same_subnet(&local_interfaces, a.resolved.ip());
        let b_is_local = is_same_subnet(&local_interfaces, b.resolved.ip());
        
        // 同网段地址优先
        let subnet_cmp = b_is_local.cmp(&a_is_local);
        if subnet_cmp != std::cmp::Ordering::Equal {
            return subnet_cmp;
        }
        
        // 相同网段内，按延迟排序（优先选择延迟低的）
        if let (Some(a_latency), Some(b_latency)) = (&a.latency, &b.latency) {
            return a_latency.cmp(b_latency);
        }
        
        // 延迟相同或没有延迟信息时，按原始地址字符串排序（保持相对稳定的顺序）
        a.original.cmp(&b.original)
    });
    
    Some(sorted_targets[0].clone())
}

// 获取目标地址的优先级（数字越小优先级越高）
fn get_target_priority(target: &TargetInfo, local_interfaces: &[Ipv4Addr]) -> u8 {
    let ip = target.resolved.ip();
    
    // 1. 真正的同网段内网地址 - 最高优先级
    if is_same_subnet(local_interfaces, ip) {
        return 1;
    }
    
    // 2. 判断是否为内网IP地址
    let is_private_ip = match ip {
        IpAddr::V4(ipv4) => {
            let octets = ipv4.octets();
            // 10.0.0.0/8
            (octets[0] == 10) ||
            // 172.16.0.0/12
            (octets[0] == 172 && octets[1] >= 16 && octets[1] <= 31) ||
            // 192.168.0.0/16
            (octets[0] == 192 && octets[1] == 168)
        }
        IpAddr::V6(_) => false, // 简化处理，IPv6暂时当作外网地址
    };
    
    if is_private_ip {
        // 3. 非同网段的内网地址 - 最低优先级（避免初始选择）
        return 10;
    } else {
        // 2. 外网地址（公网IP或通过域名解析得到的地址）- 中等优先级
        return 5;
    }
}

// 判断目标地址是否与本地接口在同一网段（独立函数）
fn is_same_subnet(local_ips: &[Ipv4Addr], target_ip: IpAddr) -> bool {
    if let IpAddr::V4(target_ipv4) = target_ip {
        for &local_ip in local_ips {
            // 基于实际网卡地址进行精确的网段匹配
            let local_octets = local_ip.octets();
            let target_octets = target_ipv4.octets();
            
            // 优先检查是否为相同的/24网段（前3个字节相同）
            if local_octets[0] == target_octets[0] && 
               local_octets[1] == target_octets[1] && 
               local_octets[2] == target_octets[2] {
                return true;
            }
            
            // 对于常见的大型内网段，使用标准的子网掩码（但只有在同一个子网内才认为是内网）
            
            // 10.x.x.x/8 网段 - 但只有前两个字节相同才认为同网段
            if local_octets[0] == 10 && target_octets[0] == 10 &&
               local_octets[1] == target_octets[1] {
                return true;
            }
            
            // 172.16-31.x.x/12 网段 - 但只有前两个字节相同才认为同网段
            if local_octets[0] == 172 && local_octets[1] >= 16 && local_octets[1] <= 31 &&
               target_octets[0] == 172 && target_octets[1] >= 16 && target_octets[1] <= 31 &&
               local_octets[1] == target_octets[1] {
                return true;
            }
        }
    } else {
        // 对于IPv6，简单判断是否为回环地址
        return target_ip.is_loopback();
    }
    
    false
}