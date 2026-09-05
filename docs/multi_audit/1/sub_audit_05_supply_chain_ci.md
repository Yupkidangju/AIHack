# Sub Audit Report

## 1. Audit Metadata

- Audit Turn: 1
- Perspective: 의존성·공급망·빌드·CI/CD·라이선스·릴리즈
- User Goal: 현재 프로젝트의 문서와 구현을 대조하여 모순과 문제를 진단하고 수정 가능한 감사 결과를 제공한다.
- Audit Basis: Standard-backed
- Standard Path: `C:\LocalDev\rust\AIHack\AI_AUDIT_DOC_STANDARD.md`; `C:\Users\temp\.codex\skills\multi-audit\references\report-contract.md`
- Repository: `C:\LocalDev\rust\AIHack`
- Audited HEAD: `80d959af94cb08c5d9b2f2601f5e63f3827a1210` (`codex/audit-report-24-remediation`)
- Git evidence: local branch는 `origin/codex/audit-report-24-remediation`과 동일 SHA이며 `origin/HEAD`는 `main`의 `41a1b63f11a57a671b0f705883431dab24298b5a`를 가리킨다.
- Report scope: 공급망·의존성 graph·toolchain/MSRV·license 정책·빌드 및 CI gate·R7/R8 checkpoint·Linux/Windows release bundle·provenance/checksum·현재 HEAD/원격 CI 증거

## 2. Assigned Scope

다음 실제 산출물을 읽고 매니페스트와 lock graph, 문서 주장, 실행 gate를 대조했다.

- 모든 활성 Cargo 매니페스트 8개와 `Cargo.lock`
- `deny.toml`, `rust-toolchain.toml`, `.gitattributes`, `.gitignore`
- `.github/workflows/ci.yml`
- `build.sh`, `build.bat`, `scripts/r7_checkpoint.sh`, `scripts/r8_checkpoint.sh`, `scripts/verify_release_bundle.sh`
- `README.md`, `BUILD_GUIDE.md`, `PROVENANCE.md`, `PROJECT_OWNER_LICENSE_APPROVAL.md`, `MODIFICATIONS.md`, `RELEASE-METADATA`, `CHANGELOG.md`
- 공급망·릴리즈 관련 회귀 테스트: `tests/build_contract.rs`, `tests/provenance_manifest.rs`, `tests/license_compliance.rs`, `tests/release_gate.rs`, `tests/release_bundle.rs`, `tests/r8_documentation.rs`
- 로컬 Git 상태·log·remote ref와 GitHub Actions read-only API/log (`gh run view`, `gh run list`)

검사한 핵심 질문은 다음과 같다.

1. 선언된 dependency graph, lockfile, source policy, wildcard/duplicate/MSRV/license exception이 실제 resolved graph와 일치하는가?
2. RustSec advisory와 cargo-deny gate가 현재 graph를 실제로 닫는가?
3. CI 명령이 local 문서와 동일한 locked gate를 실행하고 Linux/Windows에서 같은 release contract를 검증하는가?
4. R7/R8와 release script가 provenance, archive, checksum, source commit을 실패 시 중단하는가?
5. 현재 HEAD와 문서에 기록된 CI SHA/run이 서로 추적 가능한가?

## 3. Excluded and Uninspected Scope

- 사용자가 지정한 독립성 조건에 따라 `docs/multi_audit/1/`의 다른 sub-audit 보고서는 읽지 않았다. 이 보고서는 단독 증거로 판정했다.
- `target/`, generated runtime data, `legacy_nethack_port_reference/**`의 내부 구현 및 과거 참조 tree는 shipped dependency/release graph가 아니므로 상세 내용 검사를 제외했다. 다만 release archive에서 해당 tree가 제외되는지와 활성 runtime 코드의 직접 참조 여부는 script/test로 확인했다.
- 법률 자문, NetHack 라이선스의 최종 법적 해석, 외부 게시·release mutation·commit/push는 범위 밖이다. 문서는 engineering evidence로만 평가했다.
- 로컬 환경에는 `cargo-deny` subcommand가 설치되어 있지 않아 로컬에서 `cargo deny`를 재실행하지 못했다. 대신 동일 commit의 원격 CI 양 OS에서 cargo-deny 0.19.4 step이 success인 read-only 증거를 사용했다.
- 현재 worktree에는 감사 보고서 디렉터리가 untracked 상태라 로컬 release script의 clean-tree 조건을 의도적으로 통과시키지 않았다.
- 외부 release artifact가 CI에서 장기 보관되었는지는 확인하지 못했다. Actions 로그와 job status만 확인했다.

## 4. Evidence Examined

### 4.1 Dependency graph, lockfile, toolchain

