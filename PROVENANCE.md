# AIHack Provenance and License Gate

## P1–P3 콘텐츠 변경 기록 (2026-09-05)

`item.quest.ascension`은 사용자 요청에 따른 AIHack 캠페인용 신규 설계 데이터다. glyph/weight/price/effect shape는 `docs/campaign_spec.md`에 정의했으며 NetHack 원본 텍스트나 새 외부 asset을 복사하지 않았다. 기존 10개 item 정의는 변경하지 않았다. LF `items.toml` SHA256은 `77e3915de6b90667b030c341456eeeea558c61e1eed2650cb9495133fd1af1d9`에서 `12f4a178a457ff64fa739dea8e683e77ccf7c828b4653fca33c0791633ea6018`로 변경된다. 아래 소유자 승인 기록의 일자·주체를 새 승인으로 바꾸지 않으며 release/게시 승인은 별개다.

문서 상태: active inventory, project-owner derivative classification approved
작성일: 2026-07-15
관련 결정: DEC-LICENSE-01, ADR-0027
관련 Task: R7-1, R7-2
project-owner approval reference: `AIHACK-OWNER-2026-07-20-NGPL-01` (`PROJECT_OWNER_LICENSE_APPROVAL.md`)

> 이 문서는 출처·포함 상태를 관리하는 엔지니어링 게이트이며 법률 자문이 아니다. 배포 라이선스와 파생물 판단은 프로젝트 소유자 또는 적격 검토자의 승인이 필요하다.

## 1. 상태 enum

| 상태 | 의미 | runtime 포함 |
| --- | --- | --- |
| Unknown | 출처, 저작권자, 라이선스 범위 중 하나 이상 미확인 | 금지 |
| Reviewed | 파일과 source/checksum을 확인했으나 배포 허가 결론 없음 | 금지 |
| Approved | reviewer, date, 적용 라이선스, notice 의무를 기록 | 허용 |
| Blocked | 손상, 충돌, 출처 불명 또는 사용 금지 결정 | 금지 |

상태 전이는 `Unknown -> Reviewed -> Approved` 또는 `Unknown/Reviewed -> Blocked`만 허용한다. `Blocked -> Approved`는 새 검토 기록과 대체 source checksum이 있어야 한다.

## 2. 공식 NetHack 3.6.7 기준

| 필드 | 값 |
| --- | --- |
| release | NetHack 3.6.7 |
| official source page | https://www.nethack.org/v367/download-src.html |
| archive | `nethack-367-src.tgz` |
| official SHA-256 | `98cf67df6debf9668a61745aa84c09bcab362e5d33f5b944ec5155d44d2aacb2` |
| release date | 2023-02-16 |
| local verified archive | 2026-07-18 임시 다운로드 후 SHA-256 대조, repository 반입 없이 삭제 |
| archive 내부 license | `dat/license` |
| archive 내부 license SHA-256 | `93a3ae2cb8dee482daddfaebe53bcffe5b114b603def19b4dca21621cbc5a747` |
| current status | Approved as the licensing basis for AIHack's derivative distribution |
| runtime inclusion | no |

공식 배포 페이지가 source archive와 SHA-256을 게시한다. 2026-07-18 R7 engineering review에서 archive를 프로젝트 임시 디렉터리에 내려받아 게시 checksum과 일치함을 확인하고 `dat/license` checksum 및 source symbol locator만 기록한 뒤 임시 복사본을 제거했다. AIHack repository에는 archive나 NetHack source를 추가하지 않는다.

## 3. R7-1 inventory

더 구체적인 `path/scope` record가 넓은 glob보다 우선한다. 2026-07-20 프로젝트 소유자는 NetHack 3.6.7 원본 소스로 의도를 추론해 AI-assisted semantic rewrite한 AIHack 전체를 파생물로 분류하고 NGPL 배포를 승인했다. 이 라이선스 승인은 R8 기술·릴리스 감사 `PASS`를 대신하지 않는다.

