// Firewall4 (nftables) 内核态转发管理器
// 专门解决OpenWrt Firewall4优先级冲突问题

use anyhow::Result;
use async_trait::async_trait;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::process::Command;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::common::CommonManager;
use crate::config::Config;

// ================================
// 防火墙后端枚举
// ================================
#[derive(Debug, Clone, PartialEq)]
pub enum FirewallBackend {
    Nftables,
    Iptables,
    Pfctl, // macOS pfctl防火墙
}

// ================================
// 转发类型
// ================================
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
pub enum ForwardType {
    DNAT,
    SNAT,
}

// ================================
// 防火墙规则结构
// ================================
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FirewallRule {
    pub rule_id: String,
    pub listen_port: u16,
    pub protocol: String,
    pub target_addr: String,
    pub forward_type: ForwardType,
    pub enabled: bool,
    pub priority: i32,
    pub config_index: usize,
}

impl FirewallRule {
    pub fn new(
        rule_id: String,
        listen_port: u16,
        protocol: String,
        target_addr: String,
        forward_type: ForwardType,
        config_index: usize,
    ) -> Self {
        // 根据转发类型设置优先级
        let priority = match forward_type {
            ForwardType::DNAT => -150, // 比Firewall4默认DNAT(-100)更高
            ForwardType::SNAT => 50,   // 比默认SNAT(100)更低但足够
        };

        Self {
            rule_id,
            listen_port,
            protocol,
            target_addr,
            forward_type,
            enabled: true,
            priority,
            config_index,
        }
    }
}

// ================================
// 防火墙管理器特征
// ================================
#[async_trait]
#[allow(dead_code)]
pub trait FirewallManager: Send + Sync {
    async fn initialize(&mut self) -> Result<()>;
    async fn add_forward_rule(&mut self, rule: &FirewallRule) -> Result<()>;
    async fn remove_forward_rule(&mut self, rule_id: &str) -> Result<()>;
    async fn update_forward_rule(&mut self, rule: &FirewallRule) -> Result<()>;
    async fn clear_all_rules(&mut self) -> Result<()>;
    async fn list_rules(&self) -> Result<Vec<FirewallRule>>;
    async fn is_rule_exists(&self, rule_id: &str) -> Result<bool>;
    async fn rebuild_all_rules(&mut self, rules: &[FirewallRule]) -> Result<()>;
}

// ================================
// macOS pfctl 管理器实现
// ================================
#[cfg(target_os = "macos")]
pub struct PfctlManager {
    anchor_name: String,
    rules: HashMap<String, FirewallRule>,
}

#[cfg(target_os = "macos")]
impl PfctlManager {
    pub fn new() -> Self {
        Self {
            anchor_name: "smart_forward".to_string(),
            rules: HashMap::new(),
        }
    }

