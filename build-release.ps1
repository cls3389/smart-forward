# ================================
# 智能网络转发器 - 发布构建脚本
# ================================
# 支持平台: Windows, macOS, Linux
# 构建目标: x86_64, ARM64
# ================================

param(
    [string]$Platform = "all",  # all, windows, macos, linux
    [string]$Arch = "all",      # all, x86_64, aarch64
    [switch]$Release = $false,
    [switch]$Docker = $false,
    [switch]$Clean = $false
)

# 颜色输出函数
function Write-ColorOutput {
    param([string]$Message, [string]$Color = "White")
    Write-Host $Message -ForegroundColor $Color
}

# 错误处理
function Handle-Error {
    param([string]$Message)
    Write-ColorOutput "❌ 错误: $Message" "Red"
    exit 1
}

# 检查依赖
function Test-Dependencies {
    Write-ColorOutput "🔍 检查构建依赖..." "Yellow"
    
    # 检查 Rust
    try {
        $rustVersion = rustc --version
        Write-ColorOutput "✅ Rust: $rustVersion" "Green"
    } catch {
        Handle-Error "Rust 未安装或不在 PATH 中"
    }
    
    # 检查 Cargo
    try {
        $cargoVersion = cargo --version
        Write-ColorOutput "✅ Cargo: $cargoVersion" "Green"
    } catch {
        Handle-Error "Cargo 未安装或不在 PATH 中"
    }
}

# 清理函数
function Clear-Build {
    Write-ColorOutput "🧹 清理构建产物..." "Yellow"
    
    if (Test-Path "target") {
        Remove-Item -Recurse -Force "target"
        Write-ColorOutput "✅ 清理 target 目录" "Green"
    }
    
    if (Test-Path "dist") {
        Remove-Item -Recurse -Force "dist"
        Write-ColorOutput "✅ 清理 dist 目录" "Green"
    }
}

# 构建函数
function Build-Platform {
    param(
        [string]$Platform,
        [string]$Arch,
        [string]$Target,
        [string]$BinaryName
    )
    
    Write-ColorOutput "🔨 构建 $Platform ($Arch)..." "Yellow"
    
    # 添加目标平台
    rustup target add $Target
    
    # 构建
    $buildCmd = "cargo build --release --target $Target"
    if ($Release) {
        $buildCmd += " --verbose"
    }
    
    Invoke-Expression $buildCmd
    
    if ($LASTEXITCODE -ne 0) {
        Handle-Error "构建 $Platform ($Arch) 失败"
    }
    
    # 创建分发目录
    $distDir = "dist/$Platform-$Arch"
    New-Item -ItemType Directory -Force -Path $distDir | Out-Null
    
    # 复制二进制文件
    $sourcePath = "target/$Target/release/$BinaryName"
    $destPath = "$distDir/$BinaryName"
    
    if (Test-Path $sourcePath) {
        Copy-Item $sourcePath $destPath
        Write-ColorOutput "✅ 复制二进制文件: $destPath" "Green"
    } else {
        Handle-Error "二进制文件不存在: $sourcePath"
    }
    
    # 复制配置文件
    if (Test-Path "config.yaml") {
        Copy-Item "config.yaml" "$distDir/config.yaml"
    } elseif (Test-Path "config.yaml.example") {
        Copy-Item "config.yaml.example" "$distDir/config.yaml"
    }
    
    # 复制 README
    if (Test-Path "README.md") {
        Copy-Item "README.md" "$distDir/README.md"
    }
    
    Write-ColorOutput "✅ $Platform ($Arch) 构建完成" "Green"
}

# Docker 构建函数
function Build-Docker {
    Write-ColorOutput "🐳 构建 Docker 镜像..." "Yellow"
    
    # 构建多架构镜像
    docker buildx build --platform linux/amd64,linux/arm64 -t smart-forward:latest -f Dockerfile.simple .
    
    if ($LASTEXITCODE -ne 0) {
        Handle-Error "Docker 构建失败"
    }
    
    Write-ColorOutput "✅ Docker 镜像构建完成" "Green"
}

# 主函数
function Main {
    Write-ColorOutput "🚀 开始智能网络转发器发布构建..." "Cyan"
    Write-ColorOutput "平台: $Platform, 架构: $Arch, 发布模式: $Release" "Cyan"
    
    # 检查依赖
    Test-Dependencies
    
    # 清理
    if ($Clean) {
        Clear-Build
        return
    }
    
    # 创建分发目录
    $distDir = "dist"
    New-Item -ItemType Directory -Force -Path $distDir | Out-Null
    
    # 构建各平台
    if ($Platform -eq "all" -or $Platform -eq "windows") {
        if ($Arch -eq "all" -or $Arch -eq "x86_64") {
            Build-Platform "windows" "x86_64" "x86_64-pc-windows-msvc" "smart-forward.exe"
        }
    }
    
    if ($Platform -eq "all" -or $Platform -eq "macos") {
        if ($Arch -eq "all" -or $Arch -eq "x86_64") {
            Build-Platform "macos" "x86_64" "x86_64-apple-darwin" "smart-forward"
        }
        if ($Arch -eq "all" -or $Arch -eq "aarch64") {
            Build-Platform "macos" "aarch64" "aarch64-apple-darwin" "smart-forward"
        }
    }
    
    if ($Platform -eq "all" -or $Platform -eq "linux") {
        if ($Arch -eq "all" -or $Arch -eq "x86_64") {
            Build-Platform "linux" "x86_64" "x86_64-unknown-linux-gnu" "smart-forward"
        }
        if ($Arch -eq "all" -or $Arch -eq "aarch64") {
            Build-Platform "linux" "aarch64" "aarch64-unknown-linux-gnu" "smart-forward"
        }
    }
    
    # Docker构建
    if ($Docker) {
        Build-Docker
    }
    
    Write-ColorOutput "🎉 所有构建任务完成！" "Green"
    Write-ColorOutput "构建产物位于: $distDir" "Cyan"
}

# 执行主函数
Main
