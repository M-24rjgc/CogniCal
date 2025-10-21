# Quick setup script for embedded Python
# Run this before building the application

Write-Host "Setting up embedded Python for CogniCal..." -ForegroundColor Cyan
Write-Host ""

# Check if already set up
if (Test-Path "resources/python/python.exe") {
    Write-Host "✓ Embedded Python already exists" -ForegroundColor Green
    $response = Read-Host "Do you want to reinstall? (y/N)"
    if ($response -ne "y" -and $response -ne "Y") {
        Write-Host "Skipping setup." -ForegroundColor Yellow
        exit 0
    }
    Write-Host "Removing existing Python..." -ForegroundColor Yellow
    Remove-Item -Path "resources/python" -Recurse -Force
}

# Run the setup script
& ".\scripts\setup_embedded_python.ps1"

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "=== Next Steps ===" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Development:" -ForegroundColor Yellow
    Write-Host "  pnpm tauri dev" -ForegroundColor White
    Write-Host ""
    Write-Host "Production Build:" -ForegroundColor Yellow
    Write-Host "  pnpm tauri build" -ForegroundColor White
    Write-Host ""
    Write-Host "The embedded Python will be included in the build automatically." -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "Setup failed. Please check the errors above." -ForegroundColor Red
    exit 1
}
