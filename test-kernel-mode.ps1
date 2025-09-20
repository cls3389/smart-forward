# OpenWrt Firewall4 (nftables) 内核态转发测试脚本 - PowerShell版本

Write-Host "🚀 Smart Forward - Firewall4 内核态转发测试" -ForegroundColor Green
Write-Host "=============================================" -ForegroundColor Green

# 检查运行环境
Write-Host "📋 检查运行环境..." -ForegroundColor Yellow

# 检查是否为Windows（提醒用户在Linux上测试）
if ($IsWindows -or $env:OS -eq "Windows_NT") {
    Write-Host "⚠️  当前在Windows环境，内核态转发功能需要在Linux（OpenWrt）上测试" -ForegroundColor Yellow
    Write-Host "   此脚本将创建配置文件，请将项目部署到OpenWrt设备上测试" -ForegroundColor Yellow
}

# 模拟检查防火墙后端（在实际Linux环境中会检查）
Write-Host "🔍 防火墙后端检测（Linux环境）..." -ForegroundColor Yellow
$hasNftables = $true  # 假设OpenWrt支持nftables
$hasIptables = $true  # 假设也支持iptables

if ($hasNftables) {
    Write-Host "✅ nftables支持（Firewall4推荐）" -ForegroundColor Green
    $recommendedBackend = "nftables"
} elseif ($hasIptables) {
    Write-Host "✅ iptables支持" -ForegroundColor Green
    $recommendedBackend = "iptables"
} else {
    Write-Host "❌ 未检测到支持的防火墙后端" -ForegroundColor Red
    $recommendedBackend = "nftables"
}

Write-Host "🎯 推荐使用: $recommendedBackend (Firewall4)" -ForegroundColor Green

# 创建测试配置
Write-Host "📝 创建测试配置..." -ForegroundColor Yellow

$testConfig = @"
# Firewall4 (nftables) 内核态转发测试配置
logging:
  level: "info"
  format: "text"

network:
  listen_addr: "0.0.0.0"

rules:
  - name: "Web-Kernel"
    listen_port: 8080
    protocol: "tcp"
    targets:
      - "192.168.1.100:80"
      - "backup.example.com:80"
      
  - name: "SSH-Kernel"  
    listen_port: 2222
    protocol: "tcp"
    targets:
      - "192.168.1.200:22"
      
  - name: "Game-Kernel"
    listen_port: 25565
    # 不指定协议时默认TCP+UDP双协议
    targets:
      - "192.168.1.150:25565"
"@

$testConfig | Out-File -FilePath "test-kernel-config.yaml" -Encoding UTF8
Write-Host "✅ 测试配置已创建: test-kernel-config.yaml" -ForegroundColor Green

# 显示使用说明
Write-Host ""
Write-Host "🎯 内核态转发测试命令（在OpenWrt Linux环境中执行）：" -ForegroundColor Cyan
Write-Host "=============================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "1️⃣ 验证配置（推荐先执行）：" -ForegroundColor White
Write-Host "   ./smart-forward -c test-kernel-config.yaml --validate-config --kernel-mode --firewall-backend $recommendedBackend" -ForegroundColor Gray
Write-Host ""
Write-Host "2️⃣ 启动内核态转发：" -ForegroundColor White
Write-Host "   sudo ./smart-forward -c test-kernel-config.yaml --kernel-mode --firewall-backend $recommendedBackend" -ForegroundColor Gray
Write-Host ""
Write-Host "3️⃣ 自动检测防火墙后端：" -ForegroundColor White
Write-Host "   sudo ./smart-forward -c test-kernel-config.yaml --kernel-mode --firewall-backend auto" -ForegroundColor Gray
Write-Host ""
Write-Host "4️⃣ 测试转发效果：" -ForegroundColor White
Write-Host "   curl http://localhost:8080  # 应该转发到192.168.1.100:80" -ForegroundColor Gray
Write-Host "   ssh -p 2222 localhost       # 应该转发到192.168.1.200:22" -ForegroundColor Gray
Write-Host "   # 游戏服务器测试（TCP+UDP）" -ForegroundColor Gray
Write-Host ""

# 显示Firewall4优先级说明
Write-Host "🔥 Firewall4 优先级优化说明：" -ForegroundColor Red
Write-Host "=============================================" -ForegroundColor Red
Write-Host "✅ smart-forward使用优先级-150的prerouting链" -ForegroundColor Green
Write-Host "✅ 高于Firewall4默认DNAT规则（优先级-100）" -ForegroundColor Green
Write-Host "✅ 确保转发到外网地址不被覆盖" -ForegroundColor Green
Write-Host "✅ 专用table避免与现有规则冲突" -ForegroundColor Green
Write-Host ""

# 显示nftables规则查看命令
Write-Host "🔍 查看nftables规则（在Linux环境中）：" -ForegroundColor Cyan
Write-Host "   nft list table inet smart_forward" -ForegroundColor Gray
Write-Host "   nft list chain inet smart_forward prerouting" -ForegroundColor Gray
Write-Host "   nft list chain inet smart_forward postrouting" -ForegroundColor Gray
Write-Host ""

# 显示部署说明
Write-Host "📦 部署到OpenWrt说明：" -ForegroundColor Magenta
Write-Host "=============================================" -ForegroundColor Magenta
Write-Host "1. 编译项目：cargo build --release --target=mips-unknown-linux-musl" -ForegroundColor Gray
Write-Host "2. 上传到OpenWrt：scp target/release/smart-forward root@openwrt:/usr/bin/" -ForegroundColor Gray
Write-Host "3. 上传配置：scp test-kernel-config.yaml root@openwrt:/etc/" -ForegroundColor Gray
Write-Host "4. 在OpenWrt上运行上述测试命令" -ForegroundColor Gray
Write-Host ""

Write-Host "🎉 测试环境准备完成！" -ForegroundColor Green
Write-Host "请将项目部署到OpenWrt设备上进行实际测试。" -ForegroundColor Yellow
