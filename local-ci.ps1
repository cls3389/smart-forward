# ================================
# 本地 CI 脚本 - PowerShell 版本
# ================================
# 功能：替代 GitHub Actions，在本地执行完整的 CI 流程
# 优势：无计费限制，快速反馈，完全控制

param(
    [switch]$SkipTests = $false,
    [switch]$SkipBuild = $false,
    [switch]$SkipSecurity = $false,
    [switch]$Verbose = $false,
    [string]$Target = "all"
)

# 颜色输出函数
function Write-ColorOutput {
    param(
        [string]$Message,
        [string]$Color = "White"
    )
    Write-Host $Message -ForegroundColor $Color
}

# 错误处理
function Handle-Error {
    param([string]$Step, [string]$Error)
    Write-ColorOutput "❌ $Step 失败: $Error" "Red"
    exit 1
}

# 检查工具是否安装
function Test-Command {
    param([string]$Command)
    try {
        Get-Command $Command -ErrorAction Stop | Out-Null
        return $true
    } catch {
        return $false
    }
}

Write-ColorOutput "🚀 开始本地 CI 流程..." "Cyan"
Write-ColorOutput "目标: $Target" "Yellow"
Write-ColorOutput "跳过测试: $SkipTests" "Yellow"
Write-ColorOutput "跳过构建: $SkipBuild" "Yellow"
Write-ColorOutput "跳过安全扫描: $SkipSecurity" "Yellow"

# 1. 环境检查
Write-ColorOutput "`n📋 检查环境..." "Cyan"

if (-not (Test-Command "cargo")) {
    Handle-Error "环境检查" "Rust/Cargo 未安装"
}

if (-not (Test-Command "rustup")) {
    Handle-Error "环境检查" "Rustup 未安装"
}

Write-ColorOutput "✅ 环境检查通过" "Green"

# 2. 代码格式化检查
Write-ColorOutput "`n🎨 检查代码格式..." "Cyan"
try {
    $formatResult = cargo fmt -- --check 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-ColorOutput "⚠️  代码格式需要调整，运行: cargo fmt" "Yellow"
        if ($Verbose) {
            Write-ColorOutput $formatResult "Yellow"
        }
    } else {
        Write-ColorOutput "✅ 代码格式检查通过" "Green"
    }
} catch {
    Handle-Error "代码格式检查" $_.Exception.Message
}

# 3. Clippy 检查
Write-ColorOutput "`n🔍 运行 Clippy 检查..." "Cyan"
try {
    $clippyResult = cargo clippy -- -D warnings 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-ColorOutput "⚠️  Clippy 发现警告，请修复:" "Yellow"
        Write-ColorOutput $clippyResult "Yellow"
    } else {
        Write-ColorOutput "✅ Clippy 检查通过" "Green"
    }
} catch {
    Handle-Error "Clippy 检查" $_.Exception.Message
}

# 4. 运行测试
if (-not $SkipTests) {
    Write-ColorOutput "`n🧪 运行测试..." "Cyan"
    try {
        $testResult = cargo test --verbose 2>&1
        if ($LASTEXITCODE -ne 0) {
            Handle-Error "测试" "测试失败"
        } else {
            Write-ColorOutput "✅ 所有测试通过" "Green"
        }
    } catch {
        Handle-Error "测试" $_.Exception.Message
    }
} else {
    Write-ColorOutput "⏭️  跳过测试" "Yellow"
}

# 5. 安全扫描
if (-not $SkipSecurity) {
    Write-ColorOutput "`n🔒 运行安全扫描..." "Cyan"
    try {
        # 检查是否安装了 cargo-audit
        if (Test-Command "cargo-audit") {
            $auditResult = cargo audit 2>&1
            if ($LASTEXITCODE -ne 0) {
                Write-ColorOutput "⚠️  发现安全漏洞:" "Yellow"
                Write-ColorOutput $auditResult "Yellow"
            } else {
                Write-ColorOutput "✅ 安全扫描通过" "Green"
            }
        } else {
            Write-ColorOutput "⚠️  cargo-audit 未安装，跳过安全扫描" "Yellow"
            Write-ColorOutput "安装命令: cargo install cargo-audit" "Yellow"
        }
    } catch {
        Write-ColorOutput "⚠️  安全扫描失败: $($_.Exception.Message)" "Yellow"
    }
} else {
    Write-ColorOutput "⏭️  跳过安全扫描" "Yellow"
}

# 6. 构建
if (-not $SkipBuild) {
    Write-ColorOutput "`n🔨 开始构建..." "Cyan"
    
    $targets = @()
    switch ($Target) {
        "linux" { $targets = @("x86_64-unknown-linux-gnu") }
        "windows" { $targets = @("x86_64-pc-windows-msvc") }
        "all" { $targets = @("x86_64-unknown-linux-gnu", "x86_64-pc-windows-msvc") }
        default { $targets = @($Target) }
    }
    
    foreach ($target in $targets) {
        Write-ColorOutput "构建目标: $target" "Yellow"
        try {
            # 添加目标（如果不存在）
            rustup target add $target 2>$null
            
            # 构建
            $buildResult = cargo build --release --target $target 2>&1
            if ($LASTEXITCODE -ne 0) {
                Handle-Error "构建 $target" "构建失败"
            } else {
                Write-ColorOutput "✅ $target 构建成功" "Green"
            }
        } catch {
            Handle-Error "构建 $target" $_.Exception.Message
        }
    }
} else {
    Write-ColorOutput "⏭️  跳过构建" "Yellow"
}

# 7. 生成报告
Write-ColorOutput "`n📊 生成 CI 报告..." "Cyan"

$reportPath = "ci-report-$(Get-Date -Format 'yyyyMMdd-HHmmss').txt"
$report = @"
# CI 报告
生成时间: $(Get-Date)
目标: $Target
跳过测试: $SkipTests
跳过构建: $SkipBuild
跳过安全扫描: $SkipSecurity

## 构建结果
"@

if (-not $SkipBuild) {
    foreach ($target in $targets) {
        $binaryPath = "target/$target/release/smart-forward"
        if ($target -like "*windows*") {
            $binaryPath += ".exe"
        }
        
        if (Test-Path $binaryPath) {
            $size = (Get-Item $binaryPath).Length
            $report += "`n- $target : $binaryPath ($([math]::Round($size/1MB, 2)) MB)"
        } else {
            $report += "`n- $target : 构建失败"
        }
    }
}

$report | Out-File -FilePath $reportPath -Encoding UTF8
Write-ColorOutput "✅ CI 报告已保存: $reportPath" "Green"

Write-ColorOutput "`n🎉 本地 CI 流程完成！" "Green"
Write-ColorOutput "总耗时: $($(Get-Date) - $startTime)" "Cyan"
