#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
PROVENANCE="$ROOT/PROVENANCE.md"
COMPAT_DIR="$ROOT/docs/compatibility"
CONTENT_CHECKSUMS="$ROOT/docs/provenance/r7-content.sha256"
ERRORS=()
HOLD_REASONS=()

error() {
    ERRORS+=("$1")
}

hold() {
    HOLD_REASONS+=("$1")
}

required_file() {
    [[ -f "$1" ]] || error "required file missing: ${1#"$ROOT/"}"
}

field_value() {
    local file=$1 pattern=$2
    local value
    value=$(sed -n "s/^[[:space:]]*$pattern:[[:space:]]*//p" "$file" | head -n 1)
    [[ "$value" == '""' ]] && value=
    printf '%s' "$value"
}

required_file "$PROVENANCE"
required_file "$ROOT/Cargo.toml"
required_file "$ROOT/tests/nethack_367_compat.rs"
required_file "$CONTENT_CHECKSUMS"

if ((${#ERRORS[@]} == 0)); then
    mapfile -t INVENTORY_ROWS < <(
        awk -F '|' '
            /<!-- runtime-inventory:start -->/ { active = 1; next }
            /<!-- runtime-inventory:end -->/ { active = 0 }
            active && $0 ~ /^\| PROV-/ {
                if (NF != 15) {
                    printf "INVALID_COLUMNS\t%s\n", $0
                    next
                }
                for (i = 2; i <= 14; i++) {
                    gsub(/^[[:space:]]+|[[:space:]]+$/, "", $i)
                    gsub(/`/, "", $i)
                }
                printf "%s", $2
                for (i = 3; i <= 14; i++) printf "\t%s", $i
                printf "\n"
            }
        ' "$PROVENANCE"
    )

    ((${#INVENTORY_ROWS[@]} > 0)) || error "runtime inventory is empty"
    declare -A INVENTORY_IDS=()
    RUNTIME_PATTERNS=()
    BLOCKED_PATTERNS=()

    for row in "${INVENTORY_ROWS[@]}"; do
        IFS=$'\t' read -r id path origin checksum status runtime reviewer reviewed_at \
            license_id license_scope notice_required modification_notice_required evidence <<<"$row"
        if [[ "$id" == "INVALID_COLUMNS" ]]; then
            error "runtime inventory must have 13 fields"
            continue
        fi
        [[ -z "${INVENTORY_IDS[$id]:-}" ]] || error "duplicate inventory id: $id"
        INVENTORY_IDS[$id]=1
        [[ "$status" =~ ^(Unknown|Reviewed|Approved|Blocked)$ ]] \
            || error "$id invalid status: $status"
        [[ "$runtime" =~ ^(yes|no)$ ]] || error "$id invalid runtime flag: $runtime"

        if [[ "$runtime" == yes ]]; then
            RUNTIME_PATTERNS+=("$path")
            if [[ "$status" != Approved ]]; then
                hold "$id runtime approval pending: $status"
            else
                [[ -n "$reviewer" ]] || error "$id Approved reviewer missing"
                [[ "$reviewed_at" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]] \
                    || error "$id Approved reviewed_at invalid"
                [[ -n "$license_id" && "$license_id" != pending ]] \
                    || error "$id Approved license_id missing"
                [[ -n "$license_scope" && "$license_scope" != pending ]] \
                    || error "$id Approved license_scope missing"
                [[ "$notice_required" =~ ^(true|false)$ ]] \
                    || error "$id Approved notice_required invalid"
                [[ "$modification_notice_required" =~ ^(true|false)$ ]] \
                    || error "$id Approved modification_notice_required invalid"
                [[ -n "$evidence" ]] || error "$id Approved evidence missing"
            fi
        fi
        if [[ "$status" =~ ^(Unknown|Blocked)$ ]]; then
            BLOCKED_PATTERNS+=("$path")
        fi
    done

    mapfile -t RUNTIME_FILES < <(
        cd "$ROOT"
        {
            [[ -f Cargo.lock ]] && printf '%s\n' Cargo.lock
            find src crates apps -type f \
                \( -name '*.rs' -o -name '*.toml' \) -print 2>/dev/null || true
        } | sort -u
    )
    for file in "${RUNTIME_FILES[@]}"; do
        best_length=-1
        best_count=0
        for pattern in "${RUNTIME_PATTERNS[@]}"; do
            if [[ "$file" == $pattern ]]; then
                length=${#pattern}
                if ((length > best_length)); then
                    best_length=$length
                    best_count=1
                elif ((length == best_length)); then
                    ((best_count += 1))
                fi
            fi
        done
        ((best_count == 1)) || error "runtime coverage must resolve once: $file ($best_count)"
    done

    checksum_manifest_valid=true
    while IFS= read -r line; do
        if [[ ! "$line" =~ ^[0-9a-f]{64}[[:space:]][[:space:]]crates/aihack-content/src/data/[^/[:space:]]+\.toml$ \
            && ! "$line" =~ ^[0-9a-f]{64}[[:space:]][[:space:]]crates/aihack-content/src/data/levels/[^/[:space:]]+\.toml$ ]]; then
            checksum_manifest_valid=false
        fi
    done <"$CONTENT_CHECKSUMS"
    if [[ "$checksum_manifest_valid" != true ]]; then
        error "runtime content checksum manifest format invalid"
    fi
    manifest_paths=$(awk '{print $2}' "$CONTENT_CHECKSUMS" | sort -u)
    actual_content_paths=$(
        cd "$ROOT"
        find crates/aihack-content/src/data -type f -name '*.toml' -print | sort
    )
    if [[ "$manifest_paths" != "$actual_content_paths" ]]; then
        error "runtime content checksum coverage incomplete"
    fi
    if ! (cd "$ROOT" && sha256sum --check --strict "${CONTENT_CHECKSUMS#"$ROOT/"}" >/dev/null); then
        error "runtime content checksum mismatch"
    fi

    for pattern in "${BLOCKED_PATTERNS[@]}"; do
        base=${pattern%%\**}
        base=${base%/}
        [[ -n "$base" ]] || continue
        if grep -R -n -F --include='*.toml' --include='*.rs' -- "$base" \
            "$ROOT/Cargo.toml" "$ROOT/src" "$ROOT/crates" "$ROOT/apps" >/dev/null 2>&1; then
            error "Blocked/Unknown runtime reference found: $base"
        fi
    done
fi

mapfile -t SCENARIO_FILES < <(find "$COMPAT_DIR" -maxdepth 1 -type f -name 'NH367-C*.md' -print 2>/dev/null | sort)
((${#SCENARIO_FILES[@]} == 10)) || error "compatibility record count must be 10"
declare -A SCENARIO_IDS=()
approved_scenarios=0

for file in "${SCENARIO_FILES[@]}"; do
    id=$(field_value "$file" id)
    status=$(field_value "$file" status)
    release=$(field_value "$file" release)
    archive_sha256=$(field_value "$file" archive_sha256)
    locator=$(field_value "$file" locator)
    provenance_status=$(field_value "$file" provenance_status)
    commands=$(field_value "$file" commands)
    events=$(field_value "$file" events)
    hash_fields=$(field_value "$file" hash_fields)
    module=$(field_value "$file" module)
    test_file=$(field_value "$file" file)
    function=$(field_value "$file" function)

    if [[ "$id" =~ ^NH367-C(00[1-9]|010)$ ]]; then
        [[ -z "${SCENARIO_IDS[$id]:-}" ]] || error "duplicate scenario id: $id"
        SCENARIO_IDS[$id]=1
    else
        error "invalid scenario id in ${file##*/}: $id"
        continue
    fi
    [[ "$status" =~ ^(Planned|Implemented|Verified|Blocked)$ ]] || error "$id invalid status"
    [[ "$release" == "NetHack 3.6.7" ]] || error "$id invalid release"
    [[ "$archive_sha256" =~ ^[0-9a-f]{64}$ ]] || error "$id archive checksum invalid"
    [[ -n "$locator" ]] || error "$id locator missing"
    [[ -n "$commands" ]] || error "$id commands missing"
    [[ -n "$events" ]] || error "$id events missing"
    [[ -n "$hash_fields" && "$hash_fields" != "[]" ]] || error "$id hash_fields missing"
    [[ -n "$module" ]] || error "$id module missing"
    [[ "$test_file" == "tests/nethack_367_compat.rs" ]] || error "$id test file invalid"
    [[ -n "$function" ]] || error "$id test function missing"
    function_count=$(grep -E -c "^fn ${function}\(" "$ROOT/tests/nethack_367_compat.rs" 2>/dev/null || true)
    [[ "$function_count" == 1 ]] || error "$id test function link invalid: $function"

    if [[ "$provenance_status" == Approved ]]; then
        ((approved_scenarios += 1))
        approval_reviewer=$(field_value "$file" approval_reviewer)
        approval_reviewed_at=$(field_value "$file" approval_reviewed_at)
        license_id=$(field_value "$file" license_id)
        license_scope=$(field_value "$file" license_scope)
        notice_required=$(field_value "$file" notice_required)
        modification_notice_required=$(field_value "$file" modification_notice_required)
        evidence=$(field_value "$file" evidence)
        [[ -n "$approval_reviewer" ]] || error "$id Approved reviewer missing"
        [[ "$approval_reviewed_at" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]] \
            || error "$id Approved reviewed_at invalid"
        [[ -n "$license_id" && "$license_id" != pending ]] || error "$id Approved license_id missing"
        [[ -n "$license_scope" && "$license_scope" != pending ]] || error "$id Approved license_scope missing"
        [[ "$notice_required" =~ ^(true|false)$ ]] || error "$id Approved notice_required invalid"
        [[ "$modification_notice_required" =~ ^(true|false)$ ]] \
            || error "$id Approved modification_notice_required invalid"
        [[ -n "$evidence" ]] || error "$id Approved evidence missing"
    elif [[ "$provenance_status" =~ ^(Unknown|Reviewed|Blocked)$ ]]; then
        hold "$id provenance approval pending: $provenance_status"
    else
        error "$id invalid provenance_status: $provenance_status"
    fi
done

for number in $(seq -w 1 10); do
    id="NH367-C0$number"
    [[ -n "${SCENARIO_IDS[$id]:-}" ]] || error "missing scenario id: $id"
done

if ((${#ERRORS[@]} > 0)); then
    printf '%s\n' 'R7 CHECKPOINT: FAIL'
    printf ' - %s\n' "${ERRORS[@]}"
    exit 2
fi

if ((${#HOLD_REASONS[@]} > 0)); then
    printf '%s\n' 'R7 CHECKPOINT: HOLD'
    printf ' - %s\n' "${HOLD_REASONS[@]}"
    printf 'approved compatibility records: %d/10\n' "$approved_scenarios"
    printf '%s\n' 'See PROVENANCE.md and docs/R7_COMPATIBILITY_REPORT.md.'
    exit 1
fi

printf '%s\n' 'R7 CHECKPOINT: PASS'