- `cargo metadata --locked --format-version 1`: workspace member 8개, resolved package 304개. source는 crates.io registry 296개와 local path package 8개이며 git source는 0개다. 모든 workspace package는 version `0.3.0`, `rust-version = 1.94`다.
- `Cargo.lock:1-4, 20-95`: lockfile format 4이며 workspace package와 direct dependency edge가 매니페스트와 일치한다. `cargo metadata --locked`와 `git diff --exit-code -- Cargo.lock` 모두 성공했다.
- `rust-toolchain.toml:1-4`: channel `1.94.1`, profile minimal, rustfmt/clippy 고정. 실제 `rustc --version`도 `1.94.1`, `cargo --version`도 `1.94.1`이다.
- 모든 활성 매니페스트에 `rust-version = "1.94"`, `license = "NGPL"`, `publish = false`가 있고 내부 path dependency에는 `version = "0.3.0"`가 있다. literal `*` dependency는 검색되지 않았다.
- `cargo tree --locked --target all -d`: `getrandom` 0.2/0.3/0.4, `rand` 0.8/0.9, `hashbrown` 0.16/0.17, `thiserror` 1/2, `windows-sys` 0.59/0.60/0.61 등 중복 family가 존재한다. `crossterm`은 0.29.0 단일 버전이다. `deny.toml:13-16`의 `multiple-versions = "allow"` 및 crossterm 한정 duplicate deny와 실제 graph는 일치한다.
- `Cargo.lock`의 registry source line은 296개이고 git source line은 0개다. `deny.toml:18-21`의 unknown registry/git deny 및 crates.io registry allow 정책과 일치한다.

### 4.2 Advisory/license scanner

- `cargo audit --json`: advisory database last updated `2026-08-21`, lock dependency-count 304, `vulnerabilities.found=false`, vulnerability count 0, warnings empty.
- 로컬 `cargo deny --version`은 명령 부재를 보고했다. 원격 CI `gh run view 32110917881 --repo Yupkidangju/AIHack --json ...`에서 Ubuntu job `95629955903`와 Windows job `95629955943` 모두 `Install cargo-deny` 및 `Check licenses, bans, and sources`를 success로 완료했다.
- `deny.toml:4-11`은 Apache-2.0/MIT/Unicode-3.0/Zlib를 허용하고 `winx = 0.36.4`의 `Apache-2.0 WITH LLVM-exception`만 version-scoped exception으로 허용한다. `deny.toml:15`는 wildcard dependency를 deny한다.

### 4.3 Local build/checkpoint evidence

- `cargo fmt --all -- --check`: exit 0
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: exit 0
- `cargo test --workspace --all-targets --locked`: exit 0, 실행된 전체 test process에서 failure 0
- `cargo build --workspace --release --locked`: exit 0
- Git Bash executable(`C:\Program Files\Git\bin\bash.exe`)로 `scripts/r7_checkpoint.sh`와 `scripts/r8_checkpoint.sh`를 실행: 각각 `R7 CHECKPOINT: PASS`, `R8 CHECKPOINT: PASS`, exit 0
- `cmd /c build.bat --release`: exit 1, `?? docs/multi_audit/`가 있는 dirty worktree를 이유로 중단. 이는 script의 clean-tree fail-closed 동작이며 release failure finding으로 세지 않았다.
- checkpoint 실행 뒤에도 `git diff --exit-code -- Cargo.lock`: exit 0. 소스·설정·기존 문서는 수정하지 않았다.

### 4.4 Current HEAD and remote CI evidence

- `git ls-remote origin`: 현재 branch ref `80d959a...`, `main`/origin HEAD `41a1b63...`를 확인했다.
- `gh run list --repo Yupkidangju/AIHack`: 현재 HEAD `80d959af...`의 CI run `32110917881`이 2026-08-18에 completed/success다.
- `gh run view 32110917881`: Ubuntu와 Windows quality gate 모두 success. 양 job에서 metadata, fmt, clippy, test, R7/R8, release build, OS별 release bundle, cargo-audit, cargo-deny, lockfile check가 success다.
- 해당 run log에서 Ubuntu/Windows 모두 `R7 CHECKPOINT: PASS`, `R8 CHECKPOINT: PASS`가 확인된다. Ubuntu log에는 `PASS release bundle: version=0.3.0 commit=80d959af...`가 있다. 이는 현재 HEAD의 positive-path 원격 증거다.
- 기존 문서가 가리키는 report-24 implementation SHA `2519bc8e...`/Actions `32107862171`도 원격 success이며, `git diff 2519bc8e... HEAD`는 문서·테스트만 변경된 결과다. 따라서 해당 run은 당시 implementation SHA 증거로는 유효하지만 현재 HEAD run 자체는 `32110917881`이다.

## 5. Findings

### Pass 1: Implementation Compliance

