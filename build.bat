@echo off
setlocal enabledelayedexpansion
chcp 65001 >nul

set BUILD_TYPE=debug
set RUN_TESTS=n
set OUTPUT_DIR=output

if "%~1"=="" goto interactive

:parse_args
if "%~1"=="" goto execute
if "%~1"=="--release" (
    set BUILD_TYPE=release
) else if "%~1"=="--test" (
    set RUN_TESTS=y
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
if /i "!mode_ans!"=="y" set BUILD_TYPE=release

set /p test_ans="🧪 빌드 전 테스트를 실행하시겠습니까? (y/N): "
if /i "!test_ans!"=="y" set RUN_TESTS=y

:execute
echo.
echo [1/4] 🛠️ 빌드 환경 준비 중...
if not exist "%OUTPUT_DIR%" mkdir "%OUTPUT_DIR%"

if /i "!RUN_TESTS!"=="y" (
    echo [2/4] 🧪 테스트 실행 중...
    cargo test
    if errorlevel 1 exit /b 1
) else (
    echo [2/4] ⏭️ 테스트 건너뜀.
)

echo [3/4] ⚙️ !BUILD_TYPE! 모드로 빌드 중...
if "!BUILD_TYPE!"=="release" (
    cargo build --release
    set SOURCE_DIR=target\release
) else (
    cargo build
    set SOURCE_DIR=target\debug
)
if errorlevel 1 exit /b 1

echo [4/4] 📦 결과물을 %OUTPUT_DIR% 디렉토리로 복사 중...
copy /y "!SOURCE_DIR!\aihack.exe" "%OUTPUT_DIR%\" >nul 2>&1
copy /y "!SOURCE_DIR!\aihack-headless.exe" "%OUTPUT_DIR%\" >nul 2>&1

echo ✅ 빌드가 완료되었습니다! 실행 파일이 .\%OUTPUT_DIR% 에 준비되었습니다.
endlocal
