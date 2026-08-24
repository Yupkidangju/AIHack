[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$OutputDir,
    [Parameter(Mandatory = $true)]
    [string]$ExpectedCommit,
    [Parameter(Mandatory = $true)]
    [string]$ExpectedCandidateDate
)

$ErrorActionPreference = 'Stop'
$OwnerApprovalId = 'AIHACK-OWNER-2026-07-20-NGPL-01'
$ModificationNoticeId = 'AIHACK-MODIFICATIONS-2026-08-24-01'
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

function Assert-NoReparsePath([string]$Path) {
    $full = [System.IO.Path]::GetFullPath($Path)
    $root = [System.IO.Path]::GetPathRoot($full)
    $current = $root
    $relative = $full.Substring($root.Length)
    foreach ($component in $relative.Split([System.IO.Path]::DirectorySeparatorChar, [System.StringSplitOptions]::RemoveEmptyEntries)) {
        $current = Join-Path $current $component
        $item = Get-Item -Force -LiteralPath $current
        if (($item.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) {
            Fail "release output path contains a reparse component: $current"
        }
    }
    return $full
}

Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;

public static class AIHackFileIdentity {
    [StructLayout(LayoutKind.Sequential)]
    public struct FileInformation {
        public uint FileAttributes;
        public System.Runtime.InteropServices.ComTypes.FILETIME CreationTime;
        public System.Runtime.InteropServices.ComTypes.FILETIME LastAccessTime;
        public System.Runtime.InteropServices.ComTypes.FILETIME LastWriteTime;
        public uint VolumeSerialNumber;
        public uint FileSizeHigh;
        public uint FileSizeLow;
        public uint NumberOfLinks;
        public uint FileIndexHigh;
        public uint FileIndexLow;
    }

    [DllImport("kernel32.dll", SetLastError = true)]
    public static extern bool GetFileInformationByHandle(
        SafeFileHandle handle,
        out FileInformation information
    );
}
'@

function Get-HardLinkCount([string]$Path) {
    $stream = [System.IO.File]::Open($Path, [System.IO.FileMode]::Open, [System.IO.FileAccess]::Read, [System.IO.FileShare]::Read)
    try {
        $information = New-Object AIHackFileIdentity+FileInformation
        if (-not [AIHackFileIdentity]::GetFileInformationByHandle($stream.SafeFileHandle, [ref]$information)) {
            Fail "file identity query failed: $Path"
        }
        return [uint32]$information.NumberOfLinks
    }
    finally {
        $stream.Dispose()
    }
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

function ConvertFrom-CalendarDate([string]$Value, [string]$Label) {
    $parsed = [datetime]::MinValue
    $valid = [datetime]::TryParseExact(
        $Value,
        'yyyy-MM-dd',
        [System.Globalization.CultureInfo]::InvariantCulture,
        [System.Globalization.DateTimeStyles]::None,
        [ref]$parsed
    )
    if (-not $valid -or $parsed.ToString('yyyy-MM-dd', [System.Globalization.CultureInfo]::InvariantCulture) -cne $Value) {
        Fail "$Label is not a canonical Gregorian calendar date: $Value"
    }
    return $parsed
}

function Assert-CanonicalArchiveEntry([string]$Entry) {
    if ([string]::IsNullOrEmpty($Entry) -or $Entry.StartsWith('/') -or $Entry.Contains(':') -or $Entry.Contains('\') -or $Entry.Contains('//')) {
        Fail "source archive contains an unsafe path: $Entry"
    }
    $canonical = $Entry.TrimEnd('/')
    if ([string]::IsNullOrEmpty($canonical)) {
        Fail 'source archive contains an empty path'
    }
    $components = $canonical.Split('/')
    if ($components | Where-Object { [string]::IsNullOrEmpty($_) -or $_ -eq '.' -or $_ -eq '..' }) {
        Fail "source archive contains a non-canonical path: $Entry"
    }
    $canonicalComponents = foreach ($component in $components) {
        if ($component.EndsWith('.') -or $component.EndsWith(' ')) {
            Fail "source archive contains a Windows trailing-name alias: $Entry"
        }
        $baseName = $component.Split('.')[0]
        if ($baseName -match '^(CON|PRN|AUX|NUL|COM[1-9]|LPT[1-9])$') {
            Fail "source archive contains a Windows reserved device name: $Entry"
        }
        $component.ToLowerInvariant()
    }
    if (@('legacy_nethack_port_reference', 'target', 'output') -contains $canonicalComponents[0]) {
        Fail "release source archive contains an excluded path: $Entry"
    }
    return ($canonicalComponents -join '/')
}

$ResolvedOutput = Assert-NoReparsePath $OutputDir
$candidateDate = ConvertFrom-CalendarDate $ExpectedCandidateDate 'candidate date'
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
    if ((Get-HardLinkCount $path) -ne 1) {
        Fail "release artifact must have exactly one hard link: $name"
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
$archiveCanonicalEntries = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::Ordinal)
foreach ($entry in $archiveEntries) {
    $canonicalEntry = Assert-CanonicalArchiveEntry $entry
    if (-not $archiveCanonicalEntries.Add($canonicalEntry)) {
        Fail "source archive contains a Windows extraction collision: $entry"
    }
}

$ExpectedMetadata = @{
    product = 'AIHack'
    version = '0.3.0'
    commit = $ExpectedCommit
    candidate_date = $ExpectedCandidateDate
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
$periodPattern = '(?m)^Covered change period: `(?<start>[0-9]{4}-[0-9]{2}-[0-9]{2})\.\.(?<end>[0-9]{4}-[0-9]{2}-[0-9]{2})`$'
$periodMatches = [regex]::Matches($outputModifications, $periodPattern)
if ($periodMatches.Count -ne 1) {
    Fail 'MODIFICATIONS.md must contain one covered change period'
}
$periodStart = ConvertFrom-CalendarDate $periodMatches[0].Groups['start'].Value 'modification period start'
$periodEnd = ConvertFrom-CalendarDate $periodMatches[0].Groups['end'].Value 'modification period end'
if ($periodStart -gt $periodEnd) {
    Fail 'modification period start is after its end'
}
if ($candidateDate -lt $periodStart -or $candidateDate -gt $periodEnd) {
    Fail 'candidate date falls outside the modification period'
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