### [SC-F001] Provenance의 Cargo.lock record가 실제 winx license exception을 부정함

- Pass: Implementation / Supply-chain
- Pattern: `DEP-001`, `SEC-006`, reverse document-code sync
- Area: dependency license provenance, exception inventory
- Severity: Major
- Status: Confirmed
- Summary: 실제 resolved graph는 `winx 0.36.4`의 비허용 SPDX 식을 version-scoped exception으로 통과시키는데, provenance inventory는 해당 graph에 “no exception”이라고 기록한다.
- Evidence:
  - `deny.toml:8-11`: `winx`, `version = "=0.36.4"`, `allow = ["Apache-2.0 WITH LLVM-exception"]` exception.
  - `Cargo.lock`/`cargo tree --locked --target all -d`: `winx 0.36.4 -> cap-primitives 4.0.2 -> aihack-runtime` 경로가 실제 runtime graph에 존재한다.
  - `PROVENANCE.md:50`: `PROV-0005 | Cargo.lock ... SPDX allowlist ... cargo deny check ...; no exception`이라고 기록한다.
  - `BUILD_GUIDE.md:220-222`, `PROVENANCE.md:146`, `DESIGN_DECISIONS.md:15-40`은 반대로 winx exception, owner, 만료일을 설명한다.
  - 원격 CI의 cargo-deny 0.19.4는 실제 exception으로 success했으므로 현재 build가 실패한다는 finding은 아니다.
- Expected Basis: `PROVENANCE.md`의 runtime inventory, `deny.toml`의 실제 policy, 사용자가 요청한 dependency/license graph와 문서의 양방향 정합성.
- Actual: 실행 policy와 상세 설계 문서는 exception을 알고 있지만 authoritative-looking `PROV-0005` row가 예외를 누락·부정한다. 수신자는 provenance만 읽으면 모든 dependency가 allowlist 안에 있다고 오판할 수 있다.
- Impact: release source와 license evidence의 재현성이 깨진다. 특히 exception의 적용 범위와 실제 shipped dependency를 연결하지 못하고, 향후 exception 제거·교체 시 검토 대상 누락을 만든다. 법률 위반으로 단정하는 finding이 아니라 engineering evidence drift다.
- Suggested Action: `PROV-0005`를 `winx 0.36.4`의 SPDX exception, owner, 승인일, 만료일, 적용 graph와 함께 갱신하고, Cargo.lock package/license scan 결과에서 exception ledger로 자동 생성하거나 교차 검증한다. `tests/provenance_manifest.rs` 또는 별도 supply-chain test가 실제 graph에 존재하는 모든 non-allowlisted license와 승인 record의 일대일 대응을 검증하게 한다.
- Re-audit Method: `cargo metadata --locked`, `cargo tree --locked --target all`, `cargo deny check licenses bans sources`를 다시 실행하고, provenance row가 `winx 0.36.4`/`Apache-2.0 WITH LLVM-exception`/owner/date/evidence를 정확히 가리키는지와 예외가 다른 crate로 확장되지 않는지 확인한다.
- Confidence: High
- Notes: 현재 `cargo audit` 취약점 0건 및 cargo-deny success와 별개인 provenance documentation finding이다.

### [SC-F002] License exception 만료·소유자 정책이 실행 가능한 gate가 아니라 문서 문자열에만 있음

- Pass: Implementation / Debug
- Pattern: `SEC-006`, scanner provenance, fail-closed release gate
- Area: cargo-deny exception lifecycle
- Severity: Major
- Status: Confirmed
- Summary: `winx` 예외의 owner/만료일/재검토 조건은 `BUILD_GUIDE.md`, `PROVENANCE.md`, `DESIGN_DECISIONS.md`에만 있고 `deny.toml`이나 CI에서 만료·의존성 변경을 자동 실패시키는 검사가 없다.
- Evidence:
  - `deny.toml:8-11`에는 name/version/allow만 있고 owner, 승인일, 만료일, dependency-trigger field가 없다.
  - `BUILD_GUIDE.md:220-222`와 `PROVENANCE.md:146`은 예외에 이유/owner/만료일을 요구하고, `2026-10-31` 만료 및 capability dependency 변경 시 재검토를 문서로만 선언한다.
  - `tests/build_contract.rs:221-241`은 해당 문구가 존재하는지만 확인하며 현재 날짜가 만료일 전인지, Cargo graph의 exception이 ledger와 일치하는지 검사하지 않는다.
  - CI는 `cargo deny check licenses bans sources`를 실행하지만 별도의 exception expiry/owner/graph cross-check command는 없다.
