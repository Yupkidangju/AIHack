#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
R7_CHECKPOINT="$ROOT/scripts/r7_checkpoint.sh"
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

package_field() {
    local manifest=$1 field=$2
    awk -v field="$field" '
        /^\[package\]$/ { package = 1; next }
        /^\[/ { package = 0 }
        package && $0 ~ "^[[:space:]]*" field "[[:space:]]*=" {
            value = $0
            sub(/^[^=]*=[[:space:]]*/, "", value)
            gsub(/^"|"$/, "", value)
            print value
            exit
        }
    ' "$manifest"
}

MANIFESTS=(
    Cargo.toml
    crates/aihack-core/Cargo.toml
    crates/aihack-content/Cargo.toml
    crates/aihack-ai-contract/Cargo.toml
    crates/aihack-llm/Cargo.toml
    crates/aihack-runtime/Cargo.toml
    apps/aihack-tui/Cargo.toml
    apps/aihack-headless/Cargo.toml
)

ARCHIVED_DOCUMENTS=(
    spec.md
    designs.md
    DESIGN_DECISIONS.md
    BUILD_GUIDE.md
    IMPLEMENTATION_SUMMARY.md
    GAP_CLOSURE_ROADMAP.md
    audit_roadmap.md
)

for path in \
    "$R7_CHECKPOINT" \
    "$ROOT/Cargo.lock" \
    "$ROOT/README.md" \
    "$ROOT/CHANGELOG.md" \
    "$ROOT/PROVENANCE.md" \
    "$ROOT/LICENSE" \
    "$ROOT/NOTICE" \
    "$ROOT/MODIFICATIONS.md" \
    "$ROOT/PROJECT_OWNER_LICENSE_APPROVAL.md" \
    "$ROOT/RELEASE-METADATA" \
    "$ROOT/.gitattributes" \
    "$ROOT/build.sh" \
    "$ROOT/build.bat" \
    "$ROOT/scripts/verify_release_bundle.sh"; do
    required_file "$path"
done
for manifest in "${MANIFESTS[@]}"; do
    required_file "$ROOT/$manifest"
done
for document in "${ARCHIVED_DOCUMENTS[@]}"; do
    required_file "$ROOT/$document"
done

if ((${#ERRORS[@]} == 0)); then
    set +e
    r7_output=$(bash "$R7_CHECKPOINT" 2>&1)
    r7_status=$?
    set -e
    case "$r7_status" in
        0) ;;
        1) hold "R7 approval checkpoint pending" ;;
        *) error "R7 approval checkpoint failed structurally (exit $r7_status)" ;;
    esac

    workspace_version=$(package_field "$ROOT/Cargo.toml" version)
    license_pending=false
    for manifest in "${MANIFESTS[@]}"; do
        version=$(package_field "$ROOT/$manifest" version)
        license=$(package_field "$ROOT/$manifest" license)
        if [[ "$version" != "$workspace_version" ]]; then
            error "workspace package version drift: $manifest is $version, root is $workspace_version"
        fi
        [[ "$license" == "NGPL" ]] || license_pending=true

        expected_dependency_version="version = \"$workspace_version\""
        while IFS= read -r dependency; do
            if [[ "$dependency" != *"$expected_dependency_version"* ]]; then
                error "path dependency version drift: $manifest: $dependency"
            fi
        done < <(sed -n '/{[[:space:]]*path[[:space:]]*=/p' "$ROOT/$manifest")
    done
    [[ "$workspace_version" == "0.3.0" ]] || hold "workspace release version must be 0.3.0"
    [[ "$license_pending" == false ]] || hold "workspace distribution license must be NGPL"

    license_sha256=$(tr -d '\r' <"$ROOT/LICENSE" | sha256sum | awk '{print $1}')
    if [[ "$license_sha256" != \
        '93a3ae2cb8dee482daddfaebe53bcffe5b114b603def19b4dca21621cbc5a747' ]]; then
        error "LICENSE must match the verified NetHack 3.6.7 dat/license text"
    fi
    for phrase in \
        'NetHack 3.6.7' \
        'derivative reimplementation' \
        'AI-assisted semantic rewrite' \
        'complete corresponding source' \
        'MODIFICATIONS.md' \
        'PROJECT_OWNER_LICENSE_APPROVAL.md' \
        'RELEASE-METADATA' \
        'AIHack contributors'; do
        grep -Fq -- "$phrase" "$ROOT/NOTICE" \
            || error "NOTICE required phrase missing: $phrase"
    done
    for script in build.sh build.bat; do
        for phrase in LICENSE NOTICE MODIFICATIONS.md PROJECT_OWNER_LICENSE_APPROVAL.md RELEASE-METADATA SHA256SUMS 'git status --porcelain' 'git archive' 'aihack-0.3.0-source' 'owner_approval=AIHACK-OWNER-2026-07-20-NGPL-01' 'modification_notice=AIHACK-MODIFICATIONS-2026-08-23-02'; do
            grep -Fq -- "$phrase" "$ROOT/$script" \
                || error "$script release packaging contract missing: $phrase"
        done
    done
    for path in legacy_nethack_port_reference target output; do
        grep -Fq -- "$path export-ignore" "$ROOT/.gitattributes" \
            || error ".gitattributes release exclusion missing: $path"
    done
    grep -Fq -- 'RELEASE-METADATA export-subst' "$ROOT/.gitattributes" \
        || error '.gitattributes release metadata substitution missing'
    for phrase in \
        'PROJECT_OWNER_LICENSE_APPROVAL.md' \
        'require_metadata_value' \
        'count != 1' \
        'owner_approval "$OWNER_APPROVAL_ID"' \
        'modification_notice "$MODIFICATION_NOTICE_ID"' \
        'Approval ID:' \
        'Notice ID:'; do
        grep -Fq -- "$phrase" "$ROOT/scripts/verify_release_bundle.sh" \
            || error "release verifier reference-integrity missing: $phrase"
    done
    for phrase in \
        'AIHACK-OWNER-2026-07-20-NGPL-01' \
        'PROV-0001..PROV-0012' \
        'NH367-C001..NH367-C010' \
        'qualified legal opinion: not claimed'; do
        grep -Fq -- "$phrase" "$ROOT/PROJECT_OWNER_LICENSE_APPROVAL.md" \
            || error "project-owner approval record missing: $phrase"
    done
    for phrase in \
        '2025-05-20..2026-08-23' \
        'does not depend on distributed Git history'; do
        grep -Fq -- "$phrase" "$ROOT/MODIFICATIONS.md" \
            || error "modification manifest missing: $phrase"
    done
    grep -Fq -- 'commit=$Format:%H$' "$ROOT/RELEASE-METADATA" \
        || error 'RELEASE-METADATA commit export placeholder missing'
    grep -Eq 'Current code: Cargo 0\.3\.0' "$ROOT/README.md" \
        || hold "README release version pending"
    grep -Eq '^## \[0\.3\.0\] - [0-9]{4}-[0-9]{2}-[0-9]{2}$' "$ROOT/CHANGELOG.md" \
        || hold "CHANGELOG 0.3.0 release entry pending"
    grep -Eq '외부 배포 판정: APPROVED' "$ROOT/PROVENANCE.md" \
        || hold "external distribution approval pending"

    for document in "${ARCHIVED_DOCUMENTS[@]}"; do
        latest=$(sed -n 's/^> - Latest: `\([^`]*\)`.*/\1/p' "$ROOT/$document" | head -n 1)
        if [[ -z "$latest" ]]; then
            error "archive chain Latest target missing: $document"
            continue
        fi
        if [[ "$latest" == /* || "$latest" == *'..'* || "$latest" != .archive/* ]]; then
            error "archive target escapes repository archive: $document: $latest"
            continue
        fi
        [[ -s "$ROOT/$latest" ]] || error "archive target missing or empty: $document -> $latest"
    done
fi

if ((${#ERRORS[@]} > 0)); then
    printf '%s\n' 'R8 CHECKPOINT: FAIL'
    printf ' - %s\n' "${ERRORS[@]}"
    exit 2
fi

if ((${#HOLD_REASONS[@]} > 0)); then
    printf '%s\n' 'R8 CHECKPOINT: HOLD'
    printf ' - %s\n' "${HOLD_REASONS[@]}"
    printf '%s\n' 'External distribution remains blocked.'
    exit 1
fi

printf '%s\n' 'R8 CHECKPOINT: PASS'
