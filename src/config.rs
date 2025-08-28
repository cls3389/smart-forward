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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub listen_addr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardRule {
    pub name: String,
    pub listen_port: u16,
    pub protocol: Option<String>,  // 保持向后兼容
    pub protocols: Option<Vec<String>>,  // 新增：支持多协议
    pub buffer_size: Option<usize>,
    pub targets: Vec<String>,
    pub dynamic_update: Option<DynamicUpdateConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicUpdateConfig {
    pub check_interval: Option<u64>,
    pub connection_timeout: Option<u64>,
    pub auto_reconnect: Option<bool>,
    // 移除 health_check_interval，使用统一的 check_interval
}

impl Config {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let mut config: Config = serde_yaml::from_str(&content)?;
        
        // 设置默认值
        if config.buffer_size.is_none() {
            config.buffer_size = Some(16384);
        }
        
        if config.network.listen_addr.is_empty() {
            config.network.listen_addr = "0.0.0.0".to_string();
        }
        
        // 设置动态更新默认值（优化的内置参数）
        if config.dynamic_update.is_none() {
            config.dynamic_update = Some(DynamicUpdateConfig {
                check_interval: Some(45),    // 统一的检查间隔，同时用于地址更新和健康检查
                connection_timeout: Some(300), // 5分钟连接超时
                auto_reconnect: Some(true),   // 默认开启自动重连
            });
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
        self.dynamic_update.clone().unwrap_or_else(|| DynamicUpdateConfig {
            check_interval: Some(45),    // 统一的检查间隔
            connection_timeout: Some(300), // 5分钟连接超时
            auto_reconnect: Some(true),   // 默认开启自动重连
        })
    }
}

impl DynamicUpdateConfig {
    pub fn get_check_interval(&self) -> u64 {
        self.check_interval.unwrap_or(45)  // 统一的检查间隔，用于所有检查任务
    }
    
    pub fn get_connection_timeout(&self) -> u64 {
        self.connection_timeout.unwrap_or(300)
    }
    
    pub fn get_auto_reconnect(&self) -> bool {
        self.auto_reconnect.unwrap_or(true)
    }
}

impl ForwardRule {
    pub fn get_effective_buffer_size(&self, default_size: usize) -> usize {
        self.buffer_size.unwrap_or(default_size)
    }
    
    pub fn is_protocol_supported(&self, protocol: &str) -> bool {
        matches!(protocol, "tcp" | "http" | "udp")
    }
    
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
    pub fn get_dynamic_update_config(&self, global_config: &DynamicUpdateConfig) -> DynamicUpdateConfig {
        if let Some(rule_config) = &self.dynamic_update {
            DynamicUpdateConfig {
                check_interval: rule_config.check_interval.or(global_config.check_interval),
                connection_timeout: rule_config.connection_timeout.or(global_config.connection_timeout),
                auto_reconnect: rule_config.auto_reconnect.or(global_config.auto_reconnect),
            }
        } else {
            global_config.clone()
        }
    }
}