- Expected Basis: 프로젝트 자체 문서의 “만료일 최대 90일”, “version 변경 시 즉시 재검토”, “미승인 third-party exception이면 release 중단” 계약과 fail-closed 공급망 gate 원칙.
- Actual: 현재(2026-08-23) 만료일은 아직 미래이고 CI는 통과하지만, `2026-10-31` 이후에도 현재 명령만으로는 cargo-deny가 계속 success할 수 있다. exception은 의도보다 넓은 graph에 적용되지 않는지와 review freshness를 machine gate가 보장하지 않는다.
- Impact: 지금 즉시 license failure가 발생했다는 뜻은 아니지만, review 만료 뒤 release gate가 자동으로 닫히지 않는 fail-open 수명주기 위험이다. 문서 갱신을 잊으면 외부 배포 전에 stale exception이 남을 수 있다.
- Suggested Action: owner/approved_at/expires_at/crate/version/SPDX/dependency trigger를 machine-readable exception ledger로 두고, CI에서 현재 날짜·resolved graph·`deny.toml` exception을 비교하여 만료, crate/version drift, missing owner/evidence를 exit non-zero로 만든다. 기존 string-presence test는 이 checker의 negative fixture(만료, version drift, unrelated crate)를 보강하는 용도로 유지한다.
- Re-audit Method: 만료일을 과거로 둔 fixture, `winx` version 변경 fixture, 다른 crate에 같은 SPDX 허용을 붙인 fixture를 CI checker가 모두 FAIL하는지 확인한 뒤 cargo-deny positive path와 함께 실행한다.
- Confidence: High
- Notes: 만료일 전 현재 상태는 pass 가능한 known condition이나, release gate가 미래 상태를 닫지 않는 문제다.

### [SC-F003] Windows release bundle 검증이 Linux verifier와 동일한 fail-closed 계약이 아님

- Pass: Debug / Security
- Pattern: `BUILD-001`, `SEC-006`, release integrity
- Area: Windows packaging, archive/checksum/provenance verification
- Severity: Major
- Status: Confirmed
- Summary: Linux `build.sh`는 source archive를 만든 뒤 `scripts/verify_release_bundle.sh`를 호출하여 archive exclusion, output/archive record byte equality, exact metadata, SHA256SUMS를 재검증하지만, Windows `build.bat`는 해당 verifier를 호출하지 않고 일부 positive checks와 hash 생성만 수행한다.
- Evidence:
  - `build.sh:88-98`: SHA256SUMS 생성 뒤 `scripts/verify_release_bundle.sh`를 호출한다.
  - `scripts/verify_release_bundle.sh:39-62,64-100`: 필수 artifact non-empty, archive required files, `legacy_nethack_port_reference|target|output` exclusion, archive/output metadata와 approval/modification record equality, `sha256sum --check --strict`를 확인한다.
  - `build.bat:88-106`: ZIP required-name listing, metadata key 검사, approval/modification ID substring 검사, `Get-FileHash`로 SHA256SUMS 생성만 수행한다. archive exclusion 검색, output/archive record equality, 생성된 checksum의 재검증, binary/document non-empty 검사가 없다.
  - `tests/release_bundle.rs:1`은 `#![cfg(unix)]`로 Windows에서 전체 bundle negative/positive fixture가 0 tests가 된다. Windows의 `tests/release_gate.rs`는 R8 shell fixture와 script text 계약만 검사한다.
  - 현재 원격 CI Windows job `95629955943`의 positive build step은 success지만, 이 gap을 검증하는 Windows negative fixture는 실행되지 않는다.
- Expected Basis: `BUILD_GUIDE.md:150-175`의 “같은 계약”, `PROJECT_OWNER_LICENSE_APPROVAL.md:34-44`의 bundle 구성, 사용자 요청의 Linux/Windows bundle/checksum/provenance fail-closed 요구. OS별 구현은 달라도 실패 경계는 동등해야 한다.
- Actual: CI 순서상 R7/R8가 먼저 `.gitattributes`와 provenance를 검사하므로 현재 commit의 정상 경로는 보호된다. 그러나 `build.bat` 단독 실행 또는 향후 CI 순서 변경 시, legacy path 포함 ZIP, output/archive record 불일치, 손상된 SHA256SUMS/zero-size artifact를 자체적으로 거부할 수 없다.
- Impact: Windows에서 생성된 release evidence가 Linux와 같은 수준의 commit/source/record/checksum 무결성을 증명하지 못한다. 현재 성공 로그는 정상 입력의 positive path만 입증하며 negative tamper path를 닫지 않는다.
- Suggested Action: Windows용 PowerShell verifier를 별도 구현하거나 공통 manifest/checksum verifier를 cross-platform으로 만들고 `build.bat` 마지막 단계에서 반드시 호출한다. ZIP 내부 excluded path, exact archive/output record equality, non-empty artifact, SHA256SUMS strict verification을 추가하고 `tests/release_bundle.rs`의 Windows-native fixture 또는 동일 의미의 PowerShell fixture를 CI에서 실행한다.
- Re-audit Method: Windows clean checkout에서 source ZIP에 legacy path를 강제로 포함한 fixture, archive/output record byte mismatch, duplicate/suffixed metadata, wrong/duplicate hash, zero-size binary를 각각 실행하여 build gate가 non-zero로 끝나는지 확인한다. Linux verifier와 결과 표를 비교한다.
- Confidence: High
- Notes: `R8 CHECKPOINT: PASS`는 현재 canonical preflight의 positive path이고 Windows bundle verifier parity를 대신하지 않는다.

