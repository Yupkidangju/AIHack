#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
cd "$ROOT"

BUILD_TYPE="debug"
RUN_TESTS="false"
OUTPUT_DIR="output"
PACKAGE_DIR="$OUTPUT_DIR"
STAGING_DIR=""
BACKUP_DIR=""

cleanup_release_directories() {
    for path in "$STAGING_DIR"; do
        case "$path" in
            "$ROOT"/.release-stage.* | "$ROOT"/.release-old.*)
                if [[ -d "$path" && ! -L "$path" ]]; then
                    rm -rf -- "$path"
                fi
                ;;
        esac
    done
}

trap cleanup_release_directories EXIT

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

if [ "$BUILD_TYPE" = "release" ] && [ -n "$(git status --porcelain --untracked-files=normal)" ]; then
    echo "릴리스 source와 binary 일치를 위해 clean Git worktree가 필요합니다." >&2
    exit 1
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

if [ "$BUILD_TYPE" = "release" ]; then
    if [[ -e "$ROOT/$OUTPUT_DIR" || -L "$ROOT/$OUTPUT_DIR" ]]; then
        [[ -d "$ROOT/$OUTPUT_DIR" && ! -L "$ROOT/$OUTPUT_DIR" ]] \
            || { echo "release output root must be a real directory" >&2; exit 1; }
        lexical_output=$(realpath -sm -- "$ROOT/$OUTPUT_DIR")
        physical_output=$(realpath -e -- "$ROOT/$OUTPUT_DIR")
        [[ "$lexical_output" == "$physical_output" ]] \
            || { echo "release output root must not traverse a symbolic link" >&2; exit 1; }
    fi
    STAGING_DIR=$(mktemp -d "$ROOT/.release-stage.XXXXXXXX")
    PACKAGE_DIR="$STAGING_DIR"
else
    mkdir -p "$OUTPUT_DIR"
fi

cp LICENSE NOTICE MODIFICATIONS.md PROJECT_OWNER_LICENSE_APPROVAL.md "$PACKAGE_DIR/"
for binary in "aihack${suffix}" "aihack-headless${suffix}"; do
    source_path="$SOURCE_DIR/$binary"
    destination_path="$PACKAGE_DIR/$binary"

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

if [ "$BUILD_TYPE" = "release" ]; then
    release_commit=$(git rev-parse HEAD)
    candidate_date=$(git show -s --format=%cs HEAD)
    printf 'product=AIHack\nversion=0.3.0\ncommit=%s\ncandidate_date=%s\nsource_license=NGPL\nmodification_notice=AIHACK-MODIFICATIONS-2026-08-25-01\nowner_approval=AIHACK-OWNER-2026-07-20-NGPL-01\n' \
        "$release_commit" "$candidate_date" >"$PACKAGE_DIR/RELEASE-METADATA"
    source_archive="$PACKAGE_DIR/aihack-0.3.0-source.tar.gz"
    git archive --format=tar.gz --output="$source_archive" HEAD
    if [ ! -s "$source_archive" ]; then
        echo "대응 소스 archive 검증에 실패했습니다: $source_archive" >&2
        exit 1
    fi
    (
        cd "$PACKAGE_DIR"
        sha256sum \
            "aihack${suffix}" \
            "aihack-headless${suffix}" \
            LICENSE \
            NOTICE \
            MODIFICATIONS.md \
            PROJECT_OWNER_LICENSE_APPROVAL.md \
            RELEASE-METADATA \
            "${source_archive##*/}" >SHA256SUMS
    )
    "$ROOT/scripts/verify_release_bundle.sh" "$PACKAGE_DIR" "$release_commit" "$candidate_date" "$ROOT"

    if [[ -d "$ROOT/$OUTPUT_DIR" ]]; then
        BACKUP_DIR="$ROOT/.release-old.$RANDOM.$RANDOM"
        mv -- "$ROOT/$OUTPUT_DIR" "$BACKUP_DIR"
    fi
    if ! mv -- "$STAGING_DIR" "$ROOT/$OUTPUT_DIR"; then
        if [[ -n "$BACKUP_DIR" && -d "$BACKUP_DIR" && ! -e "$ROOT/$OUTPUT_DIR" ]]; then
            if mv -- "$BACKUP_DIR" "$ROOT/$OUTPUT_DIR"; then
                BACKUP_DIR=""
            else
                echo "기존 release output 복원에 실패했습니다: $BACKUP_DIR" >&2
            fi
        fi
        exit 1
    fi
    STAGING_DIR=""
    if [[ -n "$BACKUP_DIR" ]]; then
        rm -rf -- "$BACKUP_DIR"
        BACKUP_DIR=""
    fi
fi

printf '빌드 완료: %s/%s, %s/%s\n' \
    "$OUTPUT_DIR" "aihack${suffix}" "$OUTPUT_DIR" "aihack-headless${suffix}"
