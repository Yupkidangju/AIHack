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
if "!RUN_TESTS!"=="true" (
    cargo test --locked --all-targets
    if errorlevel 1 exit /b 1
)

if "!BUILD_TYPE!"=="release" (
    cargo build --locked --release --all-targets
    set "SOURCE_DIR=target\release"
) else (
    cargo build --locked --all-targets
    set "SOURCE_DIR=target\debug"
)
if errorlevel 1 exit /b 1

if not exist "%OUTPUT_DIR%" mkdir "%OUTPUT_DIR%"
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

echo 빌드 완료: %OUTPUT_DIR%\aihack.exe, %OUTPUT_DIR%\aihack-headless.exe
endlocal