<!-- runtime-inventory:start -->
| ID | path/scope | source/origin | checksum | status | runtime | reviewer | reviewed_at | license_id | license_scope | notice_required | modification_notice_required | evidence |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| PROV-0001 | `src/**` | AI-assisted semantic rewrite from NetHack 3.6.7 source and AIHack repository history | Git object history | Approved | yes | Project owner | 2026-07-20 | `NGPL` | whole AIHack derivative distribution | true | true | Project owner derivative classification; `AIHACK-OWNER-2026-07-20-NGPL-01`; Git history records modifications |
| PROV-0002 | `crates/**` | AI-assisted semantic rewrite and AIHack workspace extraction history | Git object history | Approved | yes | Project owner | 2026-07-20 | `NGPL` | whole AIHack derivative distribution; PROV-0004 is the specific content record | true | true | Project owner derivative classification; `AIHACK-OWNER-2026-07-20-NGPL-01`; workspace boundary tests |
| PROV-0003 | `apps/**` | AIHack-authored TUI/headless adapters for the derivative work | Git object history | Approved | yes | Project owner | 2026-07-20 | `NGPL` | whole AIHack derivative distribution | true | true | Project owner derivative classification; `AIHACK-OWNER-2026-07-20-NGPL-01`; headless/TUI tests |
| PROV-0004 | `crates/aihack-content/src/data/**` | AI-assisted semantic rewrite from NetHack 3.6.7 source into reduced deterministic fixtures | `docs/provenance/r7-content.sha256` | Approved | yes | Project owner | 2026-07-20 | `NGPL` | whole AIHack derivative distribution | true | true | Project owner derivative classification; `AIHACK-OWNER-2026-07-20-NGPL-01`; AI-assisted semantic rewrite from NetHack 3.6.7 source |
| PROV-0005 | `Cargo.lock` | crates.io packages locked by checksum과 별도 time-bounded exception ledger | Cargo.lock registry checksums, `dependency-exceptions.json`의 `DEP-EXC-0001` | Approved | yes | Dependency owner / Release manager | 2026-08-23 | SPDX allowlist + `winx 0.36.4`의 `Apache-2.0 WITH LLVM-exception` | private workspace dependency graph | false | false | cargo-deny 0.19.4; `DEP-EXC-0001` owner/2026-10-31 expiry/trigger graph machine gate |
| PROV-0006 | `legacy_nethack_port_reference/src/**` | previous Rust port | not trusted for inclusion | Blocked | no | Codex engineering review | 2026-07-18 | pending | mixed or unclear legacy scope | pending | pending | direct import and copy prohibited |
| PROV-0007 | `legacy_nethack_port_reference/assets/**` | previous port and possible NetHack-derived data/text | not trusted for inclusion | Blocked | no | Codex engineering review | 2026-07-18 | pending | mixed or unclear legacy scope | pending | pending | strings tables and data copy prohibited |
| PROV-0008 | `legacy_nethack_port_reference/LICENSE.NGPL` | damaged local NGPL-looking text | `5e3e7c0cd3be7f65f4d9b59b49820c303abfa92c95497c5eb8cff2b64e456bdf` | Blocked | no | Codex engineering review | 2026-07-18 | pending | not valid notice evidence | pending | pending | lines 33..35 corruption preserved; do not ship |
| PROV-0009 | `legacy_nethack_port_reference/LICENSE` | Apache-2.0 text without reliable tree scope notice | `db09a0cfbc9276c576b3f7f45e1639ed0c5d121360e71c1c52b6bf1a4a4e886a` | Reviewed | no | Codex engineering review | 2026-07-18 | pending | application scope unresolved | pending | pending | do not infer whole-tree Apache coverage |
| PROV-0010 | NetHack 3.6.7 official archive | official source and license basis | `98cf67df6debf9668a61745aa84c09bcab362e5d33f5b944ec5155d44d2aacb2` | Approved | no | Project owner | 2026-07-20 | `NGPL` | source and license reference for the whole derivative work | true | true | `AIHACK-OWNER-2026-07-20-NGPL-01`; official archive and `dat/license` checksums verified; source archive remains outside repository/runtime |
| PROV-0011 | `docs/compatibility/NH367-C*.md` | AIHack-authored source review and semantic rewrite records | record-local archive checksum | Approved | no | Project owner | 2026-07-20 | `NGPL` | derivative-work documentation distributed with source | true | true | Project owner derivative classification; `AIHACK-OWNER-2026-07-20-NGPL-01`; ten scenario approval records |
| PROV-0012 | `.archive/**` | immutable AIHack document snapshots | Git object history | Approved | no | Project owner | 2026-07-20 | `NGPL` | AIHack project documentation distributed with source | true | true | Project owner derivative classification; `AIHACK-OWNER-2026-07-20-NGPL-01`; immutable archive chain |
<!-- runtime-inventory:end -->

현재 runtime에는 `Unknown` 또는 `Blocked` 자산이 직접 포함되지 않는다. 모든 runtime record와 NH367 compatibility record는 프로젝트 소유자의 파생물 분류에 따라 `Approved`이며, AIHack 전체에는 NGPL을 적용한다. `legacy_nethack_port_reference/`는 검증되지 않은 역사 자료로 계속 격리하고 release source archive에서 제외한다.

**외부 배포 판정: APPROVED FOR NGPL-COMPLIANT PACKAGING — R8 technical audit pending**

