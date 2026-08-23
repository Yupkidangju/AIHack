[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$OutputDir,
    [Parameter(Mandatory = $true)]
    [string]$ExpectedCommit
)

$ErrorActionPreference = 'Stop'
$OwnerApprovalId = 'AIHACK-OWNER-2026-07-20-NGPL-01'
$ModificationNoticeId = 'AIHACK-MODIFICATIONS-2026-08-23-02'
$ArchiveName = 'aihack-0.3.0-source.zip'
$ChecksumNames = @(
    'aihack.exe',
    'aihack-headless.exe',
    'LICENSE',
    'NOTICE',
    'MODIFICATIONS.md',
    'PROJECT_OWNER_LICENSE_APPROVAL.md',
    'RELEASE-METADATA',
    $ArchiveName
)

function Fail([string]$Message) {
    throw $Message
}

function Normalize-Text([string]$Text) {
    return (($Text -replace "`r`n", "`n") -replace "`r", "`n").TrimEnd("`n")
}

function Get-Sha256([string]$Path) {
    $stream = [System.IO.File]::OpenRead($Path)
    try {
        $algorithm = [System.Security.Cryptography.SHA256]::Create()
        try {
            return ([System.BitConverter]::ToString($algorithm.ComputeHash($stream))).Replace('-', '').ToLowerInvariant()
        }
        finally {
            $algorithm.Dispose()
        }
    }
    finally {
        $stream.Dispose()
    }
}

function Read-ArchiveText([string]$Archive, [string]$Name) {
    $lines = @(& tar -xOf $Archive $Name)
    if ($LASTEXITCODE -ne 0) {
        Fail "archive entry read failed: $Name"
    }
    return Normalize-Text ($lines -join "`n")
}

function Assert-Metadata([string]$Content, [hashtable]$Expected, [string]$Label) {
    $lines = (Normalize-Text $Content) -split "`n"
    foreach ($key in $Expected.Keys) {
        $prefix = $key + '='
        $matches = @($lines | Where-Object { $_.StartsWith($prefix, [System.StringComparison]::Ordinal) })
        if ($matches.Count -ne 1 -or $matches[0] -cne ($prefix + $Expected[$key])) {
            Fail "$Label metadata mismatch or duplicate key: $key"
        }
    }
}

$ResolvedOutput = (Resolve-Path -LiteralPath $OutputDir).Path
$Archive = Join-Path $ResolvedOutput $ArchiveName
$Required = @($ChecksumNames + 'SHA256SUMS')
foreach ($name in $Required) {
    $path = Join-Path $ResolvedOutput $name
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        Fail "release artifact missing: $name"
    }
    if ((Get-Item -LiteralPath $path).Length -le 0) {
        Fail "release artifact is empty: $name"
    }
}
$ActualEntries = @(Get-ChildItem -LiteralPath $ResolvedOutput -Force)
if ($ActualEntries.Count -ne $Required.Count) {
    Fail 'release output entry count mismatch'
}
foreach ($entry in $ActualEntries) {
    if ($entry.PSIsContainer -or (($entry.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0)) {
        Fail "release output contains a directory or reparse entry: $($entry.Name)"
    }
    if ($Required -cnotcontains $entry.Name) {
        Fail "unexpected release output entry: $($entry.Name)"
    }
}

$archiveEntries = @(& tar -tf $Archive)
if ($LASTEXITCODE -ne 0) {
    Fail 'source archive listing failed'
}
foreach ($name in @('LICENSE', 'NOTICE', 'MODIFICATIONS.md', 'PROJECT_OWNER_LICENSE_APPROVAL.md', 'RELEASE-METADATA', 'Cargo.toml')) {
    if ($archiveEntries -cnotcontains $name) {
        Fail "source archive required entry missing: $name"
    }
}
if ($archiveEntries | Where-Object { $_ -match '^(legacy_nethack_port_reference|target|output)(/|$)' }) {
    Fail 'release source archive contains an excluded path'
}

$ExpectedMetadata = @{
    product = 'AIHack'
    version = '0.3.0'
    commit = $ExpectedCommit
    source_license = 'NGPL'
    modification_notice = $ModificationNoticeId
    owner_approval = $OwnerApprovalId
}
$outputMetadata = [System.IO.File]::ReadAllText((Join-Path $ResolvedOutput 'RELEASE-METADATA'))
$archiveMetadata = Read-ArchiveText $Archive 'RELEASE-METADATA'
Assert-Metadata $outputMetadata $ExpectedMetadata 'output RELEASE-METADATA'
Assert-Metadata $archiveMetadata $ExpectedMetadata 'archive RELEASE-METADATA'

$outputApproval = Normalize-Text ([System.IO.File]::ReadAllText((Join-Path $ResolvedOutput 'PROJECT_OWNER_LICENSE_APPROVAL.md')))
$archiveApproval = Read-ArchiveText $Archive 'PROJECT_OWNER_LICENSE_APPROVAL.md'
$outputModifications = Normalize-Text ([System.IO.File]::ReadAllText((Join-Path $ResolvedOutput 'MODIFICATIONS.md')))
$archiveModifications = Read-ArchiveText $Archive 'MODIFICATIONS.md'
if (-not $outputApproval.Contains("Approval ID: ``$OwnerApprovalId``") -or -not $archiveApproval.Contains("Approval ID: ``$OwnerApprovalId``")) {
    Fail 'Approval ID missing or mismatched'
}
if (-not $outputModifications.Contains("Notice ID: ``$ModificationNoticeId``") -or -not $archiveModifications.Contains("Notice ID: ``$ModificationNoticeId``")) {
    Fail 'Notice ID missing or mismatched'
}
if ($outputApproval -cne $archiveApproval) {
    Fail 'PROJECT_OWNER_LICENSE_APPROVAL.md differs between output and source archive'
}
if ($outputModifications -cne $archiveModifications) {
    Fail 'MODIFICATIONS.md differs between output and source archive'
}

$checksumLines = @(Get-Content -LiteralPath (Join-Path $ResolvedOutput 'SHA256SUMS'))
if ($checksumLines.Count -ne $ChecksumNames.Count) {
    Fail 'SHA256SUMS record count mismatch'
}
$records = @{}
foreach ($line in $checksumLines) {
    if ($line -cnotmatch '^([0-9a-f]{64})  (.+)$') {
        Fail "invalid SHA256SUMS record: $line"
    }
    $hash = $Matches[1]
    $name = $Matches[2]
    if ($records.ContainsKey($name)) {
        Fail "duplicate SHA256SUMS record: $name"
    }
    if ($ChecksumNames -cnotcontains $name) {
        Fail "unexpected SHA256SUMS record: $name"
    }
    $records[$name] = $hash
}
foreach ($name in $ChecksumNames) {
    if (-not $records.ContainsKey($name)) {
        Fail "missing SHA256SUMS record: $name"
    }
    $actual = Get-Sha256 (Join-Path $ResolvedOutput $name)
    if ($records[$name] -cne $actual) {
        Fail "SHA256 mismatch: $name"
    }
}

Write-Output "PASS Windows release bundle: version=0.3.0 commit=$ExpectedCommit"
