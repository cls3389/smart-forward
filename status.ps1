# Check release status
Write-Host "Checking Smart Forward v1.0.0 release status..." -ForegroundColor Cyan

Write-Host "`nLocal Status:" -ForegroundColor Yellow
Write-Host "Current branch: $(git branch --show-current)" -ForegroundColor Green
Write-Host "Latest commit: $(git log -1 --oneline)" -ForegroundColor Green
Write-Host "Version tag: $(git tag --list | Select-String 'v1.0.0')" -ForegroundColor Green

Write-Host "`nGitHub Actions Status:" -ForegroundColor Yellow
Write-Host "Check build progress at:" -ForegroundColor Cyan
Write-Host "https://github.com/cls3389/smart-forward/actions" -ForegroundColor Blue

Write-Host "`nRelease Page:" -ForegroundColor Yellow
Write-Host "Check release status at:" -ForegroundColor Cyan
Write-Host "https://github.com/cls3389/smart-forward/releases" -ForegroundColor Blue

Write-Host "`nDocker Images:" -ForegroundColor Yellow
Write-Host "Multi-arch images will be published to:" -ForegroundColor Cyan
Write-Host "ghcr.io/cls3389/smart-forward:1.0.0" -ForegroundColor Blue
Write-Host "ghcr.io/cls3389/smart-forward:latest" -ForegroundColor Blue

Write-Host "`nRelease process started! Please wait for GitHub Actions to complete." -ForegroundColor Green
