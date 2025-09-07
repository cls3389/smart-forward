# Smart Forward Packaging Script
Write-Host "Creating package directory..." -ForegroundColor Green
$distPath = "dist"

# Ensure directory exists
if (-not (Test-Path $distPath)) {
    New-Item -ItemType Directory -Force -Path $distPath | Out-Null
    Write-Host "✅ Directory created: $distPath" -ForegroundColor Green
} else {
    Write-Host "✅ Directory already exists: $distPath" -ForegroundColor Yellow
}

# Check if build target exists
$releasePath = "target/x86_64-pc-windows-msvc/release/smart-forward.exe"
if (-not (Test-Path $releasePath)) {
    Write-Host "❌ Error: Release binary not found. Please run 'cargo build --release --target x86_64-pc-windows-msvc' first" -ForegroundColor Red
    exit 1
}

Write-Host "Copying binary file..." -ForegroundColor Green
Copy-Item $releasePath $distPath/

Write-Host "Copying configuration files..." -ForegroundColor Green
if (Test-Path "config.yaml.example") {
    Copy-Item "config.yaml.example" "$distPath/config.yaml"
} else {
    Write-Host "⚠️  Warning: config.yaml.example not found" -ForegroundColor Yellow
}

Write-Host "Copying documentation..." -ForegroundColor Green
if (Test-Path "README.md") {
    Copy-Item "README.md" $distPath/
} else {
    Write-Host "⚠️  Warning: README.md not found" -ForegroundColor Yellow
}

Write-Host "Package contents:" -ForegroundColor Cyan
Get-ChildItem $distPath -Recurse | Format-Table Name, Length, LastWriteTime

Write-Host "✅ Packaging completed!" -ForegroundColor Green
Write-Host "Output directory: $(Resolve-Path $distPath)" -ForegroundColor White
