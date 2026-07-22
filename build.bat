@echo off
setlocal EnableExtensions EnableDelayedExpansion
chcp 65001 >nul

set "BUILD_TYPE=debug"
set "RUN_TESTS=false"
set "OUTPUT_DIR=output"

if "%~1"=="" goto interactive

:parse_args
if "%~1"=="" goto execute
if "%~1"=="--release" (
    set "BUILD_TYPE=release"
) else if "%~1"=="--test" (
    set "RUN_TESTS=true"
) else (
    echo 알 수 없는 옵션: %~1
    echo 사용법: build.bat [--release] [--test]
    exit /b 1
)
shift
goto parse_args

:interactive
echo === AIHack 빌드 스크립트 (인터랙티브 모드) ===
set /p mode_ans="🚀 릴리스(Release) 모드로 최적화 빌드하시겠습니까? (y/N): "
if /i "!mode_ans!"=="y" set "BUILD_TYPE=release"

set /p test_ans="🧪 빌드 전 테스트를 실행하시겠습니까? (y/N): "
if /i "!test_ans!"=="y" set "RUN_TESTS=true"

:execute
if "!BUILD_TYPE!"=="release" (
    for /f "delims=" %%G in ('git status --porcelain --untracked-files^=normal') do (
        echo 릴리스 source와 binary 일치를 위해 clean Git worktree가 필요합니다.
        exit /b 1
    )
)

if "!RUN_TESTS!"=="true" (
    cargo test --workspace --locked --all-targets
    if errorlevel 1 exit /b 1
)

if "!BUILD_TYPE!"=="release" (
    cargo build --workspace --locked --release --all-targets
    set "SOURCE_DIR=target\release"
) else (
    cargo build --workspace --locked --all-targets
    set "SOURCE_DIR=target\debug"
)
if errorlevel 1 exit /b 1

if not exist "%OUTPUT_DIR%" mkdir "%OUTPUT_DIR%"
if errorlevel 1 exit /b 1

git show HEAD:LICENSE > "%OUTPUT_DIR%\LICENSE"
if errorlevel 1 exit /b 1
copy /y "NOTICE" "%OUTPUT_DIR%\NOTICE" >nul
if errorlevel 1 exit /b 1
copy /y "MODIFICATIONS.md" "%OUTPUT_DIR%\MODIFICATIONS.md" >nul
if errorlevel 1 exit /b 1
copy /y "PROJECT_OWNER_LICENSE_APPROVAL.md" "%OUTPUT_DIR%\PROJECT_OWNER_LICENSE_APPROVAL.md" >nul
if errorlevel 1 exit /b 1

for %%F in (aihack.exe aihack-headless.exe) do (
    if not exist "!SOURCE_DIR!\%%F" (
        echo 필수 artifact가 없습니다: !SOURCE_DIR!\%%F
        exit /b 1
    )
    copy /y "!SOURCE_DIR!\%%F" "%OUTPUT_DIR%\%%F" >nul
    if errorlevel 1 exit /b 1
    if not exist "%OUTPUT_DIR%\%%F" (
        echo artifact 검증에 실패했습니다: %OUTPUT_DIR%\%%F
        exit /b 1
    )
)

