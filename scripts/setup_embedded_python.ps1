# Setup Embedded Python for CogniCal
# This script downloads and configures a minimal Python runtime

param(
    [string]$PythonVersion = "3.11.9",
    [string]$TargetDir = "resources/python"
)

$ErrorActionPreference = "Stop"

Write-Host "=== CogniCal Embedded Python Setup ===" -ForegroundColor Cyan
Write-Host ""

# Create target directory
if (!(Test-Path $TargetDir)) {
    New-Item -ItemType Directory -Path $TargetDir -Force | Out-Null
    Write-Host "✓ Created directory: $TargetDir" -ForegroundColor Green
}

# Determine platform
$Platform = "windows-amd64"
if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") {
    $Platform = "windows-arm64"
}

Write-Host "Platform: $Platform" -ForegroundColor Yellow
Write-Host "Python Version: $PythonVersion" -ForegroundColor Yellow
Write-Host ""

# Download Python embeddable package
$PythonUrl = "https://www.python.org/ftp/python/$PythonVersion/python-$PythonVersion-embed-amd64.zip"
$ZipPath = "python-embed.zip"

Write-Host "Downloading Python embeddable package..." -ForegroundColor Yellow
try {
    Invoke-WebRequest -Uri $PythonUrl -OutFile $ZipPath -UseBasicParsing
    Write-Host "✓ Downloaded Python package" -ForegroundColor Green
} catch {
    Write-Host "✗ Failed to download Python: $_" -ForegroundColor Red
    exit 1
}

# Extract Python
Write-Host "Extracting Python..." -ForegroundColor Yellow
try {
    Expand-Archive -Path $ZipPath -DestinationPath $TargetDir -Force
    Remove-Item $ZipPath
    Write-Host "✓ Extracted Python to $TargetDir" -ForegroundColor Green
} catch {
    Write-Host "✗ Failed to extract: $_" -ForegroundColor Red
    exit 1
}

# Enable site-packages by modifying python311._pth
Write-Host "Configuring Python paths..." -ForegroundColor Yellow
$PthFile = Get-ChildItem -Path $TargetDir -Filter "python*._pth" | Select-Object -First 1
if ($PthFile) {
    $content = Get-Content $PthFile.FullName
    # Uncomment the import site line
    $content = $content -replace '#import site', 'import site'
    # Add Lib/site-packages
    $content += "`nLib/site-packages"
    Set-Content -Path $PthFile.FullName -Value $content
    Write-Host "✓ Configured Python paths" -ForegroundColor Green
} else {
    Write-Host "⚠ Warning: Could not find ._pth file" -ForegroundColor Yellow
}

# Download get-pip.py
Write-Host "Installing pip..." -ForegroundColor Yellow
$GetPipUrl = "https://bootstrap.pypa.io/get-pip.py"
$GetPipPath = Join-Path $TargetDir "get-pip.py"

try {
    Invoke-WebRequest -Uri $GetPipUrl -OutFile $GetPipPath -UseBasicParsing
    
    # Run get-pip.py
    $PythonExe = Join-Path $TargetDir "python.exe"
    & $PythonExe $GetPipPath --no-warn-script-location
    
    Remove-Item $GetPipPath
    Write-Host "✓ Installed pip" -ForegroundColor Green
} catch {
    Write-Host "✗ Failed to install pip: $_" -ForegroundColor Red
    exit 1
}

# Install required packages
Write-Host ""
Write-Host "Installing Python packages..." -ForegroundColor Yellow
$packages = @(

    "txtai"
)

foreach ($package in $packages) {
    Write-Host "  Installing $package..." -ForegroundColor Cyan
    try {
        & $PythonExe -m pip install $package --no-warn-script-location --quiet
        Write-Host "  ✓ Installed $package" -ForegroundColor Green
    } catch {
        $errorMsg = $_.Exception.Message
        Write-Host "  ✗ Failed to install ${package}: $errorMsg" -ForegroundColor Red
    }
}

# Verify installation
Write-Host ""
Write-Host "Verifying installation..." -ForegroundColor Yellow

# Test Python
$pythonVersion = & $PythonExe --version 2>&1
Write-Host "  Python: $pythonVersion" -ForegroundColor Green



# Calculate size
$size = (Get-ChildItem -Path $TargetDir -Recurse | Measure-Object -Property Length -Sum).Sum / 1MB
Write-Host ""
Write-Host "Total size: $([math]::Round($size, 2)) MB" -ForegroundColor Cyan

Write-Host ""
Write-Host "=== Setup Complete ===" -ForegroundColor Green
Write-Host ""
Write-Host "Embedded Python is ready at: $TargetDir" -ForegroundColor Cyan
Write-Host "You can now build the application with: pnpm tauri build" -ForegroundColor Cyan
