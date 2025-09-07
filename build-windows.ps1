# 本地构建 Windows 版本
param(
    [switch]$Release = $true
)

Write-Host "构建 Windows x86_64 版本" -ForegroundColor Cyan

# 1. 检查 Rust 环境
Write-Host "`n检查 Rust 环境..." -ForegroundColor Yellow
rustc --version
cargo --version

# 2. 添加 Windows 目标
Write-Host "`n添加 Windows 目标..." -ForegroundColor Yellow
rustup target add x86_64-pc-windows-msvc

# 3. 构建项目
Write-Host "`n构建项目..." -ForegroundColor Yellow
if ($Release) {
    cargo build --release --target x86_64-pc-windows-msvc
} else {
    cargo build --target x86_64-pc-windows-msvc
}

# 4. 检查构建结果
$binaryPath = "target\x86_64-pc-windows-msvc\release\smart-forward.exe"
if (Test-Path $binaryPath) {
    $size = (Get-Item $binaryPath).Length
    Write-Host "`n✅ 构建成功！" -ForegroundColor Green
    Write-Host "文件位置: $binaryPath" -ForegroundColor White
    Write-Host "文件大小: $([math]::Round($size/1MB, 2)) MB" -ForegroundColor White
    
    # 5. 创建发布包
    Write-Host "`n创建发布包..." -ForegroundColor Yellow
    $releaseDir = "release\windows-x86_64"
    New-Item -ItemType Directory -Path $releaseDir -Force | Out-Null
    
    # 复制文件
    Copy-Item $binaryPath "$releaseDir\smart-forward.exe"
    Copy-Item "config.yaml.example" "$releaseDir\config.yaml"
    Copy-Item "README.md" "$releaseDir\README.md" -ErrorAction SilentlyContinue
    
    # 创建压缩包
    $zipPath = "smart-forward-windows-x86_64.zip"
    Compress-Archive -Path "$releaseDir\*" -DestinationPath $zipPath -Force
    
    Write-Host "✅ 发布包已创建: $zipPath" -ForegroundColor Green
    Write-Host "文件内容:" -ForegroundColor Cyan
    Get-ChildItem $releaseDir | ForEach-Object { Write-Host "  - $($_.Name)" -ForegroundColor White }
    
} else {
    Write-Host "`n❌ 构建失败！" -ForegroundColor Red
    Write-Host "请检查错误信息" -ForegroundColor Yellow
}
