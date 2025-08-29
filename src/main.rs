mod config;
mod forwarder;
mod common;
mod utils;
mod stats;

use anyhow::Result;
use clap::Parser;
use log::info;
use std::path::PathBuf;

use crate::config::Config;
use crate::common::CommonManager;
use crate::forwarder::SmartForwarder;

/// 后台运行处理
fn daemonize(pid_file: &PathBuf) -> Result<()> {
    use std::process;
    
    // 创建PID文件
    let pid = process::id();
    std::fs::write(pid_file, pid.to_string())?;
    
    // 在Windows上，后台运行主要通过启动脚本实现
    // 这里主要是创建PID文件用于进程管理
    
    Ok(())
}

#[derive(Parser)]
#[command(name = "smart-forward")]
#[command(about = "智能网络转发器")]
struct Args {
    /// 配置文件路径
    #[arg(short, long, default_value = "config.yaml")]
    config: PathBuf,
    
    /// 后台运行模式
    #[arg(short, long)]
    daemon: bool,
    
    /// 后台运行时PID文件路径
    #[arg(long, default_value = "smart-forward.pid")]
    pid_file: PathBuf,

    /// 验证配置模式
    #[arg(short, long)]
    validate_config: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 设置时区为北京时间
    std::env::set_var("TZ", "Asia/Shanghai");

    let args = Args::parse();
    
    // 后台运行处理
    if args.daemon {
        daemonize(&args.pid_file)?;
    }
    
    // 加载配置
    let config = Config::load_from_file(&args.config)?;

    // 初始化日志：优先使用配置中的 level/format，若环境已设置 RUST_LOG 则尊重环境
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", &config.logging.level);
    }

    let mut logger_builder = env_logger::Builder::from_default_env();
    let is_json = config.logging.format.eq_ignore_ascii_case("json");
    if is_json {
        logger_builder.format(|buf, record| {
            use std::io::Write;
            use chrono::prelude::*;
            let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            writeln!(
                buf,
                "{{\"ts\":\"{}\",\"level\":\"{}\",\"msg\":{}}}",
                ts,
                record.level(),
                serde_json::to_string(&record.args().to_string()).unwrap_or_else(|_| "\"\"".to_string())
            )
        });
    } else {
        logger_builder.format(|buf, record| {
            use std::io::Write;
            use chrono::prelude::*;
            let beijing_time = Local::now();
            writeln!(
                buf,
                "[{} {}] {}",
                beijing_time.format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            )
        });
    }
    logger_builder.init();

    // 显示启动信息
    println!("智能转发器 v0.1.0");
    println!("配置文件: {}", args.config.display());
    println!("工作目录: {}", std::env::current_dir()?.display());
    println!("日志级别: {}", std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()));
    if args.daemon {
        println!("运行模式: 后台运行 (PID: {})", std::fs::read_to_string(&args.pid_file).unwrap_or_else(|_| "未知".to_string()));
    } else {
        println!("运行模式: 前台运行");
    }
    println!("");
    
    info!("启动智能转发器...");
    
    // 如果只是验证配置，则显示配置信息并退出
    if args.validate_config {
        println!("=== 配置验证模式 ===");
        println!("✅ 配置文件加载成功");
        
        // 验证全局动态更新配置
        let global_dynamic_config = config.get_dynamic_update_config();
        println!("\n📋 全局动态更新配置:");
        println!("  检查间隔: {}秒", global_dynamic_config.get_check_interval());
        println!("  连接超时: {}秒", global_dynamic_config.get_connection_timeout());
        println!("  自动重连: {}", global_dynamic_config.get_auto_reconnect());
        
        // 验证规则配置
        println!("\n📋 转发规则配置:");
        for (i, rule) in config.rules.iter().enumerate() {
            println!("  规则 {}: {}", i + 1, rule.name);
            println!("    监听端口: {}", rule.listen_port);
            
            // 显示协议信息
            let protocols = rule.get_protocols();
            if protocols.len() == 1 {
                println!("    协议: {}", protocols[0]);
            } else {
                println!("    协议: {:?} (多协议同时转发)", protocols);
            }
            
            println!("    缓冲区大小: {}字节", rule.get_effective_buffer_size(8192));
            println!("    目标地址: {:?}", rule.targets);
            
            // 验证规则级别的动态更新配置
            let rule_dynamic_config = rule.get_dynamic_update_config(&global_dynamic_config);
            println!("    动态更新配置:");
            println!("      检查间隔: {}秒", rule_dynamic_config.get_check_interval());
            println!("      连接超时: {}秒", rule_dynamic_config.get_connection_timeout());
            println!("      自动重连: {}", rule_dynamic_config.get_auto_reconnect());
            println!();
        }
        
        println!("✅ 配置验证完成");
        println!("🎉 所有配置项验证通过！");
        return Ok(());
    }
    
    // 创建公共管理器
    let common_manager = CommonManager::new(config.clone());
    common_manager.initialize().await?;
    
    // 创建智能转发器
    let mut forwarder = SmartForwarder::new(config, common_manager);
    
    // 初始化转发器
    forwarder.initialize().await?;
    
    // 启动转发器
    forwarder.start().await?;
    
    // 等待关闭信号
    tokio::signal::ctrl_c().await?;
    
    info!("收到关闭信号，正在停止...");
    
    // 停止转发器
    forwarder.stop().await;
    
    info!("智能转发器已停止");
    Ok(())
}