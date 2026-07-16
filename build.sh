#!/usr/bin/env bash
set -euo pipefail

BUILD_TYPE="debug"
RUN_TESTS="false"
OUTPUT_DIR="output"

if [ "$#" -eq 0 ]; then
    echo "=== AIHack 빌드 스크립트 (인터랙티브 모드) ==="
    read -r -p "🚀 릴리스(Release) 모드로 최적화 빌드하시겠습니까? (y/N): " mode_ans
    if [[ "$mode_ans" =~ ^[Yy]$ ]]; then
        BUILD_TYPE="release"
    fi

    read -r -p "🧪 빌드 전 테스트를 실행하시겠습니까? (y/N): " test_ans
    if [[ "$test_ans" =~ ^[Yy]$ ]]; then
        RUN_TESTS="true"
    fi
else
    for arg in "$@"; do
        case "$arg" in
            --release) BUILD_TYPE="release" ;;
            --test) RUN_TESTS="true" ;;
            *)
                echo "알 수 없는 옵션: $arg" >&2
                echo "사용법: $0 [--release] [--test]" >&2
                exit 1
                ;;
        esac
    done
fi

if [ "$RUN_TESTS" = "true" ]; then
    cargo test --workspace --locked --all-targets
fi

if [ "$BUILD_TYPE" = "release" ]; then
    cargo build --workspace --locked --release --all-targets
    SOURCE_DIR="target/release"
else
    cargo build --workspace --locked --all-targets
    SOURCE_DIR="target/debug"
fi

suffix=""
case "$(uname -s)" in
    MINGW* | MSYS* | CYGWIN*) suffix=".exe" ;;
esac

mkdir -p "$OUTPUT_DIR"
for binary in "aihack${suffix}" "aihack-headless${suffix}"; do
    source_path="$SOURCE_DIR/$binary"
    destination_path="$OUTPUT_DIR/$binary"

    if [ ! -f "$source_path" ]; then
        echo "필수 artifact가 없습니다: $source_path" >&2
        exit 1
    fi

    cp "$source_path" "$destination_path"
    if [ ! -x "$destination_path" ]; then
        echo "artifact 검증에 실패했습니다: $destination_path" >&2
        exit 1
    fi
done

printf '빌드 완료: %s/%s, %s/%s\n' \
    "$OUTPUT_DIR" "aihack${suffix}" "$OUTPUT_DIR" "aihack-headless${suffix}"
