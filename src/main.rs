mod common;
mod config;
mod firewall;
mod forwarder;
mod utils;

use anyhow::Result;
use chrono;
use clap::Parser;
use log::info;
use serde_json;
use std::path::PathBuf;

use crate::common::CommonManager;
use crate::config::Config;
use crate::firewall::{detect_firewall_backend, FirewallBackend, FirewallScheduler};
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

    /// 强制启用内核态转发模式
    #[arg(short, long)]
    kernel_mode: bool,

    /// 强制使用用户态转发模式（禁用内核态自动尝试）
    #[arg(long)]
    user_mode: bool,

    /// 防火墙后端选择 (nftables/iptables/auto)
    #[arg(long, default_value = "auto")]
    firewall_backend: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 设置时区为北京时间（当前std::env::set_var为unsafe API）
    unsafe {
        std::env::set_var("TZ", "Asia/Shanghai");
    }

    let args = Args::parse();

    // 后台运行处理
    if args.daemon {
        daemonize(&args.pid_file)?;
    }

    // 加载配置
    let config = Config::load_from_file(&args.config)?;

    // 初始化日志系统：正确使用配置文件中的日志设置
    let log_level = match config.logging.level.to_lowercase().as_str() {
        "debug" => log::LevelFilter::Debug,
        "info" => log::LevelFilter::Info,
        "warn" => log::LevelFilter::Warn,
        "error" => log::LevelFilter::Error,
        _ => log::LevelFilter::Info,
    };

    // 如果环境变量未设置，使用配置文件中的级别
    if std::env::var("RUST_LOG").is_err() {
        unsafe {
            std::env::set_var("RUST_LOG", &config.logging.level);
        }
    }

    let mut logger_builder = env_logger::Builder::from_default_env();
    logger_builder.filter_level(log_level);
    
    let is_json = config.logging.format.eq_ignore_ascii_case("json");
    if is_json {
        logger_builder.format(|buf, record| {
            use std::io::Write;
            let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            writeln!(
                buf,
                "{{\"timestamp\":\"{}\",\"level\":\"{}\",\"target\":\"{}\",\"message\":{}}}",
                ts,
                record.level(),
                record.target(),
                serde_json::to_string(&record.args().to_string())
                    .unwrap_or_else(|_| "\"\"".to_string())
            )
        });
    } else {
        logger_builder.format(|buf, record| {
            use std::io::Write;
            let beijing_time = chrono::Local::now();
            writeln!(
                buf,
                "[{} {} {}] {}",
                beijing_time.format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.target(),
                record.args()
            )
        });
    }
    logger_builder.init();

    // 显示启动信息
    println!("智能转发器 v{}", env!("CARGO_PKG_VERSION"));
    println!("配置文件: {}", args.config.display());
    println!("工作目录: {}", std::env::current_dir()?.display());
    println!(
        "日志级别: {}",
        std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string())
    );
    if args.daemon {
        println!(
            "运行模式: 后台运行 (PID: {})",
            std::fs::read_to_string(&args.pid_file).unwrap_or_else(|_| "未知".to_string())
        );
    } else {
        println!("运行模式: 前台运行");
    }

    info!("启动智能转发器...");

    // 解析防火墙后端
    let firewall_backend = match args.firewall_backend.as_str() {
        "nftables" => FirewallBackend::Nftables,
        "iptables" => FirewallBackend::Iptables,
        "auto" => detect_firewall_backend(),
        _ => {
            println!("⚠️  未知的防火墙后端: {}，使用自动检测", args.firewall_backend);
            detect_firewall_backend()
        }
    };

    // 如果只是验证配置，则显示配置信息并退出
    if args.validate_config {
        println!("=== 配置验证模式 ===");
        println!("✅ 配置文件加载成功");

        // 显示内核态转发信息
        if args.kernel_mode {
            println!("\n🚀 内核态转发模式:");
            println!("  防火墙后端: {:?}", firewall_backend);
            println!("  优先级优化: 启用（避免Firewall4规则冲突）");
        } else {
            println!("\n📡 用户态转发模式:");
            println!("  转发方式: 应用层代理");
        }

        // 验证全局动态更新配置
        let global_dynamic_config = config.get_dynamic_update_config();
        println!("\n📋 全局动态更新配置:");
        println!(
            "  检查间隔: {}秒",
            global_dynamic_config.get_check_interval()
        );
        println!(
            "  连接超时: {}秒",
            global_dynamic_config.get_connection_timeout()
        );
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
                println!("    协议: {protocols:?} (多协议同时转发)");
            }

            println!(
                "    缓冲区大小: {}字节",
                rule.get_effective_buffer_size(8192)
            );
            println!("    目标地址: {:?}", rule.targets);

            // 验证规则级别的动态更新配置
            let rule_dynamic_config = rule.get_dynamic_update_config(&global_dynamic_config);
            println!("    动态更新配置:");
            println!(
                "      检查间隔: {}秒",
                rule_dynamic_config.get_check_interval()
            );
            println!(
                "      连接超时: {}秒",
                rule_dynamic_config.get_connection_timeout()
            );
            println!(
                "      自动重连: {}",
                rule_dynamic_config.get_auto_reconnect()
            );
            println!();
        }

        println!("✅ 配置验证完成");
        println!("🎉 所有配置项验证通过！");
        return Ok(());
    }

    // 创建公共管理器
    let common_manager = CommonManager::new(config.clone());
    common_manager.initialize().await?;

    // 智能选择转发模式：默认优先内核态，失败自动回退用户态
    let firewall_scheduler = if cfg!(target_os = "linux") {
        // Linux环境：智能转发模式选择
        if args.user_mode {
            info!("📡 强制使用用户态转发模式");
            None
        } else if args.kernel_mode {
            info!("🚀 强制启用内核态转发模式");
            let mut scheduler = FirewallScheduler::new(
                firewall_backend,
                config.clone(),
                common_manager.clone(),
            ).await?;
            scheduler.initialize().await?;
            info!("✅ 内核态转发启用成功，防火墙后端: {:?}", firewall_backend);
            Some(scheduler)
        } else if !args.validate_config {
            // 默认行为：自动尝试内核态转发，失败则回退
            info!("🚀 自动尝试内核态转发（优先高性能模式）");
            match FirewallScheduler::new(
                firewall_backend,
                config.clone(),
                common_manager.clone(),
            ).await {
                Ok(mut scheduler) => {
                    match scheduler.initialize().await {
                        Ok(_) => {
                            info!("✅ 内核态转发自动启用成功，防火墙后端: {:?}", firewall_backend);
                            Some(scheduler)
                        }
                        Err(e) => {
                            warn!("⚠️  内核态转发初始化失败: {}，自动回退到用户态转发", e);
                            info!("💡 提示：可使用 --user-mode 禁用内核态自动尝试");
                            None
                        }
                    }
                }
                Err(e) => {
                    warn!("⚠️  内核态转发创建失败: {}，自动回退到用户态转发", e);
                    info!("💡 提示：可使用 --user-mode 禁用内核态自动尝试");
                    None
                }
            }
        } else {
            None
        }
    } else {
        if args.kernel_mode {
            warn!("⚠️  内核态转发仅支持Linux系统，在{}上自动使用用户态转发", std::env::consts::OS);
        }
        None
    };

    // 创建智能转发器
    let mut forwarder = SmartForwarder::new(config, common_manager, firewall_scheduler);

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