### [SC-F004] GitHub Actions 실행 코드가 mutable tag로 신뢰됨

- Pass: Security / Debug
- Pattern: `SEC-006`, CI supply-chain provenance
- Area: GitHub Actions action pinning
- Severity: Major
- Status: Confirmed
- Summary: release artifact와 scanner 결과를 생성하는 workflow가 third-party/action reference를 commit SHA가 아닌 mutable tag로 사용한다.
- Evidence:
  - `.github/workflows/ci.yml:20`: `actions/checkout@v4`
  - `.github/workflows/ci.yml:21-24`: `dtolnay/rust-toolchain@1.94.1`
  - 같은 workflow가 fmt/clippy/test, R7/R8, release bundle, cargo-audit/cargo-deny를 모두 실행하므로 이 action code가 release evidence chain의 일부다.
  - 현재 run `32110917881`은 success지만, success는 tag가 이후 가리킬 code가 바뀌지 않는다는 provenance를 제공하지 않는다.
- Expected Basis: release CI의 입력 action은 immutable commit SHA로 고정하고, 버전 update를 별도 검토·기록해야 한다. `permissions: contents: read`는 action code substitution 위험을 제거하지 않는다.
- Actual: tag가 이동하거나 upstream repository가 compromise되면 checkout/toolchain 설치 단계가 바뀐 상태로 build/test/release evidence가 생성될 수 있다. workflow 내부에는 SHA pin 또는 action provenance assertion이 없다.
- Impact: Cargo.lock이 완전하고 cargo-audit/cargo-deny가 PASS해도 CI orchestration 자체가 변조되면 binary/source archive와 결과 로그의 신뢰성이 훼손될 수 있다.
- Suggested Action: `actions/checkout`와 `dtolnay/rust-toolchain`을 검증된 full commit SHA로 pin하고 주석에 human-readable version을 기록한다. Dependabot/Renovate 또는 수동 update 절차에서 SHA·release tag·diff를 검토하며, action pin 정책 검사(zizmor 등)를 CI에 추가한다.
- Re-audit Method: workflow에서 모든 `uses:`가 full 40-hex SHA인지 확인하고 pin 검사 negative fixture를 실행한다. 고정된 SHA로 양 OS run을 재실행하여 현재와 같은 gate·bundle 결과를 확인한다.
- Confidence: High
- Notes: 외부 action을 사용하는 일반적인 CI 자체를 문제 삼는 것이 아니라, release evidence를 만드는 mutable reference를 지적한다.

### [SC-F005] 현재 HEAD를 v0.3.0 release bundle로 포장할 때 modification/version 경계가 닫히지 않음

- Pass: Implementation / Debug
- Pattern: `IMP-001`, `IMP-004`, `BUILD-001`, provenance/release authority
- Area: release version, CHANGELOG, MODIFICATIONS, current HEAD evidence
- Severity: Major
- Status: Needs Clarification
- Summary: 현재 HEAD는 2026-08-18 변경을 포함하지만 `MODIFICATIONS.md`의 covered change period는 `2025-05-20..2026-07-20`이고 `CHANGELOG.md`에는 R9 및 report-24 후속 변경이 `[Unreleased]`로 남아 있다. 동시에 build script와 metadata는 v0.3.0 bundle을 만들 수 있다.
- Evidence:
  - `MODIFICATIONS.md:3-5,7-21`: Release version `0.3.0`, covered change period 종료일 `2026-07-20`, expanded commit을 manifest 근거로 사용한다고 기록한다.
  - `CHANGELOG.md:3-31`: R9 causal closure, capability artifact I/O, winx exception 등 후속 변경이 `[Unreleased]`에 있다.
  - `git log --since 2026-07-21`: `41a1b63`의 runtime code, `91d6016`의 save code, `2519bc8`의 release test와 현재 문서 commit 등 기간 이후 변경이 확인된다.
  - `RELEASE-METADATA:1-6`, `build.sh:78-80`, `build.bat:82-88`은 현재 clean HEAD를 version `0.3.0`으로 package한다.
  - `spec.md:757`은 R9 완료와 release decision 전까지 Cargo version을 0.3.0으로 유지한다고 하며, `README.md:20-21`, `BUILD_GUIDE.md:450`은 독립 재감사·외부 게시가 아직 HOLD라고 한다.
  - 현재 HEAD의 CI run `32110917881`은 positive bundle을 생성했지만 source modification manifest의 기간을 자동 검증하지 않는다.
