# AIHack Provenance and License Gate

문서 상태: active inventory, legal review pending
작성일: 2026-07-15
관련 결정: DEC-LICENSE-01, ADR-0027
관련 Task: R7-1, R7-2

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
| local verified archive | 없음 |
| current status | Reviewed as metadata only |
| runtime inclusion | no |

공식 배포 페이지가 source archive와 SHA-256을 게시한다. AIHack repository에는 그 archive를 추가하지 않는다. R7 검토 시 사용자가 별도 확보한 archive의 checksum을 확인하고, 필요한 source locator만 문서에 기록한다.

## 3. 초기 inventory

| 자산 | source/origin | local SHA-256 | 현재 상태 | runtime 포함 | 다음 조치 |
| --- | --- | --- | --- | --- | --- |
| `src/**`, `tests/**` | AIHack root 구현 이력 | per-file manifest 없음 | Reviewed | yes, current | R7 reviewer와 root license 결정 |
| `src/data/**` | AIHack 작성인지 legacy 변환인지 기록 불충분 | per-file manifest 없음 | Unknown | yes, current | 각 TOML field의 author/source 조사; 미확인 값 교체 |
| Cargo dependencies | crates.io + `Cargo.lock` | lockfile checksum | Reviewed | yes | `cargo deny check licenses` 정책과 exception 기록 |
| `legacy_nethack_port_reference/LICENSE` | Apache-2.0 text로 보이는 local file | `db09a0cfbc9276c576b3f7f45e1639ed0c5d121360e71c1c52b6bf1a4a4e886a` | Reviewed | no | 어떤 파일에 적용되는지 copyright notice와 함께 확인 |
| `legacy_nethack_port_reference/LICENSE.NGPL` | NGPL text로 보이는 local file | `5e3e7c0cd3be7f65f4d9b59b49820c303abfa92c95497c5eb8cff2b64e456bdf` | Blocked | no | lines 33..35 손상; 공식 3.6.7 license와 대조 후 별도 교정 계획 |
| `legacy_nethack_port_reference/src/**` | 이전 Rust port | manifest 없음 | Blocked | no | 직접 import/copy 금지; 규칙은 compatibility record로 독립 재구현 |
| `legacy_nethack_port_reference/assets/data/**` | 이전 port data | manifest 없음 | Blocked | no | item별 source/license 승인 전 사용 금지 |
| `legacy_nethack_port_reference/assets/dat/**` | NetHack text 자산 가능성 | manifest 없음 | Blocked | no | 문자열 복사 금지; 배포 권리 별도 검토 |
| `legacy_nethack_port_reference/tests/**` | 이전 port test knowledge | manifest 없음 | Blocked | no | 입력·기대값도 source locator를 갖춘 새 scenario로 작성 |
| `legacy_nethack_port_reference/REFERENCE_INDEX.md` | AIHack 내부 설명 | `242b66ba134db2a7e25b166aa60ae6be433d140dbccc29a642e436d2da1b3183` | Reviewed | no | 문서 주장만으로 재사용 승인하지 않음 |
| `.archive/**` | AIHack 문서 snapshot | archive별 checksum 미기록 | Reviewed | no | immutable chain 유지 |
| 새 NH367 scenario | 공식 Guidebook/source 관찰에서 독립 작성 | record별 source hash | Unknown | no | R7-2 template 완료 뒤 승인 |

현재 runtime에는 `Reviewed` 또는 `Unknown` 자산이 있어 R7 gate는 PASS가 아니다. 이 문서 작성만으로 배포 승인이 생기지 않는다.

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
2. 공식 Guidebook 또는 checksum이 확인된 source archive의 파일·symbol·행 범위를 locator로 기록한다.
3. 관찰 가능한 precondition, command sequence, expected outcome만 한국어로 재서술한다.
4. legacy Rust 구현을 열어본 경우 `reference_seen: true`를 기록한다.
5. 새 implementation은 root/workspace module에서 독립 작성한다.
6. 코드·상수·문자열을 복사하지 않는다.
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

- runtime path를 덮는 record가 정확히 1개
- runtime included record는 Approved
- Approved 필수 field 누락 0건
- official archive checksum format은 64 lowercase hex
- Blocked/Unknown path가 Cargo include 또는 `include_str!`에 없음

tool version은 cargo-audit 0.22.1, cargo-deny 0.19.4로 고정한다. `deny.toml` exception은 crate/version/이유/owner/만료일을 기록하며 만료일은 승인일 이후 최대 90일이다.

## 8. 배포 중단 조건

다음 중 하나면 release artifact 생성·게시를 중단한다.

- root license가 `UNLICENSED`이면서 외부 배포를 시도
- runtime asset status가 Unknown, Reviewed, Blocked
- legacy path direct import 또는 path dependency
- 손상된 license를 정식 notice로 사용
- NetHack source/data/string을 source locator 없이 포함
- third-party dependency license exception 미승인

배포 재개에는 R7 PASS와 R8 release audit PASS가 모두 필요하다.
