try {
    $response = Invoke-WebRequest -Uri "https://ghcr.io/v2/cls3389/smart-forward/manifests/v1.0.6" -Method Head
    Write-Host "Status Code: $($response.StatusCode)"
    if ($response.StatusCode -eq 200) {
        Write-Host "SUCCESS: Package is accessible" -ForegroundColor Green
    } else {
        Write-Host "WARNING: Unexpected status code" -ForegroundColor Yellow
    }
} catch {
    $statusCode = $_.Exception.Response.StatusCode.value__
    Write-Host "Status Code: $statusCode" -ForegroundColor Red
    
    if ($statusCode -eq 403) {
        Write-Host "ERROR: 403 Forbidden - Permission denied" -ForegroundColor Red
        Write-Host "Please check your GitHub repository permissions:" -ForegroundColor Yellow
        Write-Host "1. Settings > Actions > General > Workflow permissions" -ForegroundColor Yellow
        Write-Host "2. Select 'Read and write permissions'" -ForegroundColor Yellow
        Write-Host "3. Check package permissions in Settings > Packages" -ForegroundColor Yellow
    } elseif ($statusCode -eq 404) {
        Write-Host "INFO: Package not found - This is normal for first push" -ForegroundColor Blue
    } else {
        Write-Host "ERROR: $($_.Exception.Message)" -ForegroundColor Red
    }
}
