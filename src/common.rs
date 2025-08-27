use crate::config::Config;
use crate::utils::{resolve_target, test_connection};
use anyhow::Result;
use dashmap::DashMap;
use log::{error, info, warn};
use std::net::SocketAddr;
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
    
    pub async fn initialize(&self) -> Result<()> {
        // 1. DNS解析阶段：解析所有目标地址
        for rule in &self.config.rules {
            if let Err(e) = self.initialize_rule_targets(rule).await {
                error!("规则 {} DNS解析失败: {}", rule.name, e);
            }
        }
        
        // 2. 初始健康检查阶段：批量并发检查所有目标
        Self::batch_health_check(&self.target_cache).await;
        
        // 3. 选择最优地址阶段：为每个规则选择最佳目标
        Self::update_rule_targets(&self.rule_infos, &self.target_cache).await;
        
        // 4. 验证初始化结果
        let rule_infos = self.rule_infos.read().await;
        let mut available_rules = 0;
        
        for entry in rule_infos.iter() {
            let rule_name = entry.key();
            let rule_info = entry.value();
            
            if let Some(target) = &rule_info.selected_target {
                info!("规则 {}: {} -> {}", rule_name, target.original, target.resolved);
                available_rules += 1;
            }
        }
        
        info!("启动完成: {} 个规则可用", available_rules);
        
        // 5. 启动持续健康检查任务
        self.start_health_check_task().await;
        
        Ok(())
    }
    
    async fn initialize_rule_targets(&self, rule: &crate::config::ForwardRule) -> Result<()> {
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
                    
                    targets.push(target_info.clone());
                    self.target_cache.insert(target_str.clone(), target_info);
                    
                    info!("目标 {} 解析为 {}", target_str, resolved_addr);
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
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            let mut _check_count = 0;
            
            info!("启动定期健康检查任务，间隔30秒");
            
            let mut last_status = None;
            let mut is_network_down = false;
            
            loop {
                if !is_network_down {
                    interval.tick().await;
                }

                
                // 批量检查所有目标
                let current_status = Self::batch_health_check(&target_cache).await;
                
                // 更新规则信息（智能更新，避免不必要的变更）
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
    
    async fn batch_health_check(target_cache: &Arc<DashMap<String, TargetInfo>>) -> String {
        let targets: Vec<_> = target_cache.iter().map(|entry| {
            (entry.key().clone(), entry.value().clone())
        }).collect();
        
        // 并发执行健康检查
        let mut tasks = Vec::new();
        for (target_str, target_info) in targets {
            let task = tokio::spawn(async move {
                let start = Instant::now();
                let result = test_connection(&target_str).await;
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
                        
                        // 只有连续失败超过3次才标记为不健康（避免过于敏感）
                        const FAIL_THRESHOLD: u32 = 3;
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
            let new_selected_target = select_best_target(&updated_targets);
            
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

// 故障转移选择目标 - 改进的选择算法
fn select_best_target(targets: &[TargetInfo]) -> Option<TargetInfo> {
    // 1. 首先过滤出健康的目标
    let healthy_targets: Vec<_> = targets.iter()
        .filter(|t| t.healthy)
        .collect();
    
    if healthy_targets.is_empty() {
        return None;
    }
    
    // 2. 如果只有一个健康目标，直接返回
    if healthy_targets.len() == 1 {
        return healthy_targets[0].clone().into();
    }
    
    // 3. 多个健康目标时，按以下优先级选择：
    // 优先级1：延迟最低的目标（如果有延迟信息）
    // 优先级2：失败次数最少的目标
    // 优先级3：配置顺序（第一个）
    
    let mut best_target = healthy_targets[0];
    
    for target in healthy_targets.iter().skip(1) {
        // 比较延迟（如果都有延迟信息）
        if let (Some(current_latency), Some(best_latency)) = (&target.latency, &best_target.latency) {
            if current_latency < best_latency {
                best_target = target;
                continue;
            } else if current_latency > best_latency {
                continue;
            }
        }
        
        // 延迟相同或没有延迟信息时，比较失败次数
        if target.fail_count < best_target.fail_count {
            best_target = target;
        } else if target.fail_count == best_target.fail_count {
            // 失败次数相同时，选择配置顺序靠前的（保持稳定性）
            // 这里best_target已经是较早的目标，所以不需要改变
        }
    }
    
    Some(best_target.clone())
}
