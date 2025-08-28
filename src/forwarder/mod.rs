mod tcp;
mod http;
mod udp;
mod unified;

pub use tcp::TCPForwarder;
pub use http::HTTPForwarder;
pub use udp::UDPForwarder;
pub use unified::UnifiedForwarder;

use crate::config::Config;
use crate::common::CommonManager;
use anyhow::Result;
use async_trait::async_trait;
use log::{error, info, warn};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::any::Any;

#[async_trait]
pub trait Forwarder: Send + Sync {
    async fn start(&mut self) -> Result<()>;
    async fn stop(&mut self);
    #[allow(dead_code)]
    fn is_running(&self) -> bool;
    fn get_stats(&self) -> HashMap<String, String>;
    
    // 新增：支持类型转换
    #[allow(dead_code)]
    fn as_any(&self) -> &dyn Any;
    
    // 新增：支持可变类型转换
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct SmartForwarder {
    config: Config,
    common_manager: CommonManager,
    forwarders: Arc<RwLock<HashMap<String, Box<dyn Forwarder + Send + Sync>>>>,
    // 新增：防止重复启动动态更新任务
    dynamic_update_started: Arc<RwLock<bool>>,
}

impl SmartForwarder {
    pub fn new(config: Config, common_manager: CommonManager) -> Self {
        Self {
            config,
            common_manager,
            forwarders: Arc::new(RwLock::new(HashMap::new())),
            dynamic_update_started: Arc::new(RwLock::new(false)),
        }
    }
    
    pub async fn initialize(&mut self) -> Result<()> {
        info!("初始化智能转发器...");
        
        // 检查是否需要自动启动HTTP跳转
        let has_https = self.config.rules.iter().any(|r| r.listen_port == 443);
        let has_http = self.config.rules.iter().any(|r| r.listen_port == 80);
        
        // 如果有443规则但没有80规则，自动添加HTTP跳转规则
        if has_https && !has_http {
            info!("检测到HTTPS规则但无HTTP规则，将自动启动HTTP跳转服务");
        }
        
        info!("智能转发器初始化完成");
        Ok(())
    }
    
    pub async fn start(&mut self) -> Result<()> {
        info!("启动智能转发器...");
        
        // 启动所有转发器
        let rules = self.config.rules.clone();
        for rule in rules {
            if let Err(e) = self.start_forwarder(&rule).await {
                error!("启动转发器失败 {}: {}", rule.name, e);
                continue;
            }
        }
        
        // 如果有443规则但没有80规则，自动启动HTTP跳转
        let has_https = self.config.rules.iter().any(|r| r.listen_port == 443);
        let has_http = self.config.rules.iter().any(|r| r.listen_port == 80);
        
        if has_https && !has_http {
            if let Err(e) = self.start_auto_http_redirect().await {
                error!("启动自动HTTP跳转失败: {}", e);
            }
        }
        
        // 新增：启动动态地址更新任务（只启动一次）
        // 重新启用：确保转发器能从公共管理器同步最新目标地址
        if !*self.dynamic_update_started.read().await {
            self.start_dynamic_address_update_task().await;
            *self.dynamic_update_started.write().await = true;
        }
        
        info!("智能转发器启动完成");
        Ok(())
    }
    
