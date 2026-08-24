#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
OUTPUT_DIR=${1:-"$ROOT/output"}
EXPECTED_COMMIT=${2:-$(git -C "$ROOT" rev-parse HEAD)}
EXPECTED_CANDIDATE_DATE=${3:-$(git -C "$ROOT" show -s --format=%cs "$EXPECTED_COMMIT")}
ARCHIVE="$OUTPUT_DIR/aihack-0.3.0-source.tar.gz"
OWNER_APPROVAL_ID="AIHACK-OWNER-2026-07-20-NGPL-01"
MODIFICATION_NOTICE_ID="AIHACK-MODIFICATIONS-2026-08-24-01"

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

require_calendar_date() {
    local value=$1 label=$2 parsed year
    [[ "$value" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]] \
        || fail "$label must use YYYY-MM-DD"
    year=${value%%-*}
    ((10#$year >= 1 && 10#$year <= 9999)) \
        || fail "$label year must be in 0001..9999"
    parsed=$(date -u -d "$value" '+%F' 2>/dev/null) \
        || fail "$label is not a Gregorian calendar date"
    [[ "$parsed" == "$value" ]] \
        || fail "$label is not a canonical Gregorian calendar date"
}

validate_archive_entry() {
    local entry=$1 canonical component lowered basename first='' key=''
    [[ -n "$entry" && "$entry" != /* && "$entry" != *:* && "$entry" != *\\* && "$entry" != *//* ]] \
        || fail "source archive contains an unsafe path: $entry"
    canonical=${entry%/}
    [[ -n "$canonical" ]] || fail 'source archive contains an empty path'
    IFS='/' read -r -a components <<<"$canonical"
    for component in "${components[@]}"; do
        [[ -n "$component" && "$component" != '.' && "$component" != '..' ]] \
            || fail "source archive contains a non-canonical path: $entry"
        [[ ! "$component" =~ [\.\ ]$ ]] \
            || fail "source archive contains a Windows trailing-name alias: $entry"
        lowered=$(tr '[:upper:]' '[:lower:]' <<<"$component")
        basename=${lowered%%.*}
        case "$basename" in
            con|prn|aux|nul|com[1-9]|lpt[1-9])
                fail "source archive contains a Windows reserved device name: $entry"
                ;;
        esac
        [[ -n "$first" ]] || first=$lowered
        key+="${key:+/}$lowered"
    done
    case "$first" in
        legacy_nethack_port_reference|target|output)
            fail "release source archive contains an excluded path: $entry"
            ;;
    esac
    ARCHIVE_CANONICAL_KEY=$key
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
[[ -d "$OUTPUT_DIR" && ! -L "$OUTPUT_DIR" ]] \
    || fail 'release output root must be a real directory'
lexical_output=$(realpath -sm -- "$OUTPUT_DIR")
physical_output=$(realpath -e -- "$OUTPUT_DIR")
[[ "$lexical_output" == "$physical_output" ]] \
    || fail 'release output root must not traverse a symbolic link'
require_calendar_date "$EXPECTED_CANDIDATE_DATE" 'candidate date'
for file in "${required[@]}"; do
    [[ -s "$OUTPUT_DIR/$file" ]] || {
        printf 'release artifact missing or empty: %s\n' "$file" >&2
        exit 1
    }
    [[ $(stat -c '%h' -- "$OUTPUT_DIR/$file") == 1 ]] \
        || fail "release artifact must have exactly one hard link: $file"
done
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

for file in LICENSE NOTICE MODIFICATIONS.md PROJECT_OWNER_LICENSE_APPROVAL.md RELEASE-METADATA Cargo.toml; do
    tar -tzf "$ARCHIVE" "$file" >/dev/null
done

archive_listing=$(tar -tzf "$ARCHIVE") \
    || fail 'source archive listing failed'
declare -A archive_canonical_entries=()
while IFS= read -r archive_entry; do
    validate_archive_entry "$archive_entry"
    [[ -z ${archive_canonical_entries["$ARCHIVE_CANONICAL_KEY"]+present} ]] \
        || fail "source archive contains a Windows extraction collision: $archive_entry"
    archive_canonical_entries["$ARCHIVE_CANONICAL_KEY"]=1
done <<<"$archive_listing"

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
    require_metadata_value "$metadata" candidate_date "$EXPECTED_CANDIDATE_DATE" "$metadata_label RELEASE-METADATA"
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
period=$(sed -n 's/^Covered change period: `\([0-9][0-9-]*\)\.\.\([0-9][0-9-]*\)`$/\1 \2/p' \
    <<<"$output_modifications")
[[ $(wc -l <<<"$period") -eq 1 ]] \
    || fail 'MODIFICATIONS.md must contain one covered change period'
read -r period_start period_end <<<"$period"
require_calendar_date "$period_start" 'modification period start'
require_calendar_date "$period_end" 'modification period end'
[[ "$period_start" < "$period_end" || "$period_start" == "$period_end" ]] \
    || fail 'modification period start is after its end'
[[ ("$period_start" < "$EXPECTED_CANDIDATE_DATE" || "$period_start" == "$EXPECTED_CANDIDATE_DATE") \
    && ("$EXPECTED_CANDIDATE_DATE" < "$period_end" || "$EXPECTED_CANDIDATE_DATE" == "$period_end") ]] \
    || fail 'candidate date falls outside the modification period'
if grep -Fq '$Format:' <<<"$archive_metadata"; then
    printf '%s\n' 'release metadata export substitution did not run' >&2
    exit 1
fi

(
    cd "$OUTPUT_DIR"
    sha256sum --check --strict SHA256SUMS >/dev/null
)

printf 'PASS release bundle: version=0.3.0 commit=%s\n' "$EXPECTED_COMMIT"