    async fn execute_pfctl(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("pfctl").args(args).output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("pfctl命令执行失败: {}", stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn generate_nat_rule(&self, rule: &FirewallRule) -> String {
        // 生成pfctl NAT规则
        // 格式: rdr on interface from any to any port listen_port -> target_addr
        let target_parts: Vec<&str> = rule.target_addr.split(':').collect();
        let target_ip = target_parts[0];
        let listen_port_str = rule.listen_port.to_string();
        let target_port = target_parts.get(1).map_or(listen_port_str.as_str(), |v| *v);

        format!(
            "rdr pass on lo0 proto {} from any to any port {} -> {} port {}",
            rule.protocol.to_lowercase(),
            rule.listen_port,
            target_ip,
            target_port
        )
    }
}

#[cfg(target_os = "macos")]
#[async_trait]
impl FirewallManager for PfctlManager {
    async fn initialize(&mut self) -> Result<()> {
        info!("初始化pfctl防火墙管理器");

        // 检查pfctl命令是否可用，并提供详细的权限提示
        match Command::new("pfctl").arg("-s").arg("info").output() {
            Ok(output) => {
                if output.status.success() {
                    info!("✅ pfctl内核级转发已启用，性能模式激活");
                } else {
                    return Err(anyhow::anyhow!(
                        "pfctl命令执行失败，需要管理员权限\n💡 解决方法:\n   1. 使用 sudo 以管理员权限运行: sudo ./smart-forward\n   2. 或使用 --user-mode 启用用户态转发: ./smart-forward --user-mode"
                    ));
                }
            }
            Err(_) => {
                return Err(anyhow::anyhow!(
                    "pfctl命令不可用\n💡 macOS内核级转发需要:\n   1. 以管理员权限运行: sudo ./smart-forward\n   2. 或使用用户态转发: ./smart-forward --user-mode\n   3. 确保系统防火墙功能正常"
                ));
            }
        }

        Ok(())
    }

    async fn add_forward_rule(&mut self, rule: &FirewallRule) -> Result<()> {
        info!(
            "添加pfctl转发规则: {} -> {}",
            rule.listen_port, rule.target_addr
        );

        let nat_rule = self.generate_nat_rule(rule);
        let anchor_rule_file = format!("/tmp/smart_forward_{}.conf", rule.rule_id);

        // 写入规则到临时文件
        std::fs::write(&anchor_rule_file, &nat_rule)?;

        // 加载规则到锚点
        let anchor_path = format!("{}/{}", self.anchor_name, rule.rule_id);
        self.execute_pfctl(&["-a", &anchor_path, "-f", &anchor_rule_file])
            .await?;

        // 清理临时文件
        std::fs::remove_file(&anchor_rule_file).ok();

        // 保存规则到内存
        self.rules.insert(rule.rule_id.clone(), rule.clone());

        debug!("pfctl转发规则添加成功: {}", rule.rule_id);
        Ok(())
    }

    async fn remove_forward_rule(&mut self, rule_id: &str) -> Result<()> {
        info!("删除pfctl转发规则: {}", rule_id);

        let anchor_path = format!("{}/{}", self.anchor_name, rule_id);

        // 清空锚点规则
        self.execute_pfctl(&["-a", &anchor_path, "-F", "nat"])
            .await?;

        // 从内存中移除
        self.rules.remove(rule_id);

        debug!("pfctl转发规则删除成功: {}", rule_id);
        Ok(())
    }

    async fn update_forward_rule(&mut self, rule: &FirewallRule) -> Result<()> {
        info!(
            "更新pfctl转发规则: {} -> {}",
            rule.rule_id, rule.target_addr
        );

        // 先删除旧规则，再添加新规则
        self.remove_forward_rule(&rule.rule_id).await?;
        self.add_forward_rule(rule).await?;

        Ok(())
    }

    async fn clear_all_rules(&mut self) -> Result<()> {
        info!("清理所有pfctl转发规则");

        // 清空整个锚点
        self.execute_pfctl(&["-a", &self.anchor_name, "-F", "all"])
            .await?;

        // 清空内存中的规则
        self.rules.clear();

        Ok(())
    }

    async fn list_rules(&self) -> Result<Vec<FirewallRule>> {
        Ok(self.rules.values().cloned().collect())
    }

    async fn is_rule_exists(&self, rule_id: &str) -> Result<bool> {
        Ok(self.rules.contains_key(rule_id))
    }

    async fn rebuild_all_rules(&mut self, rules: &[FirewallRule]) -> Result<()> {
        // 清空所有规则
        self.clear_all_rules().await?;

        // 重新添加所有规则
        for rule in rules {
            self.add_forward_rule(rule).await?;
        }

        Ok(())
    }
}

// ================================
// nftables 管理器 - 针对Firewall4优化
// ================================
pub struct NftablesManager {
    table_name: String,
    chain_prerouting: String,
    chain_postrouting: String,
    listen_addr: String,
    rules: HashMap<String, FirewallRule>,
}

impl NftablesManager {
    pub fn new(listen_addr: String) -> Self {
        Self {
            table_name: "smart_forward".to_string(),
            chain_prerouting: "prerouting".to_string(),
            chain_postrouting: "postrouting".to_string(),
            listen_addr,
            rules: HashMap::new(),
        }
    }

    async fn execute_nft(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("nft").args(args).output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("nft命令执行失败: {}", stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    async fn table_exists(&self) -> Result<bool> {
        match self
            .execute_nft(&["list", "table", "inet", &self.table_name])
            .await
        {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn create_table_and_chains(&self) -> Result<()> {
        info!("创建nftables表和链，优先级高于Firewall4默认规则");

        // 创建专用table
        self.execute_nft(&["add", "table", "inet", &self.table_name])
            .await?;
        debug!("创建table: {}", self.table_name);

        // 创建prerouting chain - 优先级-150，高于Firewall4默认DNAT(-100)
        self.execute_nft(&[
            "add",
            "chain",
            "inet",
            &self.table_name,
            &self.chain_prerouting,
            "{",
            "type",
            "nat",
            "hook",
            "prerouting",
            "priority",
            "-150",
            ";",
            "}",
        ])
        .await?;
        info!("创建prerouting链，优先级-150（高于Firewall4默认-100）");

        // 创建postrouting chain - 优先级50，低于默认SNAT(100)但足够用
        self.execute_nft(&[
            "add",
            "chain",
            "inet",
            &self.table_name,
            &self.chain_postrouting,
            "{",
            "type",
            "nat",
            "hook",
            "postrouting",
            "priority",
            "50",
            ";",
            "}",
        ])
        .await?;
        info!("创建postrouting链，优先级50");

        Ok(())
    }

    fn detect_ip_version(target_addr: &str) -> &'static str {
        let target_parts: Vec<&str> = target_addr.split(':').collect();
        let target_ip = target_parts[0];

        // IPv6地址检测规则：
        // 1. 包含 :: (IPv6压缩格式)
        // 2. 包含 [ ] (IPv6端口格式)
        // 3. 超过2个冒号 (完整IPv6格式)
        if target_ip.contains("::") || target_ip.contains('[') || target_parts.len() > 2 {
            "ip6"
        } else {
            "ip"
        }
    }

    fn generate_dnat_rule(&self, rule: &FirewallRule) -> Vec<String> {
        let target_parts: Vec<&str> = rule.target_addr.split(':').collect();
        if target_parts.is_empty() {
            // 防止空地址导致panic
            error!("目标地址为空: {}", rule.target_addr);
            return vec![];
        }
        let target_ip = target_parts[0];
        let port_str = rule.listen_port.to_string();
        let port_str_ref = port_str.as_str();
        let target_port = target_parts.get(1).unwrap_or(&port_str_ref);

        // 动态检测IP版本
        let ip_version = Self::detect_ip_version(&rule.target_addr);

        // 处理IPv6地址格式
        let formatted_target = if ip_version == "ip6" {
            if target_ip.starts_with('[') {
                // 已经是 [IPv6]:port 格式
                rule.target_addr.clone()
            } else {
                // 转换为 [IPv6]:port 格式
                format!("[{}]:{}", target_ip, target_port)
            }
        } else {
            // IPv4格式
            format!("{}:{}", target_ip, target_port)
        };

        // DNAT规则生成：只有指定具体监听地址时才添加地址限制
        let mut rule_args = vec![
            "add".to_string(),
            "rule".to_string(),
            "inet".to_string(),
            self.table_name.clone(),
            self.chain_prerouting.clone(),
        ];

        // 只有当监听地址不是0.0.0.0时才添加目标地址限制
        // 0.0.0.0在路由系统中不推荐使用，但保持兼容性
        if self.listen_addr != "0.0.0.0" {
            rule_args.extend(vec![
                "ip".to_string(),
                "daddr".to_string(),
                self.listen_addr.clone(),
            ]);
        }

        rule_args.extend(vec![
            rule.protocol.clone(),
            "dport".to_string(),
            rule.listen_port.to_string(),
            "dnat".to_string(),
            ip_version.to_string(),
            "to".to_string(),
            formatted_target,
        ]);

        rule_args
    }

    fn generate_snat_rule(&self, _rule: &FirewallRule) -> Vec<String> {
        // 对于转发到外网的流量，添加SNAT规则
        vec![
            "add".to_string(),
            "rule".to_string(),
            "inet".to_string(),
            self.table_name.clone(),
            self.chain_postrouting.clone(),
            "oifname".to_string(),
            "!=".to_string(),
            "\"lo\"".to_string(),
            "masquerade".to_string(),
        ]
    }
}

#[async_trait]
impl FirewallManager for NftablesManager {
    async fn initialize(&mut self) -> Result<()> {
        info!("初始化nftables管理器，针对Firewall4优先级优化");

        // 检查nft命令权限（命令不可用时会自动回退到用户态）
        match Command::new("nft").arg("--version").output() {
            Ok(output) => {
                if !output.status.success() {
                    return Err(anyhow::anyhow!(
                        "nft命令权限不足\n💡 解决方法:\n   1. 使用管理员权限运行: sudo ./smart-forward\n   2. 或使用用户态转发: ./smart-forward --user-mode"
                    ));
                }
            }
            Err(_) => {
                // nft命令不可用，回退到用户态转发
                return Err(anyhow::anyhow!("nft命令不可用"));
            }
        }

        // 如果表已存在，先清理
        if self.table_exists().await? {
            warn!("检测到已存在的smart_forward表，正在清理...");
            let _ = self
                .execute_nft(&["delete", "table", "inet", &self.table_name])
                .await;
        }

        // 创建表和链
        self.create_table_and_chains().await?;

        info!("nftables管理器初始化完成，已设置高优先级规则");
        Ok(())
    }

    async fn add_forward_rule(&mut self, rule: &FirewallRule) -> Result<()> {
        debug!("添加转发规则: {} -> {}", rule.listen_port, rule.target_addr);

        // 添加DNAT规则
        if rule.forward_type == ForwardType::DNAT {
            let dnat_strings = self.generate_dnat_rule(rule);
            let dnat_args: Vec<&str> = dnat_strings.iter().map(|s| s.as_str()).collect();
            self.execute_nft(&dnat_args).await?;
            debug!(
                "添加DNAT规则: {}:{} -> {}",
                rule.protocol, rule.listen_port, rule.target_addr
            );
        }

        // 添加SNAT规则（masquerade）
        if rule.forward_type == ForwardType::SNAT {
            let snat_strings = self.generate_snat_rule(rule);
            let snat_args: Vec<&str> = snat_strings.iter().map(|s| s.as_str()).collect();
            self.execute_nft(&snat_args).await?;
            debug!("添加SNAT规则（masquerade）");
        }

        // 保存规则到内存
        self.rules.insert(rule.rule_id.clone(), rule.clone());

        Ok(())
    }

    async fn remove_forward_rule(&mut self, rule_id: &str) -> Result<()> {
        if let Some(_rule) = self.rules.get(rule_id) {
            debug!("删除转发规则: {}", rule_id);

            // 由于nftables的规则删除比较复杂，我们采用重建策略
            self.rules.remove(rule_id);
            let remaining_rules: Vec<FirewallRule> = self.rules.values().cloned().collect();
            self.rebuild_all_rules(&remaining_rules).await?;
        }

        Ok(())
    }

    async fn update_forward_rule(&mut self, rule: &FirewallRule) -> Result<()> {
        debug!("更新转发规则: {} -> {}", rule.listen_port, rule.target_addr);

        // 更新内存中的规则
        self.rules.insert(rule.rule_id.clone(), rule.clone());

        // 重建所有规则以确保一致性
        let all_rules: Vec<FirewallRule> = self.rules.values().cloned().collect();
        self.rebuild_all_rules(&all_rules).await?;

        Ok(())
    }

    async fn clear_all_rules(&mut self) -> Result<()> {
        info!("清理所有smart_forward规则");

        // 删除整个表
        if self.table_exists().await? {
            self.execute_nft(&["delete", "table", "inet", &self.table_name])
                .await?;
            info!("已删除smart_forward表");
        }

        // 清空内存中的规则
        self.rules.clear();

        Ok(())
    }

    async fn list_rules(&self) -> Result<Vec<FirewallRule>> {
        Ok(self.rules.values().cloned().collect())
    }

    async fn is_rule_exists(&self, rule_id: &str) -> Result<bool> {
        Ok(self.rules.contains_key(rule_id))
    }

    async fn rebuild_all_rules(&mut self, rules: &[FirewallRule]) -> Result<()> {
        debug!("重建所有nftables规则，共{}条", rules.len());

        // 清空现有规则（保留表和链结构）
        if self.table_exists().await? {
            self.execute_nft(&["flush", "table", "inet", &self.table_name])
                .await?;
        } else {
            // 如果表不存在，重新创建
            self.create_table_and_chains().await?;
        }

        // 重新添加所有规则
        for rule in rules {
            if rule.enabled {
                // 直接添加规则，不更新内存（避免递归调用）
                if rule.forward_type == ForwardType::DNAT {
                    let dnat_strings = self.generate_dnat_rule(rule);
                    let dnat_args: Vec<&str> = dnat_strings.iter().map(|s| s.as_str()).collect();
                    self.execute_nft(&dnat_args).await?;
                }
                if rule.forward_type == ForwardType::SNAT {
                    let snat_strings = self.generate_snat_rule(rule);
                    let snat_args: Vec<&str> = snat_strings.iter().map(|s| s.as_str()).collect();
                    self.execute_nft(&snat_args).await?;
                }
            }
        }

        debug!("规则重建完成");
        Ok(())
    }
}

// ================================
// iptables 管理器 - 兼容传统OpenWrt
// ================================
pub struct IptablesManager {
    chain_prerouting: String,
    chain_postrouting: String,
    listen_addr: String,
    rules: HashMap<String, FirewallRule>,
}

impl IptablesManager {
    pub fn new(listen_addr: String) -> Self {
        Self {
            chain_prerouting: "SMART_FORWARD_PREROUTING".to_string(),
            chain_postrouting: "SMART_FORWARD_POSTROUTING".to_string(),
            listen_addr,
            rules: HashMap::new(),
        }
    }

    async fn execute_iptables(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("iptables").args(args).output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("iptables命令执行失败: {}", stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    async fn chain_exists(&self, table: &str, chain: &str) -> Result<bool> {
        match self.execute_iptables(&["-t", table, "-L", chain]).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn create_chains(&self) -> Result<()> {
        info!("创建iptables链，优先级高于默认规则");

        // 创建PREROUTING链
        if !self.chain_exists("nat", &self.chain_prerouting).await? {
            self.execute_iptables(&["-t", "nat", "-N", &self.chain_prerouting])
                .await?;
            debug!("创建链: {}", self.chain_prerouting);
        }

        // 创建POSTROUTING链
        if !self.chain_exists("nat", &self.chain_postrouting).await? {
            self.execute_iptables(&["-t", "nat", "-N", &self.chain_postrouting])
                .await?;
            debug!("创建链: {}", self.chain_postrouting);
        }

        // 将自定义链插入到主链的开头（高优先级）
        // 检查是否已经插入，避免重复
        let prerouting_check = self
            .execute_iptables(&["-t", "nat", "-L", "PREROUTING", "--line-numbers"])
            .await?;
        if !prerouting_check.contains(&self.chain_prerouting) {
            self.execute_iptables(&[
                "-t",
                "nat",
                "-I",
                "PREROUTING",
                "1",
                "-j",
                &self.chain_prerouting,
            ])
            .await?;
            info!("插入PREROUTING链到位置1（最高优先级）");
        }

        let postrouting_check = self
            .execute_iptables(&["-t", "nat", "-L", "POSTROUTING", "--line-numbers"])
            .await?;
        if !postrouting_check.contains(&self.chain_postrouting) {
            self.execute_iptables(&[
                "-t",
                "nat",
                "-I",
                "POSTROUTING",
                "1",
                "-j",
                &self.chain_postrouting,
            ])
            .await?;
            info!("插入POSTROUTING链到位置1");
        }

        Ok(())
    }

    fn generate_dnat_args(&self, rule: &FirewallRule) -> Vec<String> {
        let target_parts: Vec<&str> = rule.target_addr.split(':').collect();
        if target_parts.is_empty() {
            // 防止空地址导致panic
            error!("目标地址为空: {}", rule.target_addr);
            return vec![];
        }
        let target_ip = target_parts[0];
        let port_str = rule.listen_port.to_string();
        let port_str_ref = port_str.as_str();
        let target_port = target_parts.get(1).unwrap_or(&port_str_ref);

        // DNAT规则生成：只有指定具体监听地址时才添加地址限制
        let mut rule_args = vec![
            "-t".to_string(),
            "nat".to_string(),
            "-A".to_string(),
            self.chain_prerouting.clone(),
        ];

        // 只有当监听地址不是0.0.0.0时才添加目标地址限制
        // 0.0.0.0在路由系统中不推荐使用，但保持兼容性
        if self.listen_addr != "0.0.0.0" {
            rule_args.extend(vec!["-d".to_string(), self.listen_addr.clone()]);
        }

        rule_args.extend(vec![
            "-p".to_string(),
            rule.protocol.clone(),
            "--dport".to_string(),
            rule.listen_port.to_string(),
            "-j".to_string(),
            "DNAT".to_string(),
            "--to-destination".to_string(),
            format!("{}:{}", target_ip, target_port),
        ]);

        rule_args
    }

    fn generate_snat_args(&self) -> Vec<String> {
        vec![
            "-t".to_string(),
            "nat".to_string(),
            "-A".to_string(),
            self.chain_postrouting.clone(),
            "!".to_string(),
            "-o".to_string(),
            "lo".to_string(),
            "-j".to_string(),
            "MASQUERADE".to_string(),
        ]
    }
}

#[async_trait]
impl FirewallManager for IptablesManager {
    async fn initialize(&mut self) -> Result<()> {
        info!("初始化iptables管理器，针对传统OpenWrt优化");

        // 检查iptables命令权限（命令不可用时会自动回退到用户态）
        match Command::new("iptables").arg("--version").output() {
            Ok(output) => {
                if !output.status.success() {
                    return Err(anyhow::anyhow!(
                        "iptables命令权限不足\n💡 解决方法:\n   1. 使用管理员权限运行: sudo ./smart-forward\n   2. 或使用用户态转发: ./smart-forward --user-mode"
                    ));
                }
            }
            Err(_) => {
                // iptables命令不可用，回退到用户态转发
                return Err(anyhow::anyhow!("iptables命令不可用"));
            }
        }

        // 创建链
        self.create_chains().await?;

        info!("iptables管理器初始化完成，已设置高优先级规则");
        Ok(())
    }

    async fn add_forward_rule(&mut self, rule: &FirewallRule) -> Result<()> {
        debug!(
            "添加iptables转发规则: {} -> {}",
            rule.listen_port, rule.target_addr
        );

        // 添加DNAT规则
        if rule.forward_type == ForwardType::DNAT {
            let dnat_strings = self.generate_dnat_args(rule);
            let dnat_args: Vec<&str> = dnat_strings.iter().map(|s| s.as_str()).collect();
            self.execute_iptables(&dnat_args).await?;
            debug!(
                "添加DNAT规则: {}:{} -> {}",
                rule.protocol, rule.listen_port, rule.target_addr
            );
        }

        // 添加SNAT规则（masquerade）
        if rule.forward_type == ForwardType::SNAT {
            let snat_strings = self.generate_snat_args();
            let snat_args: Vec<&str> = snat_strings.iter().map(|s| s.as_str()).collect();
            self.execute_iptables(&snat_args).await?;
            debug!("添加SNAT规则（masquerade）");
        }

        // 保存规则到内存
        self.rules.insert(rule.rule_id.clone(), rule.clone());

        Ok(())
    }

    async fn remove_forward_rule(&mut self, rule_id: &str) -> Result<()> {
        if let Some(_rule) = self.rules.get(rule_id) {
            debug!("删除iptables转发规则: {}", rule_id);

            // iptables规则删除比较复杂，采用重建策略
            self.rules.remove(rule_id);
            let remaining_rules: Vec<FirewallRule> = self.rules.values().cloned().collect();
            self.rebuild_all_rules(&remaining_rules).await?;
        }

        Ok(())
    }

    async fn update_forward_rule(&mut self, rule: &FirewallRule) -> Result<()> {
        debug!(
            "更新iptables转发规则: {} -> {}",
            rule.listen_port, rule.target_addr
        );

        // 更新内存中的规则
        self.rules.insert(rule.rule_id.clone(), rule.clone());

        // 重建所有规则以确保一致性
        let all_rules: Vec<FirewallRule> = self.rules.values().cloned().collect();
        self.rebuild_all_rules(&all_rules).await?;

        Ok(())
    }

    async fn clear_all_rules(&mut self) -> Result<()> {
        info!("清理所有iptables规则");

        // 清空自定义链
        let _ = self
            .execute_iptables(&["-t", "nat", "-F", &self.chain_prerouting])
            .await;
        let _ = self
            .execute_iptables(&["-t", "nat", "-F", &self.chain_postrouting])
            .await;

        // 从主链中移除跳转规则
        let _ = self
            .execute_iptables(&[
                "-t",
                "nat",
                "-D",
                "PREROUTING",
                "-j",
                &self.chain_prerouting,
            ])
            .await;
        let _ = self
            .execute_iptables(&[
                "-t",
                "nat",
                "-D",
                "POSTROUTING",
                "-j",
                &self.chain_postrouting,
            ])
            .await;

        // 删除自定义链
        let _ = self
            .execute_iptables(&["-t", "nat", "-X", &self.chain_prerouting])
            .await;
        let _ = self
            .execute_iptables(&["-t", "nat", "-X", &self.chain_postrouting])
            .await;

        // 清空内存中的规则
        self.rules.clear();

        info!("已清理所有iptables规则");
        Ok(())
    }

    async fn list_rules(&self) -> Result<Vec<FirewallRule>> {
        Ok(self.rules.values().cloned().collect())
    }

    async fn is_rule_exists(&self, rule_id: &str) -> Result<bool> {
        Ok(self.rules.contains_key(rule_id))
    }

    async fn rebuild_all_rules(&mut self, rules: &[FirewallRule]) -> Result<()> {
        debug!("重建所有iptables规则，共{}条", rules.len());

        // 清空现有规则（保留链结构）
        let _ = self
            .execute_iptables(&["-t", "nat", "-F", &self.chain_prerouting])
            .await;
        let _ = self
            .execute_iptables(&["-t", "nat", "-F", &self.chain_postrouting])
            .await;

        // 重新添加所有规则
        for rule in rules {
            if rule.enabled {
                // 直接添加规则，不更新内存（避免递归调用）
                if rule.forward_type == ForwardType::DNAT {
                    let dnat_strings = self.generate_dnat_args(rule);
                    let dnat_args: Vec<&str> = dnat_strings.iter().map(|s| s.as_str()).collect();
                    let _ = self.execute_iptables(&dnat_args).await;
                }
                if rule.forward_type == ForwardType::SNAT {
                    let snat_strings = self.generate_snat_args();
                    let snat_args: Vec<&str> = snat_strings.iter().map(|s| s.as_str()).collect();
                    let _ = self.execute_iptables(&snat_args).await;
                }
            }
        }

        debug!("iptables规则重建完成");
        Ok(())
    }
}

// ================================
// 防火墙调度器 - 集成用户态健康检查
// ================================
pub struct FirewallScheduler {
    manager: Box<dyn FirewallManager>,
    config: Config,
    common_manager: CommonManager,
    rules: Arc<RwLock<HashMap<String, FirewallRule>>>,
}

#[allow(dead_code)]
impl FirewallScheduler {
    pub async fn new(
        backend: FirewallBackend,
        config: Config,
        common_manager: CommonManager,
    ) -> Result<Self> {
        let listen_addr = config.network.first();
        let manager: Box<dyn FirewallManager> = match backend {
            FirewallBackend::Nftables => Box::new(NftablesManager::new(listen_addr)),
            FirewallBackend::Iptables => Box::new(IptablesManager::new(listen_addr)),
            #[cfg(target_os = "macos")]
            FirewallBackend::Pfctl => Box::new(PfctlManager::new()),
            #[cfg(not(target_os = "macos"))]
            FirewallBackend::Pfctl => {
                return Err(anyhow::anyhow!("pfctl防火墙后端只在macOS上支持"));
            }
        };

        Ok(Self {
            manager,
            config,
            common_manager,
            rules: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn initialize(&mut self) -> Result<()> {
        info!("初始化防火墙调度器");

        // 初始化防火墙管理器
        self.manager.initialize().await?;

        // 创建初始规则
        self.create_initial_rules().await?;

        info!("防火墙调度器初始化完成");
        Ok(())
    }

    async fn create_initial_rules(&mut self) -> Result<()> {
        info!("创建初始防火墙规则");

        for (index, rule_config) in self.config.rules.iter().enumerate() {
            // 获取最佳目标
            if let Ok(best_target) = self.common_manager.get_best_target(&rule_config.name).await {
                let target_addr = best_target.to_string();

                // 为每个协议创建规则
                let protocols = rule_config.get_protocols();
                for protocol in protocols {
                    let rule_id = format!("{}_{}", rule_config.name, protocol);

                    // 创建DNAT规则
                    let dnat_rule = FirewallRule::new(
                        format!("{}_dnat", rule_id),
                        rule_config.listen_port,
                        protocol.clone(),
                        target_addr.clone(),
                        ForwardType::DNAT,
                        index,
                    );

                    // 创建SNAT规则
                    let snat_rule = FirewallRule::new(
                        format!("{}_snat", rule_id),
                        rule_config.listen_port,
                        protocol.clone(),
                        target_addr.clone(),
                        ForwardType::SNAT,
                        index,
                    );

                    // 添加规则
                    self.manager.add_forward_rule(&dnat_rule).await?;
                    self.manager.add_forward_rule(&snat_rule).await?;

                    // 保存到内存
                    let mut rules = self.rules.write().await;
                    rules.insert(dnat_rule.rule_id.clone(), dnat_rule);
                    rules.insert(snat_rule.rule_id.clone(), snat_rule);

                    info!(
                        "创建规则: {} {} -> {}",
                        rule_config.name, protocol, target_addr
                    );
                }
            } else {
                warn!("规则 {} 没有可用的目标地址", rule_config.name);
            }
        }

        Ok(())
    }

    pub async fn sync_with_targets(&mut self) -> Result<()> {
        debug!("同步防火墙规则与健康检查结果");

        // 收集需要更新的规则
        let mut rules_to_update = Vec::new();

        for rule_config in &self.config.rules {
            if let Ok(best_target) = self.common_manager.get_best_target(&rule_config.name).await {
                let target_addr = best_target.to_string();

                // 检查是否需要更新规则
                let protocols = rule_config.get_protocols();
                for protocol in protocols {
                    let dnat_rule_id = format!("{}_{}_dnat", rule_config.name, protocol);
                    let snat_rule_id = format!("{}_{}_snat", rule_config.name, protocol);

                    let existing_dnat_rule = {
                        let rules = self.rules.read().await;
                        rules.get(&dnat_rule_id).cloned()
                    };

                    if let Some(existing_rule) = existing_dnat_rule {
                        if existing_rule.target_addr != target_addr {
                            // 需要更新规则
                            info!(
                                "🔄 内核态转发规则更新: {} {} {} -> {}",
                                rule_config.name, protocol, existing_rule.target_addr, target_addr
                            );

                            // 创建更新后的DNAT和SNAT规则
                            let mut updated_dnat = existing_rule.clone();
                            updated_dnat.target_addr = target_addr.clone();

                            let existing_snat_rule = {
                                let rules = self.rules.read().await;
                                rules.get(&snat_rule_id).cloned()
                            };

                            if let Some(mut updated_snat) = existing_snat_rule {
                                updated_snat.target_addr = target_addr.clone();
                                rules_to_update.push((updated_dnat, updated_snat));
                            }
                        }
                    }
                }
            }
        }

        // 执行规则更新
        for (dnat_rule, snat_rule) in rules_to_update {
            // 更新DNAT规则
            if let Err(e) = self.manager.update_forward_rule(&dnat_rule).await {
                error!("更新DNAT规则失败: {} - {}", dnat_rule.rule_id, e);
                continue;
            }

            // 更新SNAT规则
            if let Err(e) = self.manager.update_forward_rule(&snat_rule).await {
                error!("更新SNAT规则失败: {} - {}", snat_rule.rule_id, e);
                continue;
            }

            // 更新内存中的规则
            {
                let mut rules = self.rules.write().await;
                rules.insert(dnat_rule.rule_id.clone(), dnat_rule);
                rules.insert(snat_rule.rule_id.clone(), snat_rule);
            }

            debug!("✅ 内核态转发规则更新完成");
        }

        Ok(())
    }

    pub async fn clear_all(&mut self) -> Result<()> {
        info!("清理所有防火墙规则");
        self.manager.clear_all_rules().await?;
        self.rules.write().await.clear();
        Ok(())
    }
}

// ================================
// 防火墙后端检测
// ================================
pub fn detect_firewall_backend() -> FirewallBackend {
    // Windows环境不需要防火墙后端检测
    if cfg!(target_os = "windows") {
        return FirewallBackend::Nftables; // 返回默认值，但不会实际使用
    }

    // macOS环境检测pfctl
    if cfg!(target_os = "macos") {
        // 检查pfctl命令是否可用
        if Command::new("pfctl").arg("-s").arg("info").output().is_ok() {
            info!("检测到macOS pfctl支持");
            return FirewallBackend::Pfctl;
        } else {
            // pfctl不可用时，不提示警告，让后续初始化时提供更详细的错误信息
            return FirewallBackend::Pfctl;
        }
    }

    // Linux环境的防火墙检测
    // 检查nft命令
    if Command::new("nft").arg("--version").output().is_ok() {
        info!("检测到nftables支持");
        return FirewallBackend::Nftables;
    }

    // 检查iptables命令
    if Command::new("iptables").arg("--version").output().is_ok() {
        info!("检测到iptables支持");
        return FirewallBackend::Iptables;
    }

    warn!("未检测到支持的防火墙后端，默认使用nftables");
    FirewallBackend::Nftables
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ip_version_detection() {
        // IPv4测试
        assert_eq!(NftablesManager::detect_ip_version("192.168.1.1:80"), "ip");
        assert_eq!(
            NftablesManager::detect_ip_version("121.40.167.222:443"),
            "ip"
        );
        assert_eq!(NftablesManager::detect_ip_version("10.0.0.1"), "ip");

        // IPv6测试
        assert_eq!(NftablesManager::detect_ip_version("2001:db8::1:80"), "ip6");
        assert_eq!(
            NftablesManager::detect_ip_version("[2001:db8::1]:80"),
            "ip6"
        );
        assert_eq!(NftablesManager::detect_ip_version("fe80::1"), "ip6");
        assert_eq!(
            NftablesManager::detect_ip_version("2001:0db8:85a3:0000:0000:8a2e:0370:7334:443"),
            "ip6"
        );

        // 边界情况
        assert_eq!(NftablesManager::detect_ip_version("::1"), "ip6");
        assert_eq!(NftablesManager::detect_ip_version("::"), "ip6");
    }
}
