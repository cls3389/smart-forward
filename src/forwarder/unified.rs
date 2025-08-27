use crate::config::ForwardRule;
use anyhow::Result;
use async_trait::async_trait;
use log::{info, error};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Instant;

use super::{Forwarder, TCPForwarder, HTTPForwarder, UDPForwarder};

pub struct UnifiedForwarder {
    rule: ForwardRule,
    listen_addr: String,
    target_addr: String,
    tcp_forwarder: Option<TCPForwarder>,
    http_forwarder: Option<HTTPForwarder>,
    udp_forwarder: Option<UDPForwarder>,
    running: Arc<RwLock<bool>>,
    // 新增：动态地址更新支持
    last_target_update: Arc<RwLock<Instant>>,
}

impl UnifiedForwarder {
    pub fn new_with_target(rule: &ForwardRule, listen_addr: &str, target_addr: &str) -> Self {
        Self {
            rule: rule.clone(),
            listen_addr: listen_addr.to_string(),
            target_addr: target_addr.to_string(),
            tcp_forwarder: None,
            http_forwarder: None,
            udp_forwarder: None,
            running: Arc::new(RwLock::new(false)),
            last_target_update: Arc::new(RwLock::new(Instant::now())),
        }
    }
    

    
    // 新增：更新目标地址
    pub async fn update_target(&mut self, new_target: &str) -> Result<()> {
        if self.target_addr != new_target {
            info!("统一转发器 {} 更新目标地址: {} -> {}", 
                self.rule.name, self.target_addr, new_target);
            
            self.target_addr = new_target.to_string();
            *self.last_target_update.write().await = Instant::now();
            
            // 如果TCP转发器正在运行，更新其目标地址
            if let Some(tcp_forwarder) = &mut self.tcp_forwarder {
                if let Err(e) = tcp_forwarder.update_target(new_target).await {
                    log::error!("更新TCP转发器目标地址失败 {}: {}", new_target, e);
                    return Err(e);
                }
            }
            
            // 如果UDP转发器正在运行，更新其目标地址
            if let Some(udp_forwarder) = &mut self.udp_forwarder {
                if let Err(e) = udp_forwarder.update_target(new_target).await {
                    log::error!("更新UDP转发器目标地址失败 {}: {}", new_target, e);
                    return Err(e);
                }
            }
            
            // 如果HTTP转发器正在运行，也需要更新（如果有相关配置）
            // 这里可以根据需要添加HTTP转发器的地址更新逻辑
            
            info!("统一转发器 {} 目标地址更新成功: {}", self.rule.name, new_target);
        }
        Ok(())
    }
    

    
    // 新增：使用自定义间隔检查是否需要更新目标地址
    pub async fn should_update_target_with_interval(&self, interval_seconds: u64) -> bool {
        let last_update = *self.last_target_update.read().await;
        let interval = tokio::time::Duration::from_secs(interval_seconds);
        Instant::now().duration_since(last_update) >= interval
    }
    
    // 新增：获取当前目标地址
    pub fn get_current_target(&self) -> &str {
        &self.target_addr
    }
    
    // 新增：设置协议类型
    pub async fn set_protocol(&mut self, protocol: &str) -> Result<()> {
        // 创建一个新的规则副本，设置指定的协议
        let mut new_rule = self.rule.clone();
        new_rule.protocol = Some(protocol.to_string());
        self.rule = new_rule;
        Ok(())
    }
    
    // 新增：设置多协议支持
    pub async fn set_multi_protocol(&mut self, protocols: &[String]) -> Result<()> {
        // 创建一个新的规则副本，设置多协议
        let mut new_rule = self.rule.clone();
        new_rule.protocols = Some(protocols.to_vec());
        new_rule.protocol = None; // 清除单协议设置
        self.rule = new_rule;
        Ok(())
    }
}