- Expected Basis: release artifact의 version, changelog scope, modification manifest, source archive commit이 하나의 명확한 release candidate authority를 구성해야 한다. R9가 아직 독립 재감사 전이면 v0.3.0 외부 release인지 pre-release build인지 명시적으로 구분해야 한다.
- Actual: 현재 문서는 “개발 중이며 외부 게시 HOLD”와 “v0.3.0 bundle contract/CI success”를 함께 말하고, R8 preflight는 clean HEAD의 bundle 생성 자체를 막지 않는다. 따라서 실제 외부 게시를 시도할 때 어느 변경일까지의 modification notice와 어느 CI run이 release authority인지 문서만으로 단일하게 닫히지 않는다.
- Impact: 현재 외부 게시가 이루어졌다는 주장은 하지 않지만, operator가 preflight PASS와 positive bundle을 최종 v0.3.0 release 승인으로 오해하거나, 7월 20일 이후 source 변경을 누락한 modification notice와 함께 배포할 위험이 있다.
- Suggested Action: release candidate 정책을 하나로 결정한다. (a) R9 후속 변경을 포함해 `MODIFICATIONS.md`, `CHANGELOG.md`, approval/metadata를 exact candidate commit 기준으로 갱신하거나, (b) release scripts가 `[Unreleased]`/R9 pending 상태와 manifest cutoff mismatch를 감지해 package를 HOLD한다. CI evidence 문서에는 implementation SHA와 current HEAD SHA/run을 각각 명시하고 old same-SHA run을 historical evidence로 라벨링한다.
- Re-audit Method: 결정한 release boundary를 문서에 고정한 뒤 clean candidate commit으로 `R7/R8`, Linux/Windows bundle, source archive expanded metadata, modification period, CHANGELOG release entry를 다시 대조한다. candidate commit 외 source archive가 생성되면 fail하는 fixture도 추가한다.
- Confidence: Medium-High
- Notes: `Needs Clarification`은 법적 해석이 아니라 현재 문서가 development/pre-release와 final v0.3.0 release의 authority를 분리하지 못한다는 뜻이다. 현재 외부 게시 HOLD 자체는 문서에 명시되어 있다.

### [SC-F006] CI 문서의 active evidence가 현재 HEAD run과 historical implementation run을 구분하지 않음

- Pass: Implementation
- Pattern: `IMP-004`, reverse documentation sync
- Area: CI evidence traceability
- Severity: Minor
- Status: Confirmed
- Summary: 현재 active 문서의 여러 위치가 report-24 implementation SHA `2519bc8e`와 run `32107862171`을 “현재 working tree/current baseline”처럼 제시하지만 현재 HEAD에는 문서·테스트 후속 commit이 있고 최신 same-HEAD run은 `32110917881`이다.
- Evidence:
  - `BUILD_GUIDE.md:22,448`, `README.md:12,76`, `IMPLEMENTATION_SUMMARY.md:16`, `audit_roadmap.md:386`은 `2519bc8e...`/`32107862171`을 active evidence로 기록한다.
  - `git diff 2519bc8e... HEAD`는 source implementation보다는 문서·테스트 변경을 포함하며, `gh run list`는 current HEAD `80d959af...`의 run `32110917881` success를 보여준다.
  - `gh run view 32110917881`에서 current HEAD 양 OS gate와 current-HEAD Linux bundle pass가 확인된다.
- Expected Basis: historical implementation SHA evidence와 current HEAD CI evidence를 문서에서 명시적으로 분리하고, “현재 baseline” 표현은 해당 SHA/run으로 재현 가능해야 한다.
- Actual: old run은 implementation code 기준으로 유효하지만 current HEAD run이 문서에 연결되어 있지 않아 독자가 문서 SHA와 checkout HEAD를 혼동할 수 있다.
- Impact: 증거 재현·감사 인계 시 어느 commit을 검증해야 하는지 불필요한 ambiguity가 생긴다. old run 자체가 거짓이라는 finding은 아니다.
- Suggested Action: active docs에 `implementation SHA 2519bc8e / historical same-SHA run 32107862171`과 `current HEAD 80d959af / current run 32110917881`를 구분해 기록하고, “current working tree” 표기를 source implementation 기준인지 full tree 기준인지 명시한다.
- Re-audit Method: 각 active document의 CI SHA/run link가 현재 branch/ref 및 intended release candidate와 일대일로 해석되는지 확인하고 `gh run view` job SHA를 다시 대조한다.
- Confidence: High
- Notes: 현재 HEAD run은 success이므로 CI 기능 실패가 아니라 evidence lineage/문서 동기화 문제다.

