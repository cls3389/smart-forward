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
        info!("初始健康检查完成: {health_check_result}");

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
                warn!("规则 {rule_name}: 没有可用的目标地址");
            }
        }

        info!("启动完成: {available_rules} 个规则可用");

        // 5. 启动持续健康检查任务
        self.start_health_check_task().await;

        Ok(())
    }

    async fn initialize_rule_targets(&self, rule: &crate::config::ForwardRule) -> Result<()> {
        let mut targets = Vec::new();

        for target_str in rule.targets.iter() {
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
                    error!("无法解析目标 {target_str}: {e}");
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

                // 1. 进行DNS检查并立即验证连接（已包含连接验证）
                Self::update_dns_resolutions(&target_cache, &rule_infos, &config).await;

                // 2. 对所有目标进行常规健康检查（补充验证）
                let current_status = Self::batch_health_check(&target_cache, &config).await;

                // 3. 立即更新规则目标选择（基于最新的健康状态）
                Self::update_rule_targets(&rule_infos, &target_cache, &config).await;

                // 只在状态变化时记录日志，减少重复输出
                if last_status != Some(current_status.clone()) {
                    info!("健康检查状态: {current_status}");
                    last_status = Some(current_status.clone());
                }
            }
        });
    }

    // DNS解析更新 - 智能批量检查，确保所有域名都参与检测和更新
    async fn update_dns_resolutions(
        target_cache: &Arc<DashMap<String, TargetInfo>>,
        rule_infos: &Arc<RwLock<DashMap<String, RuleInfo>>>,
        config: &Config,
    ) {
        let targets: Vec<_> = target_cache
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();

        // 第一阶段：检测是否有任何DNS变化或解析失败的域名需要重试
        let mut needs_batch_update = false;
        let mut dns_check_tasks = Vec::new();

        for (target_str, target_info) in targets.iter() {
            // 只对域名进行DNS解析检查，跳过IP地址
            if target_str.parse::<std::net::IpAddr>().is_err() && target_str.contains('.') {
                let target_str_clone = target_str.clone();
                let target_info_clone = target_info.clone();
                let task = tokio::spawn(async move {
                    match resolve_target(&target_str_clone).await {
                        Ok(new_resolved) => {
                            if new_resolved != target_info_clone.resolved {
                                // DNS解析结果变化
                                Some((target_str_clone, "changed".to_string()))
                            } else {
                                // DNS解析结果未变化
                                None
                            }
                        }
                        Err(_) => {
                            // DNS解析失败，如果目标当前是不健康的，也需要批量更新来重试
                            if !target_info_clone.healthy {
                                Some((target_str_clone, "retry_failed".to_string()))
                            } else {
                                // 当前健康但本次解析失败，可能需要重试
                                Some((target_str_clone, "newly_failed".to_string()))
                            }
                        }
                    }
                });
                dns_check_tasks.push(task);
            }
        }

        // 收集需要更新的原因
        let mut update_reasons = Vec::new();
        for task in dns_check_tasks {
            if let Ok(Some((domain, reason))) = task.await {
                update_reasons.push((domain, reason));
                needs_batch_update = true;
            }
        }

        // 如果需要批量更新（有变化或需要重试失败的域名）
        if needs_batch_update {
            info!("检测到DNS变化或失败域名，触发所有域名批量重新解析和验证");

            // 记录触发批量更新的原因
            for (domain, reason) in update_reasons {
                match reason.as_str() {
                    "changed" => info!("触发原因: {domain} DNS解析变化"),
                    "retry_failed" => info!("触发原因: {domain} 重试解析失败的域名"),
                    "newly_failed" => info!("触发原因: {domain} 新发现解析失败"),
                    _ => {}
                }
            }

            // 第二阶段：对所有域名进行重新解析（包括之前失败的）
            let mut batch_resolve_tasks = Vec::new();
            for (target_str, target_info) in targets {
                // 只处理域名，跳过IP地址
                if target_str.parse::<std::net::IpAddr>().is_err() && target_str.contains('.') {
                    let task = tokio::spawn(async move {
                        match resolve_target(&target_str).await {
                            Ok(new_resolved) => {
                                let mut updated_info = target_info.clone();
                                let has_changed = new_resolved != target_info.resolved;
                                let was_failed = !target_info.healthy;

                                if has_changed {
                                    info!(
                                        "目标 {} DNS解析变化: {} -> {}",
                                        target_str, target_info.resolved, new_resolved
                                    );
                                } else if was_failed {
                                    info!("目标 {target_str} DNS重新解析成功: {new_resolved}");
                                }

                                updated_info.resolved = new_resolved;
                                updated_info.last_check = Instant::now();

                                Some((target_str, updated_info, has_changed || was_failed))
                            }
                            Err(e) => {
                                warn!("目标 {target_str} DNS重新解析仍然失败: {e}");
                                // 即使解析失败，也要更新last_check时间
                                let mut failed_info = target_info.clone();
                                failed_info.last_check = Instant::now();
                                failed_info.healthy = false;
                                failed_info.fail_count += 1;
                                Some((target_str, failed_info, false))
                            }
                        }
                    });
                    batch_resolve_tasks.push(task);
                }
            }

            // 第三阶段：并发验证所有处理过的地址
            let mut verification_tasks = Vec::new();
            for task in batch_resolve_tasks {
                if let Ok(Some((target_str, mut target_info, should_verify))) = task.await {
                    if should_verify {
                        // 对有变化或之前失败的域名进行连接验证
                        let verification_task = tokio::spawn(async move {
                            let connection_result = tokio::time::timeout(
                                Duration::from_secs(5),
                                crate::utils::test_connection(&target_str),
                            )
                            .await;

                            match connection_result {
                                Ok(Ok(_)) => {
                                    target_info.healthy = true;
                                    target_info.fail_count = 0;
                                    info!("目标 {target_str} 重新验证连接成功");
                                }
                                Ok(Err(e)) => {
                                    target_info.healthy = false;
                                    target_info.fail_count += 1;
                                    warn!("目标 {target_str} 重新验证连接失败: {e}");
                                }
                                Err(_) => {
                                    target_info.healthy = false;
                                    target_info.fail_count += 1;
                                    warn!("目标 {target_str} 重新验证连接超时");
                                }
                            }

                            (target_str, target_info)
                        });
                        verification_tasks.push(verification_task);
                    } else {
                        // 直接更新缓存（解析失败的情况）
                        target_cache.insert(target_str, target_info);
                    }
                }
            }

            // 批量更新缓存
            for task in verification_tasks {
                if let Ok((target_str, target_info)) = task.await {
                    target_cache.insert(target_str, target_info);
                }
            }

            info!("批量DNS重新解析和验证完成");
            
            // 延迟更新规则目标，确保连接验证有足够时间完成
            tokio::time::sleep(Duration::from_secs(2)).await;
            Self::update_rule_targets(rule_infos, target_cache, config).await;
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
                            status_changes.push(format!("{target_str} 恢复"));
                        }
                    }
                    Err(_e) => {
                        target_info.fail_count += 1;
                        target_info.last_check = Instant::now();

                        // 失败1次就标记为不健康，快速切换
                        if target_info.fail_count >= 1 && old_healthy {
                            target_info.healthy = false;
                            status_changes.push(format!("{target_str} 异常"));
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
            format!("{healthy_addresses} 个地址健康，{unhealthy_addresses} 个地址异常")
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
                            status_changes.push(format!("{target_str} 恢复"));
                        }
                    }
                    Err(_e) => {
                        target_info.fail_count += 1;
                        target_info.last_check = Instant::now();

                        // 失败1次就标记为不健康，快速切换
                        if target_info.fail_count >= 1 && old_healthy {
                            target_info.healthy = false;
                            status_changes.push(format!("{target_str} 异常"));
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
            format!("{healthy_addresses} 个地址健康，{unhealthy_addresses} 个地址异常")
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

            // 检查是否需要更新目标 - 简化逻辑，智能判断已在选择算法中处理
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
                    warn!("规则 {rule_name} 不可用");
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

// 智能目标选择算法 - 保守策略，优先稳定性，集成智能判断
fn select_best_target_with_stickiness(
    targets: &[TargetInfo],
    current_target: Option<&TargetInfo>,
) -> Option<TargetInfo> {
    if targets.is_empty() {
        return None;
    }

    // 1. 过滤健康目标
    let healthy_targets: Vec<_> = targets.iter().filter(|t| t.healthy).collect();

    // 2. 核心策略：如果当前目标仍然健康，坚持使用（最高优先级）
    if let Some(current) = current_target {
        if current.healthy && healthy_targets.iter().any(|t| t.resolved == current.resolved) {
            // 当前目标仍然健康，保持稳定性
            return Some(current.clone());
        }
    }

    // 3. 智能判断：避免从健康目标切换到不健康目标
    if let Some(current) = current_target {
        if current.healthy && healthy_targets.is_empty() {
            // 当前目标健康但没有其他健康目标，保持现状
            return Some(current.clone());
        }
    }

    // 4. 选择最优健康目标：按配置优先级选择
    if !healthy_targets.is_empty() {
        for target in targets {
            if target.healthy {
                // 如果选择的目标与当前不同，输出选择原因
                if let Some(current) = current_target {
                    if target.resolved != current.resolved {
                        log::debug!(
                            "选择新目标: {} (健康:{}) 替换 {} (健康:{})",
                            target.resolved, target.healthy,
                            current.resolved, current.healthy
                        );
                    }
                }
                return Some(target.clone());
            }
        }
    }

    // 5. 保守策略：没有健康目标时保持当前目标，避免无意义切换
    if let Some(current) = current_target {
        log::debug!(
            "保持当前目标: {} (无更好选择)",
            current.resolved
        );
        return Some(current.clone());
    }

    // 6. 初始化场景：选择配置中第一个作为起始目标
    if let Some(first) = targets.first() {
        log::debug!(
            "初始化选择: {} (健康:{})",
            first.resolved, first.healthy
        );
        return Some(first.clone());
    }

    None
}
