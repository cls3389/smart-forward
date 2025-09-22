use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub logging: LoggingConfig,
    pub network: NetworkConfig,
    pub buffer_size: Option<usize>,
    pub rules: Vec<ForwardRule>,
    pub dynamic_update: Option<DynamicUpdateConfig>,
    pub dns: Option<DnsConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub listen_addrs: Vec<String>,
}

impl NetworkConfig {
    pub fn contains_wildcard(&self) -> bool {
        self.listen_addrs.iter().any(|addr| addr == "0.0.0.0")
    }

    pub fn first(&self) -> String {
        self.listen_addrs
            .first()
            .unwrap_or(&"0.0.0.0".to_string())
            .clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardRule {
    pub name: String,
    pub listen_port: u16,
    pub protocol: Option<String>,       // 保持向后兼容
    pub protocols: Option<Vec<String>>, // 新增：支持多协议
    pub buffer_size: Option<usize>,
    pub targets: Vec<String>,
    pub dynamic_update: Option<DynamicUpdateConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicUpdateConfig {
    pub check_interval: Option<u64>,
    pub connection_timeout: Option<u64>,
    // 移除 health_check_interval，使用统一的 check_interval
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsConfig {
    pub servers: Vec<String>,
    pub timeout: Option<u64>,    // DNS查询超时秒数，默认2秒
    pub attempts: Option<usize>, // DNS查询重试次数，默认2次
}

impl Config {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let mut config: Config = serde_yml::from_str(&content)?;

        // 设置默认值
        if config.buffer_size.is_none() {
            config.buffer_size = Some(16384);
        }

        // 处理空的监听地址配置
        if config.network.listen_addrs.is_empty() {
            config.network.listen_addrs = vec!["0.0.0.0".to_string()];
        }

        // 设置动态更新默认值（优化的内置参数）
        if config.dynamic_update.is_none() {
            config.dynamic_update = Some(DynamicUpdateConfig {
                check_interval: Some(5),     // 5秒健康检查间隔，快速故障检测
                connection_timeout: Some(3), // 3秒连接超时，快速故障检测
            });
        }

        // 0.0.0.0监听地址处理：建议手动配置或回退用户态
        if config.network.contains_wildcard() {
            log::warn!("⚠️  监听地址包含0.0.0.0");
            log::warn!("⚠️  内核态转发时可能会劫持所有端口流量");
            log::warn!("⚠️  建议：1) 手动指定LAN地址  2) 使用用户态转发");
        }

        // 验证配置
        config.validate()?;

        Ok(config)
    }

    pub fn validate(&self) -> Result<()> {
        if self.rules.is_empty() {
            anyhow::bail!("至少需要配置一个转发规则");
        }

        for (i, rule) in self.rules.iter().enumerate() {
            if rule.name.is_empty() {
                anyhow::bail!("规则 {}: 名称不能为空", i + 1);
            }

            if rule.listen_port == 0 {
                anyhow::bail!("规则 {}: 端口号不能为0", rule.name);
            }

            if rule.targets.is_empty() {
                anyhow::bail!("规则 {}: 至少需要一个目标", rule.name);
            }

            // 验证协议
            if let Some(protocol) = &rule.protocol {
                if !rule.is_protocol_supported(protocol) {
                    anyhow::bail!("规则 {}: 不支持的协议 {}", rule.name, protocol);
                }
            }

            // 验证多协议
            if let Some(protocols) = &rule.protocols {
                for protocol in protocols {
                    if !rule.is_protocol_supported(protocol) {
                        anyhow::bail!("规则 {}: 不支持的协议 {}", rule.name, protocol);
                    }
                }
            }
        }

        Ok(())
    }

    // 获取动态更新配置（优化的内置默认值）
    pub fn get_dynamic_update_config(&self) -> DynamicUpdateConfig {
        self.dynamic_update.clone().unwrap_or(DynamicUpdateConfig {
            check_interval: Some(5),     // 5秒健康检查间隔，快速故障检测
            connection_timeout: Some(2), // 2秒连接超时，快速故障检测
        })
    }

    // 获取DNS配置（默认只使用阿里云DNS）
    pub fn get_dns_config(&self) -> DnsConfig {
        self.dns.clone().unwrap_or(DnsConfig {
            servers: vec![
                "223.5.5.5:53".to_string(), // 阿里云DNS
                "223.6.6.6:53".to_string(), // 阿里云DNS备用
            ],
            timeout: Some(2),  // 2秒超时
            attempts: Some(2), // 重试2次
        })
    }
}

impl DynamicUpdateConfig {
    pub fn get_check_interval(&self) -> u64 {
        self.check_interval.unwrap_or(5) // 5秒快速故障检测
    }

    pub fn get_connection_timeout(&self) -> u64 {
        self.connection_timeout.unwrap_or(2) // 2秒快速故障检测
    }
}

impl ForwardRule {
    pub fn get_effective_buffer_size(&self, default_size: usize) -> usize {
        self.buffer_size.unwrap_or(default_size)
    }

    pub fn is_protocol_supported(&self, protocol: &str) -> bool {
        matches!(protocol, "tcp" | "http" | "udp")
    }

    #[allow(dead_code)]
    pub fn get_protocol(&self) -> String {
        self.protocol.clone().unwrap_or_else(|| "tcp".to_string())
    }

    // 获取所有支持的协议列表
    pub fn get_protocols(&self) -> Vec<String> {
        if let Some(protocols) = &self.protocols {
            // 如果明确指定了protocols，使用指定的协议列表
            protocols.clone()
        } else if let Some(protocol) = &self.protocol {
            // 如果指定了单个protocol，只使用该协议
            vec![protocol.clone()]
        } else {
            // 默认同时支持TCP和UDP（最常见的使用场景）
            vec!["tcp".to_string(), "udp".to_string()]
        }
    }

    pub fn get_listen_addr(&self, base_addr: &str) -> String {
        format!("{}:{}", base_addr, self.listen_port)
    }

    // 获取规则级别的动态更新配置
    pub fn get_dynamic_update_config(
        &self,
        global_config: &DynamicUpdateConfig,
    ) -> DynamicUpdateConfig {
        if let Some(rule_config) = &self.dynamic_update {
            DynamicUpdateConfig {
                check_interval: rule_config.check_interval.or(global_config.check_interval),
                connection_timeout: rule_config
                    .connection_timeout
                    .or(global_config.connection_timeout),
            }
        } else {
            global_config.clone()
        }
    }
}
