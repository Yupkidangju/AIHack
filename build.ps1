<#
.SYNOPSIS
    NetHack-RS Build Script
    Supports Portable, Distribution, and All build modes.
    
.DESCRIPTION
    This script builds the NetHack-RS project and packages it according to the selected mode.
    It supports both interactive menu and command-line arguments.

.PARAMETER Portable
    Build and package for portable use (folder-based).

.PARAMETER Dist
    Build and package for distribution (zip-based).

.PARAMETER All
    Build both portable and distribution packages.
#>

param (
    [switch]$Portable,
    [switch]$Dist,
    [switch]$All
)

$ProjectRoot = Get-Location
$BuildDir = Join-Path $ProjectRoot "build"
$ReleaseExe = Join-Path $ProjectRoot "target\release\nethack-rs.exe"
$AssetsDir = Join-Path $ProjectRoot "assets"

function Show-Menu {
    Clear-Host
    Write-Host "==============================" -ForegroundColor Cyan
    Write-Host "   NetHack-RS Build System    " -ForegroundColor Cyan
    Write-Host "==============================" -ForegroundColor Cyan
    Write-Host "1. Portable Build (Folder)"
    Write-Host "2. Distribution Build (Zip)"
    Write-Host "3. Build All"
    Write-Host "q. Quit"
    Write-Host ""
    $choice = Read-Host "Select an option [1-3, q]"
    return $choice
}

function Prepare-Release {
    Write-Host "[*] Building release binary..." -ForegroundColor Yellow
    cargo build --release
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Cargo build failed!"
        exit 1
    }
}

function Build-Portable {
    Write-Host "[*] Creating Portable Build..." -ForegroundColor Green
    $PortableDir = Join-Path $BuildDir "Portable"
    if (Test-Path $PortableDir) { Remove-Item -Recurse -Force $PortableDir }
    New-Item -ItemType Directory -Path $PortableDir | Out-Null
    
    Copy-Item $ReleaseExe -Destination $PortableDir
    Copy-Item -Recurse $AssetsDir -Destination (Join-Path $PortableDir "assets")
    
    Write-Host "[+] Portable build completed at: $PortableDir" -ForegroundColor Green
}

function Build-Dist {
    Write-Host "[*] Creating Distribution Build..." -ForegroundColor Green
    $DistDir = Join-Path $BuildDir "Dist"
    if (Test-Path $DistDir) { Remove-Item -Recurse -Force $DistDir }
    New-Item -ItemType Directory -Path $DistDir | Out-Null
    
    # Create a temporary folder to zip
    $TempZipDir = Join-Path $DistDir "nethack-rs-v1.0.3"
    New-Item -ItemType Directory -Path $TempZipDir | Out-Null
    
    Copy-Item $ReleaseExe -Destination $TempZipDir
    Copy-Item -Recurse $AssetsDir -Destination (Join-Path $TempZipDir "assets")
    
    $ZipFile = Join-Path $DistDir "nethack-rs-v1.0.3-windows-x64.zip"
    Compress-Archive -Path "$TempZipDir\*" -DestinationPath $ZipFile -Force
    
    Remove-Item -Recurse -Force $TempZipDir
    
    Write-Host "[+] Distribution build completed at: $ZipFile" -ForegroundColor Green
}

# --- Main Logic ---

if (-not (Test-Path $BuildDir)) { New-Item -ItemType Directory -Path $BuildDir | Out-Null }

if ($Portable -or $Dist -or $All) {
    # Command-line mode
    Prepare-Release
    if ($All) {
        Build-Portable
        Build-Dist
    } elseif ($Portable) {
        Build-Portable
    } elseif ($Dist) {
        Build-Dist
    }
} else {
    # Interactive mode
    while ($true) {
        $choice = Show-Menu
        switch ($choice) {
            "1" { Prepare-Release; Build-Portable; Read-Host "Press Enter to continue..." }
            "2" { Prepare-Release; Build-Dist; Read-Host "Press Enter to continue..." }
            "3" { Prepare-Release; Build-Portable; Build-Dist; Read-Host "Press Enter to continue..." }
            "q" { exit }
            default { Write-Host "Invalid option!" -ForegroundColor Red; Start-Sleep -Seconds 1 }
        }
    }
}