#[async_trait]
impl Forwarder for UnifiedForwarder {
    async fn start(&mut self) -> Result<()> {
        info!("启动统一转发器: {}", self.rule.name);
        
        let protocols = self.rule.get_protocols();
        
        // 处理多协议情况
        if protocols.len() > 1 {
            // 多协议同时转发：需要同时启动TCP和UDP
            for protocol in &protocols {
                match protocol.as_str() {
                    "tcp" => {
                        if self.tcp_forwarder.is_none() {
                            let mut tcp_forwarder = TCPForwarder::new(
                                &self.listen_addr,
                                &format!("{}_TCP", self.rule.name),
                                self.rule.get_effective_buffer_size(16384),
                            );
                            
                            if let Err(e) = tcp_forwarder.start_with_target(&self.target_addr).await {
                                error!("TCP转发器启动失败 {}: {}", self.rule.name, e);
                                return Err(e);
                            }
                            
                            self.tcp_forwarder = Some(tcp_forwarder);
                            info!("TCP转发器启动成功: {} -> {}", self.rule.name, self.target_addr);
                        }
                    }
                    "udp" => {
                        if self.udp_forwarder.is_none() {
                            let mut udp_forwarder = UDPForwarder::new(
                                &self.listen_addr,
                                &format!("{}_UDP", self.rule.name),
                                self.rule.get_effective_buffer_size(16384),
                            );
                            
                            if let Err(e) = udp_forwarder.start_with_target(&self.target_addr).await {
                                error!("UDP转发器启动失败 {}: {}", self.rule.name, e);
                                return Err(e);
                            }
                            
                            self.udp_forwarder = Some(udp_forwarder);
                            info!("UDP转发器启动成功: {} -> {}", self.rule.name, self.target_addr);
                        }
                    }
                    "http" => {
                        if self.http_forwarder.is_none() {
                            let mut http_forwarder = HTTPForwarder::new(
                                &self.listen_addr,
                                &format!("{}_HTTP", self.rule.name),
                                self.rule.get_effective_buffer_size(16384),
                            );
                            
                            if let Err(e) = http_forwarder.start().await {
                                error!("HTTP转发器启动失败 {}: {}", self.rule.name, e);
                                return Err(e);
                            }
                            
                            self.http_forwarder = Some(http_forwarder);
                            info!("HTTP转发器启动成功: {} -> {}", self.rule.name, self.target_addr);
                        }
                    }
                    _ => {
                        let error_msg = format!("不支持的协议: {}", protocol);
                        error!("统一转发器 {} {}", self.rule.name, error_msg);
                        return Err(anyhow::anyhow!(error_msg));
                    }
                }
            }
            
            info!("多协议转发器启动成功: {} (协议: {:?})", self.rule.name, protocols);
        } else {
            // 单协议转发（原有逻辑）
            let protocol = protocols[0].clone();
            
            match protocol.as_str() {
                "tcp" => {
                    let mut tcp_forwarder = TCPForwarder::new(
                        &self.listen_addr,
                        &self.rule.name,
                        self.rule.get_effective_buffer_size(16384),
                    );
                    
                    if let Err(e) = tcp_forwarder.start_with_target(&self.target_addr).await {
                        error!("TCP转发器启动失败 {}: {}", self.rule.name, e);
                        return Err(e);
                    }
                    
                    self.tcp_forwarder = Some(tcp_forwarder);
                    info!("TCP转发器启动成功: {} -> {}", self.rule.name, self.target_addr);
                }
                "udp" => {
                    let mut udp_forwarder = UDPForwarder::new(
                        &self.listen_addr,
                        &self.rule.name,
                        self.rule.get_effective_buffer_size(16384),
                    );
                    
                    if let Err(e) = udp_forwarder.start_with_target(&self.target_addr).await {
                        error!("UDP转发器启动失败 {}: {}", self.rule.name, e);
                        return Err(e);
                    }
                    
                    self.udp_forwarder = Some(udp_forwarder);
                    info!("UDP转发器启动成功: {} -> {}", self.rule.name, self.target_addr);
                }
                "http" => {
                    let mut http_forwarder = HTTPForwarder::new(
                        &self.listen_addr,
                        &self.rule.name,
                        self.rule.get_effective_buffer_size(16384),
                    );
                    
                    if let Err(e) = http_forwarder.start().await {
                        error!("HTTP转发器启动失败 {}: {}", self.rule.name, e);
                        return Err(e);
                    }
                    
                    self.http_forwarder = Some(http_forwarder);
                    info!("HTTP转发器启动成功: {} -> {}", self.rule.name, self.target_addr);
                }
                _ => {
                    let error_msg = format!("不支持的协议: {}", protocol);
                    error!("统一转发器 {} {}", self.rule.name, error_msg);
                    return Err(anyhow::anyhow!(error_msg));
                }
            }
        }
        
        *self.running.write().await = true;
        info!("统一转发器 {} 启动完成", self.rule.name);
        Ok(())
    }
    
    async fn stop(&mut self) {
        info!("停止统一转发器: {}", self.rule.name);
        
        // 停止TCP转发器
        if let Some(mut tcp_forwarder) = self.tcp_forwarder.take() {
            tcp_forwarder.stop().await;
        }
        
        // 停止UDP转发器
        if let Some(mut udp_forwarder) = self.udp_forwarder.take() {
            udp_forwarder.stop().await;
        }
        
        // 停止HTTP转发器
        if let Some(mut http_forwarder) = self.http_forwarder.take() {
            http_forwarder.stop().await;
        }
        
        *self.running.write().await = false;
        info!("统一转发器已停止: {}", self.rule.name);
    }
    
    fn is_running(&self) -> bool {
        *self.running.blocking_read()
    }
    
    fn get_stats(&self) -> HashMap<String, String> {
        let mut stats = HashMap::new();
        
        stats.insert("rule_name".to_string(), self.rule.name.clone());
        stats.insert("listen_port".to_string(), self.rule.listen_port.to_string());
        stats.insert("protocol".to_string(), self.rule.get_protocol());
        stats.insert("target_addr".to_string(), self.target_addr.clone());
        stats.insert("last_target_update".to_string(), {
            let last_update = *self.last_target_update.blocking_read();
            format!("{:?}", last_update.elapsed())
        });
        
        // 添加TCP转发器统计信息
        if let Some(tcp_forwarder) = &self.tcp_forwarder {
            let tcp_stats = tcp_forwarder.get_stats();
            for (key, value) in tcp_stats {
                stats.insert(format!("tcp_{}", key), value);
            }
        }
        
        // 添加UDP转发器统计信息
        if let Some(udp_forwarder) = &self.udp_forwarder {
            let udp_stats = udp_forwarder.get_stats();
            for (key, value) in udp_stats {
                stats.insert(format!("udp_{}", key), value);
            }
        }
        
        // 添加HTTP转发器统计信息
        if let Some(http_forwarder) = &self.http_forwarder {
            let http_stats = http_forwarder.get_stats();
            for (key, value) in http_stats {
                stats.insert(format!("http_{}", key), value);
            }
        }
        
        stats
    }
    
    // 新增：实现as_any方法
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    // 新增：实现as_any_mut方法
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

