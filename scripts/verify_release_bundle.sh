#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
OUTPUT_DIR=${1:-"$ROOT/output"}
EXPECTED_COMMIT=${2:-$(git -C "$ROOT" rev-parse HEAD)}
ARCHIVE="$OUTPUT_DIR/aihack-0.3.0-source.tar.gz"
OWNER_APPROVAL_ID="AIHACK-OWNER-2026-07-20-NGPL-01"
MODIFICATION_NOTICE_ID="AIHACK-MODIFICATIONS-2026-07-20-01"

fail() {
    printf '%s\n' "$1" >&2
    exit 1
}

require_text() {
    local content=$1 needle=$2 label=$3
    grep -Fq "$needle" <<<"$content" || fail "$label missing or mismatched: $needle"
}

for file in \
    aihack \
    aihack-headless \
    LICENSE \
    NOTICE \
    MODIFICATIONS.md \
    PROJECT_OWNER_LICENSE_APPROVAL.md \
    RELEASE-METADATA \
    SHA256SUMS \
    aihack-0.3.0-source.tar.gz; do
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

require_text "$archive_metadata" "version=0.3.0" "archive RELEASE-METADATA version"
require_text "$archive_metadata" "commit=$EXPECTED_COMMIT" "archive RELEASE-METADATA commit"
require_text "$archive_metadata" "owner_approval=$OWNER_APPROVAL_ID" "archive RELEASE-METADATA owner_approval"
require_text "$archive_metadata" "modification_notice=$MODIFICATION_NOTICE_ID" "archive RELEASE-METADATA modification_notice"
require_text "$output_metadata" "commit=$EXPECTED_COMMIT" "output RELEASE-METADATA commit"
require_text "$output_metadata" "owner_approval=$OWNER_APPROVAL_ID" "output RELEASE-METADATA owner_approval"
require_text "$output_metadata" "modification_notice=$MODIFICATION_NOTICE_ID" "output RELEASE-METADATA modification_notice"
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
