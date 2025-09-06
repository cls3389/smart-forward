use crate::config::Config;
use crate::utils::resolve_target;
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
}

#[derive(Debug)]
pub struct RuleInfo {
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
        let health_check_result =
            Self::quick_batch_health_check(&self.target_cache, &self.config).await;
        info!("初始健康检查完成: {}", health_check_result);

        // 3. 选择最优地址阶段：为每个规则选择最佳目标
        Self::update_rule_targets(&self.rule_infos, &self.target_cache, &self.config).await;

        // 4. 验证初始化结果
        let rule_infos = self.rule_infos.read().await;
        let mut available_rules = 0;

        for entry in rule_infos.iter() {
            let rule_name = entry.key();
            let rule_info = entry.value();

            if let Some(target) = &rule_info.selected_target {
                info!(
                    "规则 {}: {} -> {}",
                    rule_name, target.original, target.resolved
                );
                available_rules += 1;
            } else {
                warn!("规则 {}: 没有可用的目标地址", rule_name);
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
                    };

                    targets.push(target_info.clone());
                    self.target_cache.insert(target_str.clone(), target_info);
                }
                Err(e) => {
                    error!("无法解析目标 {}: {}", target_str, e);
                }
            }
        }

        let rule_info = RuleInfo {
            targets,
            selected_target: None,
            last_update: Instant::now(),
        };

        self.rule_infos
            .write()
            .await
            .insert(rule.name.clone(), rule_info);
        Ok(())
    }

    async fn start_health_check_task(&self) {
        let target_cache = self.target_cache.clone();
        let rule_infos = self.rule_infos.clone();
        let config = self.config.clone(); // 传递配置信息

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(15)); // 缩短检查间隔到15秒
            let mut _check_count = 0;

            info!("启动定期健康检查任务，间隔15秒");

            let mut last_status = None;

            loop {
                // 等待检查间隔
                interval.tick().await;

                // 1. 进行DNS检查，更新所有目标地址的解析结果
                Self::update_dns_resolutions(&target_cache).await;

                // 2. 等待5秒后进行健康检查，避免与DNS检查冲突
                tokio::time::sleep(Duration::from_secs(5)).await;

                // 3. 基于最新的DNS解析结果进行健康检查
                let current_status = Self::batch_health_check(&target_cache, &config).await;

                // 4. 更新规则目标选择
                Self::update_rule_targets(&rule_infos, &target_cache, &config).await;

                // 只在状态变化时记录日志，减少重复输出
                if last_status != Some(current_status.clone()) {
                    info!("健康检查状态: {}", current_status);
                    last_status = Some(current_status.clone());
                }
            }
        });
    }

    // DNS解析更新 - 定期检查DNS变化并更新target_cache
    async fn update_dns_resolutions(target_cache: &Arc<DashMap<String, TargetInfo>>) {
        let targets: Vec<_> = target_cache
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();

        // 并发解析所有域名目标
        let mut dns_tasks = Vec::new();
        for (target_str, target_info) in targets {
            // 只对域名进行DNS解析，跳过IP地址
            if !target_str.parse::<std::net::IpAddr>().is_ok() && target_str.contains('.') {
                let task = tokio::spawn(async move {
                    match resolve_target(&target_str).await {
                        Ok(new_resolved) => {
                            if new_resolved != target_info.resolved {
                                info!(
                                    "目标 {} DNS解析变化: {} -> {}",
                                    target_str, target_info.resolved, new_resolved
                                );
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

    // 快速健康检查 - 启动时使用，缩短超时时间，根据规则配置智能选择协议
    async fn quick_batch_health_check(
        target_cache: &Arc<DashMap<String, TargetInfo>>,
        config: &Config,
    ) -> String {
        let targets: Vec<_> = target_cache
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();

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

        // 并发执行健康检查，使用统一的超时时间
        let mut tasks = Vec::new();
        for (target_str, target_info) in targets {
            let protocol_to_check = target_to_protocol
                .get(&target_str)
                .copied()
                .unwrap_or("tcp");

            let task = tokio::spawn(async move {
                let start = Instant::now();

                // 使用统一的超时时间
                let timeout_duration = Duration::from_secs(5); // 统一使用5秒超时

                // 根据规则配置决定健康检查协议
                let result = if protocol_to_check == "udp" {
                    // UDP协议：智能健康检查
                    if target_str.parse::<std::net::SocketAddr>().is_ok() {
                        // 直接IP:PORT格式，跳过检查（无法有效验证UDP服务）
                        Ok(Duration::from_millis(0))
                    } else {
                        // 域名格式，尝试DNS解析
                        match crate::utils::resolve_target(&target_str).await {
                            Ok(_) => Ok(Duration::from_millis(0)), // DNS解析成功即可
                            Err(e) => Err(anyhow::anyhow!("UDP目标解析失败: {}", e)),
                        }
                    }
                } else {
                    // TCP测试使用动态超时时间
                    tokio::time::timeout(
                        timeout_duration,
                        crate::utils::test_connection(&target_str),
                    )
                    .await
                    .unwrap_or(Err(anyhow::anyhow!("TCP连接测试超时")))
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
                    Ok(_) => {
                        target_info.healthy = true;
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

                        // 失败1次就标记为不健康，快速切换
                        if target_info.fail_count >= 1 {
                            if old_healthy {
                                target_info.healthy = false;
                                status_changes.push(format!("{} 异常", target_str));
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
            format!(
                "{} 个地址健康，{} 个地址异常 [{}]",
                healthy_addresses,
                unhealthy_addresses,
                status_changes.join(", ")
            )
        } else {
            format!(
                "{} 个地址健康，{} 个地址异常",
                healthy_addresses, unhealthy_addresses
            )
        }
    }

    // 标准健康检查 - 定期检查使用，根据规则配置智能选择协议
    async fn batch_health_check(
        target_cache: &Arc<DashMap<String, TargetInfo>>,
        config: &Config,
    ) -> String {
        let targets: Vec<_> = target_cache
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();

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
            let protocol_to_check = target_to_protocol
                .get(&target_str)
                .copied()
                .unwrap_or("tcp");

            let task = tokio::spawn(async move {
                let start = Instant::now();

                // 根据规则配置决定健康检查协议
                let result = if protocol_to_check == "udp" {
                    // UDP协议：智能健康检查
                    if target_str.parse::<std::net::SocketAddr>().is_ok() {
                        // 直接IP:PORT格式，跳过检查（无法有效验证UDP服务）
                        Ok(Duration::from_millis(0))
                    } else {
                        // 域名格式，尝试DNS解析
                        match crate::utils::resolve_target(&target_str).await {
                            Ok(_) => Ok(Duration::from_millis(0)), // DNS解析成功即可
                            Err(e) => Err(anyhow::anyhow!("UDP目标解析失败: {}", e)),
                        }
                    }
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
                    Ok(_) => {
                        target_info.healthy = true;
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

                        // 失败1次就标记为不健康，快速切换
                        if target_info.fail_count >= 1 {
                            if old_healthy {
                                target_info.healthy = false;
                                status_changes.push(format!("{} 异常", target_str));
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
            format!(
                "{} 个地址健康，{} 个地址异常 [{}]",
                healthy_addresses,
                unhealthy_addresses,
                status_changes.join(", ")
            )
        } else {
            format!(
                "{} 个地址健康，{} 个地址异常",
                healthy_addresses, unhealthy_addresses
            )
        }
    }

    async fn update_rule_targets(
        rule_infos: &Arc<RwLock<DashMap<String, RuleInfo>>>,
        target_cache: &Arc<DashMap<String, TargetInfo>>,
        config: &Config,
    ) {
        let rule_infos_write = rule_infos.write().await;

        for mut entry in rule_infos_write.iter_mut() {
            let rule_name = entry.key().clone();
            let rule_info = entry.value_mut();

            // 获取当前规则的目标列表（直接从配置中查找）
            let rule_targets =
                if let Some(rule) = config.rules.iter().find(|r| r.name == *rule_name) {
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

            // 选择最佳目标（基于健康状态和配置优先级）
            let new_selected_target = select_best_target_with_stickiness(
                &updated_targets,
                rule_info.selected_target.as_ref(),
            );

            // 检查是否需要更新目标
            let should_update = match (&rule_info.selected_target, &new_selected_target) {
                (None, Some(_)) => {
                    // 之前没有目标，现在有了
                    true
                }
                (Some(old), Some(new)) => {
                    // 比较新旧目标是否相同
                    if old.resolved != new.resolved {
                        info!(
                            "规则 {} 切换: {} -> {}",
                            rule_name, old.resolved, new.resolved
                        );
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

    #[allow(dead_code)]
    pub async fn get_best_target_string(&self, rule_name: &str) -> Result<String> {
        let addr = self.get_best_target(rule_name).await?;
        Ok(addr.to_string())
    }
}

// 简化目标选择算法 - 优先保持当前健康目标，否则按配置顺序选择
fn select_best_target_with_stickiness(
    targets: &[TargetInfo],
    current_target: Option<&TargetInfo>,
) -> Option<TargetInfo> {
    // 1. 过滤健康目标
    let healthy_targets: Vec<_> = targets.iter().filter(|t| t.healthy).collect();

    if healthy_targets.is_empty() {
        // 没有健康目标，选择配置中第一个
        return targets.first().cloned();
    }

    // 2. 检查当前目标是否仍然健康
    if let Some(current) = current_target {
        if healthy_targets
            .iter()
            .any(|t| t.resolved == current.resolved)
        {
            // 当前目标仍然健康，保持不变
            return Some(current.clone());
        }
    }

    // 3. 选择配置中最靠前的健康目标
    for target in targets {
        if target.healthy {
            return Some(target.clone());
        }
    }

    // 4. 无健康目标，返回第一个
    targets.first().cloned()
}
