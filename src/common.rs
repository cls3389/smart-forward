use crate::config::Config;
use crate::utils::resolve_target;
use anyhow::Result;
use dashmap::DashMap;
use log::{error, info, warn};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

// 目标切换回调函数类型
pub type TargetSwitchCallback = Arc<dyn Fn(&str, &str, &str) + Send + Sync>;

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
    target_switch_callback: Option<TargetSwitchCallback>,
}

impl CommonManager {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            target_cache: Arc::new(DashMap::new()),
            rule_infos: Arc::new(RwLock::new(DashMap::new())),
            target_switch_callback: None,
        }
    }

    pub fn set_target_switch_callback(&mut self, callback: TargetSwitchCallback) {
        self.target_switch_callback = Some(callback);
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
        Self::update_rule_targets(
            &self.rule_infos,
            &self.target_cache,
            &self.config,
            &self.target_switch_callback,
        )
        .await;

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
        let callback = self.target_switch_callback.clone();

        tokio::spawn(async move {
            let check_interval = config.get_dynamic_update_config().get_check_interval();
            let mut interval = tokio::time::interval(Duration::from_secs(check_interval));
            let mut _check_count = 0;

            info!("启动定期健康检查任务，间隔{}秒", check_interval);

            let mut last_status = None;

            loop {
                // 等待检查间隔
                interval.tick().await;

                // 1. 重新解析所有域名的DNS（每次检查间隔都解析）
                Self::update_dns_resolutions(&target_cache, &rule_infos, &config, &callback).await;

                // 2. 对所有目标进行健康检查（根据协议类型选择检查方式）
                let current_status = Self::batch_health_check(&target_cache, &config).await;

                // 3. 异常后立即切换到可用的最高优先级地址
                Self::update_rule_targets(&rule_infos, &target_cache, &config, &callback).await;

                // 只在状态变化时记录日志，减少重复输出
                if last_status != Some(current_status.clone()) {
                    info!("健康检查状态: {current_status}");
                    last_status = Some(current_status.clone());
                }
            }
        });
    }

    // DNS解析更新 - 每次检查间隔都重新解析所有域名，确保地址变化能及时反映
    async fn update_dns_resolutions(
        target_cache: &Arc<DashMap<String, TargetInfo>>,
        rule_infos: &Arc<RwLock<DashMap<String, RuleInfo>>>,
        config: &Config,
        callback: &Option<TargetSwitchCallback>,
    ) {
        let targets: Vec<_> = target_cache
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();

        let mut dns_tasks = Vec::new();
        let mut any_updated = false;

        // 并发处理每个域名的DNS解析，各自独立，不再有批量触发逻辑
        for (target_str, target_info) in targets {
            // 只处理域名，跳过IP:PORT格式
            if target_str.parse::<std::net::SocketAddr>().is_err() && target_str.contains('.') {
                let target_cache_clone = target_cache.clone();
                let task = tokio::spawn(async move {
                    match resolve_target(&target_str).await {
                        Ok(new_resolved) => {
                            let mut updated_info = target_info.clone();
                            let has_changed = new_resolved != target_info.resolved;

                            if has_changed {
                                info!(
                                    "目标 {} DNS解析变化: {} -> {}",
                                    target_str, target_info.resolved, new_resolved
                                );
                            }

                            updated_info.resolved = new_resolved;
                            updated_info.last_check = Instant::now();
                            // 注意：健康状态将由后续的batch_health_check更新

                            // 验证完成后更新缓存
                            target_cache_clone.insert(target_str.clone(), updated_info);

                            Some(has_changed) // 只有地址变化时才标记为有更新
                        }
                        Err(e) => {
                            // DNS解析失败，单独标记此域名，不触发批量操作
                            let mut failed_info = target_info.clone();
                            failed_info.last_check = Instant::now();
                            failed_info.healthy = false;
                            failed_info.fail_count += 1;

                            warn!("目标 {target_str} DNS解析失败: {e}");
                            target_cache_clone.insert(target_str, failed_info);
                            Some(false)
                        }
                    }
                });
                dns_tasks.push(task);
            }
        }

        // 等待所有DNS解析完成
        for task in dns_tasks {
            if let Ok(Some(updated)) = task.await {
                if updated {
                    any_updated = true;
                }
            }
        }

        // 只有当确实有更新时才更新规则目标选择
        if any_updated {
            // 连接验证已同步完成，只有健康的地址会参与规则选择
            Self::update_rule_targets(rule_infos, target_cache, config, callback).await;
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

            let config_clone = config.clone();
            let task = tokio::spawn(async move {
                let start = Instant::now();

                // 使用配置的超时时间
                let timeout_duration = Duration::from_secs(
                    config_clone
                        .get_dynamic_update_config()
                        .get_connection_timeout(),
                );

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
                    // UDP协议：跳过健康检查，认为DNS解析成功的目标都是健康的
                    // （DNS解析已在update_dns_resolutions中完成）
                    Ok(Duration::from_millis(0))
                } else {
                    // TCP协议：进行连接测试
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
        callback: &Option<TargetSwitchCallback>,
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

            // 检查是否有健康目标，避免重复的无健康目标警告
            let has_healthy_targets = updated_targets.iter().any(|t| t.healthy);
            if !has_healthy_targets && rule_info.selected_target.is_some() {
                let time_since_last_update = rule_info.last_update.elapsed();
                if time_since_last_update > Duration::from_secs(30) {
                    // 每30秒最多记录一次无健康目标警告
                    if let Some(current_target) = &rule_info.selected_target {
                        warn!(
                            "规则 {} 无健康目标，保持当前地址: {}",
                            rule_name, current_target.resolved
                        );
                    }
                }
            }

            // 检查是否需要更新目标 - 增强逻辑，包括健康状态变化检测
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

                        // 立即调用回调通知防火墙更新
                        if let Some(cb) = callback {
                            cb(
                                &rule_name,
                                &old.resolved.to_string(),
                                &new.resolved.to_string(),
                            );
                        }

                        true
                    } else {
                        // 地址相同，检查健康状态是否变化
                        let old_healthy = updated_targets
                            .iter()
                            .find(|t| t.resolved == old.resolved)
                            .map(|t| t.healthy)
                            .unwrap_or(false);

                        if old.healthy != old_healthy {
                            // 避免重复记录相同的健康状态变化（检查时间间隔）
                            let time_since_last_update = rule_info.last_update.elapsed();
                            if time_since_last_update > Duration::from_secs(10) || old_healthy {
                                // 只有间隔超过10秒或状态恢复为健康时才记录日志
                                if old_healthy {
                                    info!("规则 {} 目标 {} 恢复健康", rule_name, old.resolved);
                                } else {
                                    info!(
                                        "规则 {} 目标 {} 健康状态变化: {} -> {}",
                                        rule_name, old.resolved, old.healthy, old_healthy
                                    );
                                }
                            }
                            // 健康状态变化也需要更新
                            true
                        } else {
                            false
                        }
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

// 智能目标选择算法 - 优先级优先策略，确保切换到最高优先级健康地址
fn select_best_target_with_stickiness(
    targets: &[TargetInfo],
    current_target: Option<&TargetInfo>,
) -> Option<TargetInfo> {
    if targets.is_empty() {
        return None;
    }

    // 1. 过滤健康目标
    let healthy_targets: Vec<_> = targets.iter().filter(|t| t.healthy).collect();

    // 2. 检查当前目标是否还健康，如果不健康则强制切换
    if let Some(current) = current_target {
        // 检查当前目标在最新targets中的健康状态
        let current_in_targets = targets.iter().find(|t| t.resolved == current.resolved);
        if let Some(current_updated) = current_in_targets {
            if !current_updated.healthy {
                // 检查是否有其他健康目标可切换
                let has_other_healthy = targets
                    .iter()
                    .any(|t| t.healthy && t.resolved != current.resolved);
                if has_other_healthy {
                    log::warn!(
                        "当前目标 {} 变为不健康，强制切换到备用地址",
                        current.resolved
                    );
                } else {
                    // 没有其他健康目标，只记录一次警告（通过外层时间控制）
                    log::debug!("当前目标 {} 不健康但无备用地址", current.resolved);
                }
            }
        }
    }

    // 3. 优先级优先策略：总是选择优先级最高的健康目标
    if !healthy_targets.is_empty() {
        for target in targets {
            if target.healthy {
                // 找到优先级最高的健康目标
                if let Some(current) = current_target {
                    if target.resolved != current.resolved {
                        log::info!(
                            "切换到最高优先级健康地址: {} 替换 {}",
                            target.resolved,
                            current.resolved
                        );
                        // 这里需要传递rule_name，但在这个函数中没有，需要重构
                    }
                }
                return Some(target.clone());
            }
        }
    }

    // 4. 保守策略：没有健康目标时，如果当前目标存在则保持
    if let Some(current) = current_target {
        // 移除频繁的警告日志，由调用方决定是否记录
        return Some(current.clone());
    }

    // 5. 初始化场景：选择配置中第一个作为起始目标
    if let Some(first) = targets.first() {
        log::debug!("初始化选择: {} (健康:{})", first.resolved, first.healthy);
        return Some(first.clone());
    }

    None
}