if "!BUILD_TYPE!"=="release" (
    for /f "delims=" %%G in ('git rev-parse HEAD') do set "RELEASE_COMMIT=%%G"
    >"%OUTPUT_DIR%\RELEASE-METADATA" echo product=AIHack
    >>"%OUTPUT_DIR%\RELEASE-METADATA" echo version=0.3.0
    >>"%OUTPUT_DIR%\RELEASE-METADATA" echo commit=!RELEASE_COMMIT!
    >>"%OUTPUT_DIR%\RELEASE-METADATA" echo source_license=NGPL
    >>"%OUTPUT_DIR%\RELEASE-METADATA" echo modification_notice=AIHACK-MODIFICATIONS-2026-07-20-01
    >>"%OUTPUT_DIR%\RELEASE-METADATA" echo owner_approval=AIHACK-OWNER-2026-07-20-NGPL-01
    git archive --format=zip --output="%OUTPUT_DIR%\aihack-0.3.0-source.zip" HEAD
    if errorlevel 1 exit /b 1
    if not exist "%OUTPUT_DIR%\aihack-0.3.0-source.zip" (
        echo 대응 소스 archive 검증에 실패했습니다: %OUTPUT_DIR%\aihack-0.3.0-source.zip
        exit /b 1
    )
    tar -tf "%OUTPUT_DIR%\aihack-0.3.0-source.zip" LICENSE NOTICE MODIFICATIONS.md PROJECT_OWNER_LICENSE_APPROVAL.md RELEASE-METADATA Cargo.toml >nul
    if errorlevel 1 exit /b 1
    powershell -NoProfile -Command "$archiveMetadata=@(& tar -xOf '%OUTPUT_DIR%\aihack-0.3.0-source.zip' RELEASE-METADATA); if($LASTEXITCODE -ne 0){exit 1}; $expected=[ordered]@{product='AIHack';version='0.3.0';commit='!RELEASE_COMMIT!';source_license='NGPL';modification_notice='AIHACK-MODIFICATIONS-2026-07-20-01';owner_approval='AIHACK-OWNER-2026-07-20-NGPL-01'}; function Assert-Metadata([string[]]$lines){foreach($key in $expected.Keys){$prefix=$key+'='; $matches=@($lines | Where-Object {$_.StartsWith($prefix,[System.StringComparison]::Ordinal)}); if($matches.Count -ne 1 -or $matches[0] -cne ($prefix+$expected[$key])){Write-Error ('invalid release metadata key: '+$key); exit 1}}}; Assert-Metadata (Get-Content -LiteralPath '%OUTPUT_DIR%\RELEASE-METADATA'); Assert-Metadata $archiveMetadata"
    if errorlevel 1 exit /b 1
    findstr /c:"Approval ID: `AIHACK-OWNER-2026-07-20-NGPL-01`" "%OUTPUT_DIR%\PROJECT_OWNER_LICENSE_APPROVAL.md" >nul
    if errorlevel 1 exit /b 1
    tar -xOf "%OUTPUT_DIR%\aihack-0.3.0-source.zip" PROJECT_OWNER_LICENSE_APPROVAL.md | findstr /c:"Approval ID: `AIHACK-OWNER-2026-07-20-NGPL-01`" >nul
    if errorlevel 1 exit /b 1
    findstr /c:"Notice ID: `AIHACK-MODIFICATIONS-2026-07-20-01`" "%OUTPUT_DIR%\MODIFICATIONS.md" >nul
    if errorlevel 1 exit /b 1
    tar -xOf "%OUTPUT_DIR%\aihack-0.3.0-source.zip" MODIFICATIONS.md | findstr /c:"Notice ID: `AIHACK-MODIFICATIONS-2026-07-20-01`" >nul
    if errorlevel 1 exit /b 1
    powershell -NoProfile -Command "$names=@('aihack.exe','aihack-headless.exe','LICENSE','NOTICE','MODIFICATIONS.md','PROJECT_OWNER_LICENSE_APPROVAL.md','RELEASE-METADATA','aihack-0.3.0-source.zip'); $lines=foreach($name in $names){$hash=(Get-FileHash -Algorithm SHA256 (Join-Path '%OUTPUT_DIR%' $name)).Hash.ToLower(); $hash+'  '+$name}; Set-Content -Encoding Ascii (Join-Path '%OUTPUT_DIR%' 'SHA256SUMS') $lines"
    if errorlevel 1 exit /b 1
)

echo 빌드 완료: %OUTPUT_DIR%\aihack.exe, %OUTPUT_DIR%\aihack-headless.exe
endlocal