### [SC-F007] 다중 버전 dependency는 정책상 허용되지만 유지보수용 duplicate budget이 없음

- Pass: Debug
- Pattern: `DEP-001`
- Area: duplicate dependency families
- Severity: Info
- Status: Confirmed
- Summary: 실제 graph에 여러 duplicate family가 있고 `deny.toml`은 `multiple-versions = "allow"`로 전역 허용한다. 현재 crossterm duplicate는 없고 cargo-deny는 pass하지만, 전역 allow는 새 duplicate가 생겨도 별도 검토 없이 통과시킨다.
- Evidence:
  - `cargo tree --locked --target all -d`: `getrandom` 3개, `rand` 2개, `hashbrown` 2개, `thiserror` 2개, `windows-sys` 3개 등.
  - `deny.toml:13-16`: multiple versions allow, crossterm만 duplicate deny.
  - `Cargo.lock`과 CI cargo-deny success는 현재 graph/policy 일치를 입증한다.
- Expected Basis: dependency graph의 duplicate는 의도된 transitive compatibility boundary로 허용할 수 있지만, 변경 시 triage 가능한 budget/allowlist 또는 정기 검토가 있어야 한다.
- Actual: 특정 family의 허용 사유·크기 영향·업데이트 owner가 문서화되지 않았고 전역 allow로 신규 중복까지 자동 허용된다.
- Impact: 현재 release 차단급 취약점은 아니나 binary size와 patch surface, 공급망 triage 비용이 증가할 수 있다.
- Suggested Action: `cargo tree --target all -d` 결과를 CI artifact 또는 summary로 기록하고, crossterm 외에도 known duplicate family와 사유를 allowlist/owner로 관리한다. 해결 가능해진 중복은 주기적으로 deduplicate한다.
- Re-audit Method: 새 dependency update에서 duplicate diff가 의도된 변경인지 review record가 남고, crossterm duplicate 및 신규 high-risk duplicate가 non-zero/검토 대기로 처리되는지 확인한다.
- Confidence: High
- Notes: `Info`로 분류했으며 현재 `cargo deny` gate의 실패나 wildcard violation으로 판단하지 않았다.

## 6. Cross-Pass Conflicts

### [XPF-SC-001] 양 OS CI positive PASS와 Windows release negative coverage 공백

- Related Findings: `SC-F003`, `SC-F006`
- Conflict: Implementation/Debug 증거로는 current HEAD의 Ubuntu/Windows job, R7/R8, release build가 모두 success다. 그러나 Security/release-integrity 관점에서는 Windows bundle script에 Linux verifier와 같은 negative boundary가 없고 Windows `release_bundle` fixture가 실행되지 않는다.
- Resolution: 원격 success는 정상 입력의 build/CI 증거로만 유지하고, Windows verifier parity와 negative fixture가 추가될 때까지 release-integrity PASS로 승격하지 않는다.
- Gate Impact: 공급망·릴리즈 관점의 PASS 차단. 현재 외부 게시 HOLD와 일치한다.
- Required Fix Before PASS: `SC-F003`의 Windows verifier와 tamper fixture를 CI에 추가하고, 양 OS 결과가 같은 fail-closed contract를 통과해야 한다.

### [XPF-SC-002] R8 checkpoint PASS와 외부 게시 HOLD의 표현 경계

- Related Findings: `SC-F005`
- Conflict: `scripts/r8_checkpoint.sh`는 현재 `R8 CHECKPOINT: PASS`를 반환하고, 동시에 `PROVENANCE.md:62-64`, `PROJECT_OWNER_LICENSE_APPROVAL.md:48-57`, `BUILD_GUIDE.md:450`은 독립 R8 재감사와 별도 사용자 승인 전 외부 게시를 HOLD한다.
- Resolution: 이는 의도된 preflight/final-approval 분리로 해석한다. checkpoint PASS는 기술 preflight와 bundle 생성 가능성만 의미하며 외부 release approval이 아니다. 다만 release boundary를 script와 active docs에서 더 명시해야 한다.
- Gate Impact: 현재 external publication은 계속 차단된다. preflight PASS를 final PASS로 소비하는 caller가 있으면 차단급 오판이 된다.
- Required Fix Before PASS: `SC-F005`의 v0.3.0 candidate authority와 preflight/final approval 명칭을 명시하고, 외부 게시 경로가 pending 상태에서 실행되지 않음을 machine gate 또는 명시적 operator handoff로 확인한다.

## 7. Accepted and Remaining Risks

