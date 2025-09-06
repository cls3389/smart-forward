# ================================
# 智能网络转发器 - 跨平台构建脚本
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
    Write-ColorOutput "🔍 检查构建依赖..." "Cyan"
    
    # 检查 Rust
    try {
        $rustVersion = rustc --version
        Write-ColorOutput "✅ Rust: $rustVersion" "Green"
    } catch {
        Handle-Error "未找到 Rust，请先安装: https://rustup.rs/"
    }
    
    # 检查 Cargo
    try {
        $cargoVersion = cargo --version
        Write-ColorOutput "✅ Cargo: $cargoVersion" "Green"
    } catch {
        Handle-Error "未找到 Cargo"
    }
    
    # 检查交叉编译工具链
    if ($Platform -eq "all" -or $Platform -ne "windows") {
        try {
            rustup target list --installed | Out-Null
            Write-ColorOutput "✅ 交叉编译工具链已安装" "Green"
        } catch {
            Write-ColorOutput "⚠️  需要安装交叉编译工具链" "Yellow"
        }
    }
}

# 安装交叉编译工具链
function Install-CrossCompileTargets {
    Write-ColorOutput "🔧 安装交叉编译工具链..." "Cyan"
    
    $targets = @()
    if ($Platform -eq "all" -or $Platform -eq "macos") {
        $targets += "x86_64-apple-darwin", "aarch64-apple-darwin"
    }
    if ($Platform -eq "all" -or $Platform -eq "linux") {
        $targets += "x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu"
    }
    
    foreach ($target in $targets) {
        Write-ColorOutput "安装目标: $target" "Yellow"
        rustup target add $target
        if ($LASTEXITCODE -ne 0) {
            Handle-Error "安装目标 $target 失败"
        }
    }
}

# 构建函数
function Build-Target {
    param(
        [string]$Target,
        [string]$OutputDir,
        [string]$BinaryName
    )
    
    Write-ColorOutput "🔨 构建目标: $Target" "Cyan"
    
    $buildArgs = @("build", "--target", $Target)
    if ($Release) {
        $buildArgs += "--release"
    }
    
    # 执行构建
    & cargo @buildArgs
    if ($LASTEXITCODE -ne 0) {
        Handle-Error "构建目标 $Target 失败"
    }
    
    # 确定源文件路径
    $sourcePath = if ($Release) {
        "target\$Target\release\$BinaryName"
    } else {
        "target\$Target\debug\$BinaryName"
    }
    
    # 确定目标文件路径
    $targetPath = "$OutputDir\$BinaryName"
    if ($Target -like "*windows*") {
        $targetPath += ".exe"
    }
    
    # 复制文件
    if (Test-Path $sourcePath) {
        Copy-Item $sourcePath $targetPath -Force
        Write-ColorOutput "✅ 构建完成: $targetPath" "Green"
    } else {
        Handle-Error "构建产物未找到: $sourcePath"
    }
}

# 清理函数
function Clear-BuildArtifacts {
    Write-ColorOutput "🧹 清理构建产物..." "Cyan"
    
    if (Test-Path "target") {
        Remove-Item "target" -Recurse -Force
    }
    
    if (Test-Path "dist") {
        Remove-Item "dist" -Recurse -Force
    }
    
    Write-ColorOutput "✅ 清理完成" "Green"
}

# Docker构建
function Build-Docker {
    Write-ColorOutput "🐳 构建 Docker 镜像..." "Cyan"
    
    # 构建镜像
    docker build -t smart-forward:latest .
    if ($LASTEXITCODE -ne 0) {
        Handle-Error "Docker 构建失败"
    }
    
    # 创建多架构镜像
    docker buildx create --use --name multiarch-builder
    docker buildx build --platform linux/amd64,linux/arm64 -t smart-forward:latest --push .
    
    Write-ColorOutput "✅ Docker 镜像构建完成" "Green"
}

# 主函数
function Main {
    Write-ColorOutput "🚀 智能网络转发器 - 跨平台构建" "Magenta"
    Write-ColorOutput "=================================" "Magenta"
    
    # 清理
    if ($Clean) {
        Clear-BuildArtifacts
        return
    }
    
    # 检查依赖
    Test-Dependencies
    
    # 安装交叉编译工具链
    if ($Platform -ne "windows") {
        Install-CrossCompileTargets
    }
    
    # 创建输出目录
    $distDir = "dist"
    if (!(Test-Path $distDir)) {
        New-Item -ItemType Directory -Path $distDir
    }
    
    # 构建目标
    $buildTargets = @()
    
    if ($Platform -eq "all" -or $Platform -eq "windows") {
        $buildTargets += @{
            Target = "x86_64-pc-windows-msvc"
            OutputDir = "$distDir\windows-x86_64"
            BinaryName = "smart-forward"
        }
    }
    
    if ($Platform -eq "all" -or $Platform -eq "macos") {
        if ($Arch -eq "all" -or $Arch -eq "x86_64") {
            $buildTargets += @{
                Target = "x86_64-apple-darwin"
                OutputDir = "$distDir\macos-x86_64"
                BinaryName = "smart-forward"
            }
        }
        if ($Arch -eq "all" -or $Arch -eq "aarch64") {
            $buildTargets += @{
                Target = "aarch64-apple-darwin"
                OutputDir = "$distDir\macos-aarch64"
                BinaryName = "smart-forward"
            }
        }
    }
    
    if ($Platform -eq "all" -or $Platform -eq "linux") {
        if ($Arch -eq "all" -or $Arch -eq "x86_64") {
            $buildTargets += @{
                Target = "x86_64-unknown-linux-gnu"
                OutputDir = "$distDir\linux-x86_64"
                BinaryName = "smart-forward"
            }
        }
        if ($Arch -eq "all" -or $Arch -eq "aarch64") {
            $buildTargets += @{
                Target = "aarch64-unknown-linux-gnu"
                OutputDir = "$distDir\linux-aarch64"
                BinaryName = "smart-forward"
            }
        }
    }
    
    # 执行构建
    foreach ($buildTarget in $buildTargets) {
        # 创建输出目录
        if (!(Test-Path $buildTarget.OutputDir)) {
            New-Item -ItemType Directory -Path $buildTarget.OutputDir
        }
        
        # 构建
        Build-Target -Target $buildTarget.Target -OutputDir $buildTarget.OutputDir -BinaryName $buildTarget.BinaryName
        
        # 复制配置文件
        Copy-Item "config.yaml.example" "$($buildTarget.OutputDir)\config.yaml" -Force
        Copy-Item "README.md" "$($buildTarget.OutputDir)\README.md" -Force
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
