# Simple GitHub Setup Script
param(
    [string]$RepoName = "smart-forward"
)

Write-Host "GitHub Repository Setup" -ForegroundColor Cyan
Write-Host "Repository: $RepoName" -ForegroundColor Yellow

# 1. Check Git status
Write-Host "`nChecking Git status..." -ForegroundColor Cyan

if (-not (Test-Path ".git")) {
    Write-Host "Initializing Git repository..." -ForegroundColor Yellow
    git init
    git add .
    git commit -m "Initial commit: Smart Forward project setup"
} else {
    Write-Host "Git repository exists" -ForegroundColor Green
}

# 2. Create .gitignore
Write-Host "`nCreating .gitignore..." -ForegroundColor Cyan

$gitignore = @"
# Rust
/target/
**/*.rs.bk
Cargo.lock

# IDE
.vscode/
.idea/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Logs
*.log
logs/

# Build artifacts
dist/
*.exe
*.dll
*.so
*.dylib

# Temporary files
*.tmp
*.temp
"@

$gitignore | Out-File -FilePath ".gitignore" -Encoding UTF8
Write-Host ".gitignore created" -ForegroundColor Green

# 3. Check Actions configuration
Write-Host "`nChecking GitHub Actions..." -ForegroundColor Cyan

$actionsDir = ".github/workflows"
if (Test-Path $actionsDir) {
    $workflows = Get-ChildItem $actionsDir -Filter "*.yml"
    Write-Host "Found workflow files:" -ForegroundColor Green
    foreach ($workflow in $workflows) {
        Write-Host "  - $($workflow.Name)" -ForegroundColor White
    }
} else {
    Write-Host "Actions directory not found" -ForegroundColor Red
}

# 4. Generate GitHub commands
Write-Host "`nGenerating GitHub commands..." -ForegroundColor Cyan

$commands = @"
# GitHub Repository Setup Commands

# 1. Create remote repository
gh repo create $RepoName --public --description "Smart Forward - Network forwarding tool"

# 2. Add remote origin
git remote add origin https://github.com/$env:USERNAME/$RepoName.git

# 3. Push code
git branch -M main
git push -u origin main

# 4. Create first tag
git tag v1.0.0
git push origin v1.0.0

# 5. View Actions
gh run list
"@

$commands | Out-File -FilePath "github-commands.txt" -Encoding UTF8
Write-Host "Commands saved to github-commands.txt" -ForegroundColor Green

# 5. Show next steps
Write-Host "`nNext steps:" -ForegroundColor Cyan
Write-Host "1. Install GitHub CLI: winget install GitHub.cli" -ForegroundColor Yellow
Write-Host "2. Login: gh auth login" -ForegroundColor Yellow
Write-Host "3. Run the commands from github-commands.txt" -ForegroundColor Yellow
Write-Host "4. Push your code to GitHub" -ForegroundColor Yellow
Write-Host "5. Check Actions status" -ForegroundColor Yellow

Write-Host "`nSetup completed!" -ForegroundColor Green