- Major finding에 대해 현재 명시적으로 수용된 `Accepted Risk`는 없다. owner, 만료일, 재검토 조건이 없는 침묵의 면제로 처리하지 않는다.
- `SC-F007`의 duplicate family는 현재 cargo-deny 정책과 graph가 일치하고 즉시 gate를 차단하지 않는 `Info` 유지보수 위험이다. 이 상태를 release license 예외의 승인으로 해석하지 않는다.
- 외부 게시 HOLD는 project owner/release manager가 독립 R8/R9 재감사와 별도 승인 전까지 유지하는 문서화된 운영 경계다. 이는 현재 bundle verifier parity, exception lifecycle, release boundary finding을 해소하지 않는다.
- 남은 핵심 위험은 `winx` exception provenance/expiry drift, Windows bundle negative-path 미검증, mutable CI action, current HEAD의 release scope ambiguity다.

## 8. Uncertainties and Clarifications Needed

1. `cargo-deny`는 local Windows environment에 설치되어 있지 않다. 동일 HEAD 원격 CI 양 OS success로 보완했지만, 로컬 재현을 요구하면 `cargo-deny 0.19.4` 설치 후 다시 실행해야 한다.
2. Windows positive release bundle은 원격 CI에서 success지만, `tests/release_bundle.rs`가 Unix-only라 Windows negative bundle fixture의 실행 증거가 없다. 이 공백은 SC-F003의 핵심이다.
3. `MODIFICATIONS.md`의 2026-07-20 cutoff와 R9/Unreleased 변경을 v0.3.0 candidate에 포함할지, 아니면 release boundary를 R9 완료 뒤로 미룰지는 프로젝트 owner/maintainer가 결정해야 한다. 임의로 법적 요구를 창작하지 않았다.
4. GitHub Actions artifact가 보존·서명·attestation 되었는지는 확인하지 못했다. 현재 run log가 build/checkpoint success를 보여주는 것과 실제 외부 배포 artifact의 보관·서명은 별도 문제다.
5. `R8 CHECKPOINT: PASS`는 현재 문서상 technical preflight이고, `PROVENANCE.md:62-64`와 `PROJECT_OWNER_LICENSE_APPROVAL.md:48-57`이 독립 R8 audit/외부 게시 승인을 별도로 요구한다. 따라서 이 보고서는 R8 script PASS를 외부 release approval로 해석하지 않는다.

## 9. Perspective Decision

### 판정: HOLD (공급망·릴리즈 범위)

현재 graph와 advisory 자체는 통과한다.

- Cargo metadata/lock/source policy: Verified
- wildcard dependency: Verified (literal wildcard 0건; semver ranges는 lock으로 고정)
- toolchain/MSRV: Verified for current build (`rustc/cargo 1.94.1`, package MSRV 1.94, local/remote build success)
- RustSec: Verified, vulnerability 0건
- current HEAD remote CI positive path: Verified, run `32110917881`, Ubuntu/Windows success
- R7/R8 Git Bash checkpoint: Verified, exit 0

그러나 다음 Major 또는 미해결 gate가 남아 전체 release-ready PASS를 허용하지 않는다.

- SC-F001: provenance가 실제 winx exception을 부정함
- SC-F002: exception 만료/owner 정책이 executable fail-closed gate가 아님
- SC-F003: Windows bundle verifier가 Linux와 동등하지 않음
- SC-F004: release CI action이 mutable tag에 의존함
- SC-F005: current HEAD의 modification/version/release authority가 명확히 닫히지 않음

SC-F006은 Minor 문서 추적성 문제이고 SC-F007은 Info 유지보수 위험이다. Critical finding은 확인하지 않았으나, 위 Major가 해결되거나 명시적 owner·기한·재검토 조건을 가진 Accepted Risk로 기록되기 전에는 이 관점의 PASS 계열 판정을 내리지 않는다.

권장 수정 순서는 (1) exception/provenance machine ledger와 expiry gate, (2) Windows verifier 및 Windows negative fixture, (3) action SHA pin, (4) exact release boundary와 active CI evidence 갱신, (5) duplicate budget이다.

## 10. Coder Handoff

`C:\LocalDev\rust\AIHack\docs\multi_audit\1\sub_audit_05_supply_chain_ci.md`를 먼저 읽고, 각 finding을 현재 `spec.md`, `BUILD_GUIDE.md`, `PROVENANCE.md`, 실제 Cargo graph와 CI/release script에 대조한 뒤 우선순위대로 수정한다. 계약 변경이 필요하면 관련 문서를 먼저 갱신하고, 수정 후 `cargo metadata --locked`, `cargo audit`, cargo-deny, 양 OS build/checkpoint와 Windows negative bundle fixture의 재감사 증거를 기록한다.