    // 新增：启动动态地址更新任务
    async fn start_dynamic_address_update_task(&self) {
        let forwarders = self.forwarders.clone();
        let common_manager = self.common_manager.clone();
        let config = self.config.clone();
        
        tokio::spawn(async move {
            // 获取全局动态更新配置
            let global_dynamic_config = config.get_dynamic_update_config();
            let check_interval = global_dynamic_config.get_check_interval();
            
            // 使用可变间隔，启动后的前几次检查使用更短的间隔
            let mut check_count = 0;
            let mut current_interval = if check_interval > 5 { 5 } else { check_interval }; // 启动后前几次使用5秒或更短的间隔
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(current_interval));
            
            info!("启动动态地址更新任务，初始检查间隔: {}秒", current_interval);
            
            loop {
                interval.tick().await;
                check_count += 1;
                
                // 每5次检查后恢复到正常间隔
                if check_count == 5 && current_interval != check_interval {
                    current_interval = check_interval;
                    interval = tokio::time::interval(tokio::time::Duration::from_secs(current_interval));
                    info!("动态地址更新任务恢复到正常检查间隔: {}秒", current_interval);
                }
                
                // 获取所有转发器
                let mut forwarders_write = forwarders.write().await;
                
                for (rule_name, forwarder) in forwarders_write.iter_mut() {
                    // 获取规则特定的动态更新配置
                    let rule_config = if let Some(rule) = config.rules.iter().find(|r| r.name == *rule_name) {
                        rule.get_dynamic_update_config(&global_dynamic_config)
                    } else {
                        global_dynamic_config.clone()
                    };
                    
                    // 智能转发是系统的核心功能，默认开启
                    // 无需检查配置，直接执行
                    
                    // 检查是否需要更新目标地址
                    if let Some(unified_forwarder) = forwarder.as_any_mut().downcast_mut::<crate::forwarder::unified::UnifiedForwarder>() {
                        // 使用规则特定的检查间隔（但受当前间隔影响）
                        let rule_check_interval = if check_count <= 5 {
                            // 启动后的前5次检查使用较短间隔
                            let rule_interval = rule_config.get_check_interval();
                            if rule_interval > 5 { 5 } else { rule_interval }
                        } else {
                            rule_config.get_check_interval()
                        };
                        
                        if unified_forwarder.should_update_target_with_interval(rule_check_interval).await {
                            // 从公共管理器获取最新的最佳目标地址
                            match common_manager.get_best_target_string(rule_name).await {
                                Ok(new_target) => {
                                    let current_target = unified_forwarder.get_current_target();
                                    if new_target != current_target {
                                        info!("规则 {} 检测到地址变化: {} -> {}", 
                                            rule_name, current_target, new_target);
                                        
                                        // 更新转发器目标地址
                                        if let Err(e) = unified_forwarder.update_target(&new_target).await {
                                            error!("更新规则 {} 目标地址失败: {}", rule_name, e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    warn!("获取规则 {} 最新目标地址失败: {}", rule_name, e);
                                }
                            }
                        }
                    }
                }
            }
        });
    }
    
    async fn start_forwarder(&mut self, rule: &crate::config::ForwardRule) -> Result<()> {
        let listen_addr = rule.get_listen_addr(&self.config.network.listen_addr);
        
        // 从公共管理器获取已选择的最佳目标地址
        let best_target = self.common_manager.get_best_target_string(&rule.name).await?;
        
        // 获取规则支持的所有协议
        let protocols = rule.get_protocols();
        
        if protocols.len() == 1 {
            // 单协议转发
            let mut unified_forwarder = UnifiedForwarder::new_with_target(rule, &listen_addr, &best_target);
            unified_forwarder.start().await?;
            self.forwarders.write().await.insert(rule.name.clone(), Box::new(unified_forwarder));
        } else {
            // 多协议同时转发 - 修复：创建一个支持多协议的转发器实例
            let mut unified_forwarder = UnifiedForwarder::new_with_target(rule, &listen_addr, &best_target);
            
            // 设置支持多协议
            unified_forwarder.set_multi_protocol(&protocols).await?;
            
            // 启动转发器（内部会处理多协议绑定）
            unified_forwarder.start().await?;
            
            // 只保存一个转发器实例
            self.forwarders.write().await.insert(rule.name.clone(), Box::new(unified_forwarder));
            
            info!("规则 {} 启动多协议转发: {:?}", rule.name, protocols);
        }
        
        Ok(())
    }
    
    async fn start_auto_http_redirect(&mut self) -> Result<()> {
        let listen_addr = format!("{}:80", self.config.network.listen_addr);
        
        // 创建一个虚拟的HTTP规则
        let http_rule = crate::config::ForwardRule {
            name: "AutoHTTP".to_string(),
            listen_port: 80,
            protocol: Some("http".to_string()),
            protocols: None,  // 单协议，不需要多协议支持
            buffer_size: self.config.buffer_size,
            targets: vec!["127.0.0.1:443".to_string()], // 虚拟目标
            dynamic_update: None,
        };
        
        // 创建HTTP转发器
        let mut http_forwarder = HTTPForwarder::new(&listen_addr, &http_rule.name, http_rule.get_effective_buffer_size(8192));
        
        // 启动HTTP转发器
        http_forwarder.start().await?;
        
        // 保存转发器引用
        self.forwarders.write().await.insert("AutoHTTP".to_string(), Box::new(http_forwarder));
        
        info!("自动HTTP跳转服务启动成功");
        Ok(())
    }
    
    pub async fn stop(&mut self) {
        info!("停止智能转发器...");
        
        // 停止所有转发器
        let mut forwarders = self.forwarders.write().await;
        for (name, forwarder) in forwarders.iter_mut() {
            info!("停止转发器: {}", name);
            forwarder.stop().await;
        }
        forwarders.clear();
        
        info!("智能转发器已停止");
    }
    

}