이 판정은 라이선스와 출처 기록 측면의 승인이다. 실제 외부 게시에는 R8 release audit `PASS`, `LICENSE`와 `NOTICE`, 그리고 해당 바이너리를 만든 커밋의 complete corresponding source archive가 모두 필요하다.

## 4. 손상된 NGPL 사본 기록

`legacy_nethack_port_reference/LICENSE.NGPL` 33..35행은 이름과 팀 설명 위치에 `combatants`가 반복되어 정상적인 license 원문으로 신뢰할 수 없다.

처리 규칙:

- 현 파일을 수정, 삭제, 이름 변경하지 않는다.
- 이 checksum을 손상 증거로 보존한다.
- 공식 3.6.7 archive를 확보하면 archive checksum부터 확인한다.
- 공식 archive 안의 license 파일 path와 checksum을 별도 record에 추가한다.
- 두 text의 diff를 만들되, 교정본을 현재 legacy file 위에 덮어쓰지 않는다.
- 검토 완료 전 Apache-2.0과 NGPL 중 하나가 전체 legacy tree에 적용된다고 단정하지 않는다.

## 5. 승인 record

```yaml
asset_id: PROV-0001
path_glob: crates/aihack-core/**
origin:
  kind: project-authored | official-nethack-367 | third-party | generated
  url: ""
  version_or_commit: ""
  sha256: ""
review:
  status: Unknown | Reviewed | Approved | Blocked
  reviewer: ""
  reviewed_at: YYYY-MM-DD
  license_id: ""
  license_scope: ""
  notice_required: true
  modification_notice_required: true
runtime:
  included: false
  target_crate: ""
evidence:
  - ""
notes: ""
```

필수값:

- Approved이면 reviewer, reviewed_at, license_id, license_scope가 비어 있지 않다.
- official-nethack-367이면 URL, archive SHA-256, archive 내부 path가 필요하다.
- generated이면 generator path와 input provenance를 기록한다.
- path_glob이 겹치면 더 구체적인 record가 우선한다.

## 6. 규칙 재구현 절차

1. NH367 compatibility ID를 만든다.
2. checksum이 확인된 공식 source archive의 파일·symbol·행 범위를 locator로 기록한다.
3. 관찰 가능한 precondition, command sequence, expected outcome만 한국어로 재서술한다.
4. NetHack 원본 source 또는 legacy Rust 구현을 열어본 경우 `reference_seen: true`를 기록한다.
5. 새 implementation은 root/workspace module에서 독립 작성한다.
6. 직접 복사 대신 프로젝트 소유자가 승인한 AI-assisted semantic rewrite 절차로 재작성하고 변경 이력을 남긴다.
7. test가 expected outcome을 검증한다.
8. reviewer가 scenario와 diff를 확인한 뒤 Approved로 전환한다.

## 7. 자동 게이트 계획

R7-1 구현에서 추가:

```bash
cargo audit
cargo deny check licenses bans sources
! rg -n "legacy_nethack_port_reference" Cargo.toml crates apps src \
  --glob '*.toml' --glob '*.rs'
cargo test --workspace --locked --test provenance_manifest
```

`tests/provenance_manifest.rs` 검증:

- runtime file마다 가장 구체적인 record가 정확히 하나 선택됨
- runtime included record는 Approved여야 SC-LICENSE-01과 R8 런칭 PASS 가능
- Approved의 reviewer/date/license/scope/notice/evidence 누락 0건
- content data의 full SHA-256 manifest가 실제 파일과 일치
- official archive 및 scenario checksum은 64 lowercase hex
- Blocked/Unknown path가 Cargo path/import/`include_str!`에 없음
- scenario ID 10개가 유일하고 필수 field 및 실제 test function과 연결됨
- R7 checkpoint는 asset/scenario provenance를 판정하고 root distribution license는 R8 release gate가 판정

tool version은 cargo-audit 0.22.1, cargo-deny 0.19.4로 고정한다. `deny.toml` exception은 crate/version/이유/owner/만료일을 기록하며 만료일은 승인일 이후 최대 90일이다.

## 8. 배포 중단 조건

다음 중 하나면 release artifact 생성·게시를 중단한다.

- root 또는 workspace package license가 `NGPL`이 아닌 상태에서 외부 배포를 시도
- runtime asset status가 Unknown, Reviewed, Blocked
- legacy path direct import 또는 path dependency
- 손상된 license를 정식 notice로 사용
- NetHack source/data/string을 source locator 없이 포함
- third-party dependency license exception 미승인

배포 재개에는 SC-LICENSE-01을 포함한 R8 release audit PASS가 필요하다. R7 `PASS WITH KNOWN RISKS`만으로 배포하지 않는다.
