# Optimized Docker Build Script with Retry Mechanism
Write-Host "🐳 Building optimized Docker image..." -ForegroundColor Green

# Check if Docker is available
try {
    docker --version | Out-Null
    Write-Host "✅ Docker is available" -ForegroundColor Green
} catch {
    Write-Host "❌ Docker is not available" -ForegroundColor Red
    exit 1
}

# Function to build with retry
function Build-WithRetry {
    param(
        [int]$MaxRetries = 3,
        [int]$RetryDelay = 30
    )
    
    $retryCount = 0
    $success = $false
    
    while (-not $success -and $retryCount -lt $MaxRetries) {
        $retryCount++
        
        if ($retryCount -gt 1) {
            Write-Host "🔄 Retry attempt $retryCount of $MaxRetries..." -ForegroundColor Yellow
            Write-Host "⏳ Waiting $RetryDelay seconds before retry..." -ForegroundColor Yellow
            Start-Sleep -Seconds $RetryDelay
        }
        
        Write-Host "🔨 Starting build attempt $retryCount..." -ForegroundColor Yellow
        
        # Build the image
        docker build -t smart-forward:latest .
        
        if ($LASTEXITCODE -eq 0) {
            $success = $true
            Write-Host "✅ Build successful on attempt $retryCount!" -ForegroundColor Green
        } else {
            Write-Host "❌ Build failed on attempt $retryCount" -ForegroundColor Red
            
            if ($retryCount -lt $MaxRetries) {
                Write-Host "💡 Tip: This might be a network timeout issue. Retrying..." -ForegroundColor Cyan
            }
        }
    }
    
    return $success
}

# Build with retry mechanism
$buildSuccess = Build-WithRetry -MaxRetries 3 -RetryDelay 30

if ($buildSuccess) {
    Write-Host "✅ Build completed successfully!" -ForegroundColor Green
    
    # Show image information
    Write-Host "`n📊 Image information:" -ForegroundColor Cyan
    docker images smart-forward:latest
    
    Write-Host "`n💡 Usage:" -ForegroundColor Cyan
    Write-Host "  docker run -d --name smart-forward --network host smart-forward:latest" -ForegroundColor White
    
    Write-Host "`n🎯 Optimization features:" -ForegroundColor Cyan
    Write-Host "  - Alpine 3.18 base image" -ForegroundColor White
    Write-Host "  - Extreme compilation optimization (opt-level=z)" -ForegroundColor White
    Write-Host "  - Expected size: ~8MB" -ForegroundColor White
    
    # Test the image
    Write-Host "`n🧪 Testing image..." -ForegroundColor Cyan
    docker run --rm smart-forward:latest --version
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Image test passed!" -ForegroundColor Green
    } else {
        Write-Host "⚠️  Image test failed, but build was successful" -ForegroundColor Yellow
    }
    
} else {
    Write-Host "❌ All build attempts failed!" -ForegroundColor Red
    Write-Host "Possible causes:" -ForegroundColor Yellow
    Write-Host "  - Network connectivity issues" -ForegroundColor White
    Write-Host "  - Docker daemon problems" -ForegroundColor White
    Write-Host "  - Insufficient disk space" -ForegroundColor White
    Write-Host "  - Memory constraints" -ForegroundColor White
    
    Write-Host "`n💡 Troubleshooting tips:" -ForegroundColor Cyan
    Write-Host "  - Check your internet connection" -ForegroundColor White
    Write-Host "  - Restart Docker daemon" -ForegroundColor White
    Write-Host "  - Clean up Docker system: docker system prune" -ForegroundColor White
    Write-Host "  - Increase Docker resources in settings" -ForegroundColor White
    
    exit 1
}
