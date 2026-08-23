#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
OUTPUT_DIR=${1:-"$ROOT/output"}
EXPECTED_COMMIT=${2:-$(git -C "$ROOT" rev-parse HEAD)}
ARCHIVE="$OUTPUT_DIR/aihack-0.3.0-source.tar.gz"
OWNER_APPROVAL_ID="AIHACK-OWNER-2026-07-20-NGPL-01"
MODIFICATION_NOTICE_ID="AIHACK-MODIFICATIONS-2026-08-23-02"

fail() {
    printf '%s\n' "$1" >&2
    exit 1
}

require_text() {
    local content=$1 needle=$2 label=$3
    grep -Fq "$needle" <<<"$content" || fail "$label missing or mismatched: $needle"
}

require_metadata_value() {
    local content=$1 key=$2 expected=$3 label=$4 value
    if ! value=$(awk -v key="$key" '
        index($0, key "=") == 1 {
            count += 1
            value = substr($0, length(key) + 2)
        }
        END {
            if (count != 1) exit 2
            print value
        }
    ' <<<"$content"); then
        fail "$label $key must appear exactly once"
    fi
    [[ "$value" == "$expected" ]] \
        || fail "$label $key mismatched: expected $expected, got $value"
}

required=(
    aihack
    aihack-headless
    LICENSE
    NOTICE
    MODIFICATIONS.md
    PROJECT_OWNER_LICENSE_APPROVAL.md
    RELEASE-METADATA
    SHA256SUMS
    aihack-0.3.0-source.tar.gz
)
mapfile -d '' -t actual_entries < <(find "$OUTPUT_DIR" -mindepth 1 -maxdepth 1 -print0)
[[ "${#actual_entries[@]}" -eq "${#required[@]}" ]] \
    || fail 'release output entry count mismatch'
for path in "${actual_entries[@]}"; do
    name=${path##*/}
    [[ -f "$path" && ! -L "$path" ]] \
        || fail "release output contains a directory or symbolic link: $name"
    found=false
    for expected in "${required[@]}"; do
        if [[ "$name" == "$expected" ]]; then
            found=true
            break
        fi
    done
    [[ "$found" == true ]] || fail "unexpected release output entry: $name"
done

for file in "${required[@]}"; do
    [[ -s "$OUTPUT_DIR/$file" ]] || {
        printf 'release artifact missing or empty: %s\n' "$file" >&2
        exit 1
    }
done

for file in LICENSE NOTICE MODIFICATIONS.md PROJECT_OWNER_LICENSE_APPROVAL.md RELEASE-METADATA Cargo.toml; do
    tar -tzf "$ARCHIVE" "$file" >/dev/null
done

if tar -tzf "$ARCHIVE" | grep -Eq '^(legacy_nethack_port_reference|target|output)/'; then
    printf '%s\n' 'release source archive contains an excluded path' >&2
    exit 1
fi

archive_metadata=$(tar -xOzf "$ARCHIVE" RELEASE-METADATA)
archive_approval=$(tar -xOzf "$ARCHIVE" PROJECT_OWNER_LICENSE_APPROVAL.md)
archive_modifications=$(tar -xOzf "$ARCHIVE" MODIFICATIONS.md)
output_metadata=$(<"$OUTPUT_DIR/RELEASE-METADATA")
output_approval=$(<"$OUTPUT_DIR/PROJECT_OWNER_LICENSE_APPROVAL.md")
output_modifications=$(<"$OUTPUT_DIR/MODIFICATIONS.md")

for metadata_label in archive output; do
    if [[ "$metadata_label" == archive ]]; then
        metadata=$archive_metadata
    else
        metadata=$output_metadata
    fi
    require_metadata_value "$metadata" product AIHack "$metadata_label RELEASE-METADATA"
    require_metadata_value "$metadata" version 0.3.0 "$metadata_label RELEASE-METADATA"
    require_metadata_value "$metadata" commit "$EXPECTED_COMMIT" "$metadata_label RELEASE-METADATA"
    require_metadata_value "$metadata" source_license NGPL "$metadata_label RELEASE-METADATA"
    require_metadata_value "$metadata" owner_approval "$OWNER_APPROVAL_ID" "$metadata_label RELEASE-METADATA"
    require_metadata_value "$metadata" modification_notice "$MODIFICATION_NOTICE_ID" "$metadata_label RELEASE-METADATA"
done
require_text "$archive_approval" "Approval ID: \`$OWNER_APPROVAL_ID\`" "archive Approval ID"
require_text "$output_approval" "Approval ID: \`$OWNER_APPROVAL_ID\`" "output Approval ID"
require_text "$archive_modifications" "Notice ID: \`$MODIFICATION_NOTICE_ID\`" "archive Notice ID"
require_text "$output_modifications" "Notice ID: \`$MODIFICATION_NOTICE_ID\`" "output Notice ID"

[[ "$archive_approval" == "$output_approval" ]] \
    || fail 'PROJECT_OWNER_LICENSE_APPROVAL.md differs between output and source archive'
[[ "$archive_modifications" == "$output_modifications" ]] \
    || fail 'MODIFICATIONS.md differs between output and source archive'
if grep -Fq '$Format:' <<<"$archive_metadata"; then
    printf '%s\n' 'release metadata export substitution did not run' >&2
    exit 1
fi

(
    cd "$OUTPUT_DIR"
    sha256sum --check --strict SHA256SUMS >/dev/null
)

printf 'PASS release bundle: version=0.3.0 commit=%s\n' "$EXPECTED_COMMIT"
