#!/bin/bash
set -e

# 기본값
BUILD_TYPE="debug"
RUN_TESTS="n"
OUTPUT_DIR="output"

# 인터랙티브 모드인지 파라미터가 있는지 확인
if [ $# -eq 0 ]; then
    echo "=== AIHack 빌드 스크립트 (인터랙티브 모드) ==="
    read -p "🚀 릴리스(Release) 모드로 최적화 빌드하시겠습니까? (y/N): " mode_ans
    if [[ "$mode_ans" =~ ^[Yy]$ ]]; then
        BUILD_TYPE="release"
    fi
    
    read -p "🧪 빌드 전 테스트를 실행하시겠습니까? (y/N): " test_ans
    if [[ "$test_ans" =~ ^[Yy]$ ]]; then
        RUN_TESTS="y"
    fi
else
    echo "=== AIHack 빌드 스크립트 (명령형 모드) ==="
    for arg in "$@"; do
        case $arg in
            --release)
                BUILD_TYPE="release"
                ;;
            --test)
                RUN_TESTS="y"
                ;;
            *)
                echo "알 수 없는 옵션: $arg"
                echo "사용법: $0 [--release] [--test]"
                exit 1
                ;;
        esac
    done
fi

echo ""
echo "[1/4] 🛠️ 빌드 환경 준비 중..."
mkdir -p "$OUTPUT_DIR"

if [ "$RUN_TESTS" = "y" ]; then
    echo "[2/4] 🧪 테스트 실행 중..."
    cargo test
else
    echo "[2/4] ⏭️ 테스트 건너뜀."
fi

echo "[3/4] ⚙️ $BUILD_TYPE 모드로 빌드 중..."
if [ "$BUILD_TYPE" = "release" ]; then
    cargo build --release
    SOURCE_DIR="target/release"
else
    cargo build
    SOURCE_DIR="target/debug"
fi

echo "[4/4] 📦 결과물을 $OUTPUT_DIR 디렉토리로 복사 중..."
cp "$SOURCE_DIR/aihack" "$OUTPUT_DIR/" 2>/dev/null || true
cp "$SOURCE_DIR/aihack-headless" "$OUTPUT_DIR/" 2>/dev/null || true

# Windows cross-compilation이나 WSL 호환을 위해 .exe도 복사
cp "$SOURCE_DIR/aihack.exe" "$OUTPUT_DIR/" 2>/dev/null || true
cp "$SOURCE_DIR/aihack-headless.exe" "$OUTPUT_DIR/" 2>/dev/null || true

echo "✅ 빌드가 완료되었습니다! 실행 파일이 ./$OUTPUT_DIR 에 준비되었습니다."
