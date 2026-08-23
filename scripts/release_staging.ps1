[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('New', 'Promote', 'Cleanup')]
    [string]$Mode,
    [Parameter(Mandatory = $true)]
    [string]$Root,
    [Parameter(Mandatory = $true)]
    [string]$OutputDir,
    [string]$Stage
)

$ErrorActionPreference = 'Stop'

function Fail([string]$Message) {
    throw $Message
}

function Assert-NoReparseDirectory([string]$Path) {
    $full = [System.IO.Path]::GetFullPath($Path)
    $item = Get-Item -Force -LiteralPath $full
    if (-not $item.PSIsContainer) {
        Fail "release path must be a directory: $full"
    }
    if (($item.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) {
        Fail "release path must not be a reparse point: $full"
    }
    return $full
}

function Assert-NoNestedReparse([string]$Path) {
    $pending = [System.Collections.Generic.Stack[string]]::new()
    $pending.Push($Path)
    while ($pending.Count -gt 0) {
        $directory = $pending.Pop()
        foreach ($entry in Get-ChildItem -Force -LiteralPath $directory) {
            if (($entry.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) {
                Fail "release output tree contains a reparse entry: $($entry.FullName)"
            }
            if ($entry.PSIsContainer) {
                $pending.Push($entry.FullName)
            }
        }
    }
}

$resolvedRoot = Assert-NoReparseDirectory $Root
$output = [System.IO.Path]::GetFullPath((Join-Path $resolvedRoot $OutputDir))
if ([System.IO.Path]::GetDirectoryName($output) -cne $resolvedRoot.TrimEnd('\')) {
    Fail 'release output must be a direct workspace child'
}

if ($Mode -eq 'New') {
    if ([System.IO.Directory]::Exists($output) -or [System.IO.File]::Exists($output)) {
        [void](Assert-NoReparseDirectory $output)
        Assert-NoNestedReparse $output
    }
    $stagePath = Join-Path $resolvedRoot ('.release-stage-' + [guid]::NewGuid().ToString('N'))
    [void][System.IO.Directory]::CreateDirectory($stagePath)
    [void](Assert-NoReparseDirectory $stagePath)
    Write-Output $stagePath
    exit 0
}

if ([string]::IsNullOrWhiteSpace($Stage)) {
    Fail 'release stage is required'
}
$stagePath = Assert-NoReparseDirectory $Stage
if ([System.IO.Path]::GetDirectoryName($stagePath) -cne $resolvedRoot.TrimEnd('\') -or
    -not [System.IO.Path]::GetFileName($stagePath).StartsWith('.release-stage-', [System.StringComparison]::Ordinal)) {
    Fail 'release stage must be a generated direct workspace child'
}

if ($Mode -eq 'Cleanup') {
    Remove-Item -LiteralPath $stagePath -Recurse -Force
    exit 0
}

$backup = $null
if ([System.IO.Directory]::Exists($output) -or [System.IO.File]::Exists($output)) {
    [void](Assert-NoReparseDirectory $output)
    Assert-NoNestedReparse $output
    $backup = Join-Path $resolvedRoot ('.release-old-' + [guid]::NewGuid().ToString('N'))
    [System.IO.Directory]::Move($output, $backup)
}

try {
    [System.IO.Directory]::Move($stagePath, $output)
}
catch {
    if ($null -ne $backup -and -not [System.IO.Directory]::Exists($output)) {
        [System.IO.Directory]::Move($backup, $output)
    }
    throw
}

if ($null -ne $backup) {
    $backupPath = [System.IO.Path]::GetFullPath($backup)
    if ([System.IO.Path]::GetDirectoryName($backupPath) -cne $resolvedRoot.TrimEnd('\') -or
        -not [System.IO.Path]::GetFileName($backupPath).StartsWith('.release-old-', [System.StringComparison]::Ordinal)) {
        Fail 'release backup cleanup boundary mismatch'
    }
    Remove-Item -LiteralPath $backupPath -Recurse -Force
}
