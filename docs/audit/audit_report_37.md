# AIHack 전체 감사 37

**판정은 HOLD다.** 정상 플레이에서 미결제 두루마리를 읽으면 저장이 실패하고, 같은 거래의 남은 채무가 다른 상점주의 사망으로 소거된다. 이전 버전에서 정상적으로 열리는 캠페인 파일도 현행판에서는 재개할 수 없다. 각 결과를 긍정 대조군과 함께 확인했다.

## 대상과 증거

2026-11-28 KST, Windows, Rust/Cargo 1.94.1. 대상은 `main`의 `67a18973ac0883d0fc1a68fed36a5e29ed826e87`과 그 위의 미커밋 P13·감사 36 시정 작업이다. 기존 수정 파일 61개, 미추적 파일 3개(감사 36 보고서와 테스트 2개)의 SHA-256이 재개 시점에도 모두 일치했다. 비교 목록 자체의 SHA-256은 `c62c131c7001aedc072387764d1748ca8db845eea47a6e522377b84b9553863d`다.

기준은 [감사 표준](../../AI_AUDIT_DOC_STANDARD.md), [제품 명세](../../spec.md), [캠페인 계약](../campaign_spec.md), [설계 결정](../../DESIGN_DECISIONS.md), [감사 로드맵](../../audit_roadmap.md), [이전 감사](audit_report_36.md)다. README, 빌드 가이드, 변경 이력, 출처 목록, 구현 요약 및 CI와 대조했다. 실행 대상은 루트 패키지, 5개 크레이트, TUI와 Headless다. 게임 세션·저장·리플레이·콘텐츠·상점·어댑터 경로에 독립 사례를 추가했다. 임시 실행물과 로그는 저장소의 `target/audit37-20261128-1859/`에 있다.

## Pass 1 · 구현과 게임 계약

### A37-01 · Major · 외상으로 사용한 두루마리 때문에 저장 불가

시드 42 Knight에서 `Mines 1` 상점의 `ScrollEnchantArmor`를 주웠다. 사용 직전 동일 세션을 `ArtifactStore::save_session`으로 저장하면 성공한다. `submit(Read)`는 성공하고 `ScrollRead` 이벤트를 내며 아이템을 소비한다. 직후 같은 저장 API는 `item 184 has invalid unpaid location or kind`를 반환한다. 테스트 위치와 장비는 기존 감사 36 픽스처를 사용했고, 구매·독서·저장은 제품 API로 실행했다.

- **원인.** 마지막 한 개를 사용하면 런타임은 위치를 `Consumed`로 바꾸고 `unpaid`를 유지한다. 저장 검증기는 정확히 이 조합을 거부한다. 이 경로는 TUI와 Headless가 공유한다. [소비 처리](../../crates/aihack-runtime/src/systems/items.rs#L122) · [읽기](../../crates/aihack-runtime/src/systems/items.rs#L808) · [저장 전 검증](../../crates/aihack-runtime/src/save.rs#L154) · [거부 조건](../../crates/aihack-runtime/src/save.rs#L755)
- **시정과 재감사.** 상품의 소모와 외상 채무를 독립적으로 기록한다. 마지막 한 개 및 스택 일부 사용 직후 저장과 재개에 성공하고, 사용한 상품의 채무가 그대로 남아 정산되는지 확인한다. 기존 저장 파일 보존도 대조한다.

### A37-02 · Major · 다른 상점의 채무 80골드가 사라짐

감사 36 `DBG-F003`을 부분 재개한다. `Main 2`와 `Mines 1`의 물품을 하나씩 집어 총 160골드 채무를 만들었다. 두 물품을 들고 `Main 2` 상점주만 죽이면 기대대로 80골드가 남았다. 같은 상태에서 `Mines 1` 두루마리를 먼저 읽으면 읽기 직후에는 160골드지만, 같은 상점주 사망 직후 **0골드**가 된다. 두 경우 모두 대상 상점주의 사망을 확인했다.

- **원인.** 다른 상점의 채무가 남았는지를 현재 인벤토리의 미결제 물품만으로 판단한다. 이미 소비한 물품은 검색에서 빠진다. 기존 회귀는 두 물품이 모두 인벤토리에 남은 경우를 검사한다. [정산 분기](../../crates/aihack-runtime/src/systems/npc.rs#L230) · [기존 대조 사례](../../tests/audit36_remediation.rs#L1184)
- **시정과 재감사.** 상점별 잔액을 추적하고 해당 상점의 몫만 소거한다. 두 상점에서 각각 80골드의 외상을 만든 후 소지·소비·반납·지불 및 저장 재개 순서를 바꿔 최종 잔액을 대조한다. A37-01과 같은 거래로 재검증할 수 있다.

### A37-03 · Major · 소원 완드의 일반 캠페인 획득 경로 누락

시드 42·7·1234에서 13개 직업 모두를 일반 생성 명령으로 시작했다. 39개 월드의 모든 층, 인벤토리와 컨테이너를 포함한 전체 아이템 엔티티를 합쳐 55종이었다. 마법서는 `MagicMissile`과 `RemoveCurse`, 완드는 `Cold`, `Fire`, `MagicMissile`만 있었다. **소원 완드는 39회 모두 0개**였다. 이 수치는 시작 상태 표본이다.

이후 획득 경로도 확인했다. 캠페인 배치와 직업 장비 목록, 9개 상점 판매 목록, 일반 전리품·아티팩트·아이템 변환에서 소원 완드 공급 경로를 찾지 못했다. 소원 명령 자체는 이미 소원 완드를 소지해야 하고 자기 자신은 소원 대상에서 제외한다. 반면 감사 36의 TUI 회귀는 소원 완드를 인벤토리에 직접 넣고 시작한다. 카탈로그 전 종 테스트도 엔티티를 직접 생성한다. [캠페인 배치](../../crates/aihack-runtime/src/bootstrap.rs#L449) · [상점 판매 목록](../../crates/aihack-runtime/src/systems/shop.rs#L15) · [소원 선행조건](../../crates/aihack-runtime/src/systems/projectiles.rs#L89) · [TUI 회귀](../../tests/audit36_remediation.rs#L1337) · [카탈로그 회귀](../../tests/stage6_p12.rs#L1075)

시정 후에는 명세가 의도한 희귀도에 맞춰 입수 경로를 제공하고, 고정 시드에서 실제 명령으로 획득·식별·소원·충전량 감소까지 확인한다. 다른 26종 마법서의 공급 범위도 별도로 명시한다.

### A37-04 · 설계 확인 · 소코반 퍼즐을 풀지 않고도 보상 칸까지 이어지는 지형

시드 42의 실제 저장 맵에서 바위와 모든 구덩이를 통행 불가로 두고, 바닥·계단만으로 상하좌우 최단 경로를 계산했다. 입구에서 1~3층 다음 계단과 4층 보상 칸까지 각각 **62, 62, 64, 64걸음**이다. 네 층 모두 바위를 한 번도 밀 필요가 없었다. 이는 지형 접근성 측정이며 몬스터와의 전투를 포함한 플레이 시간 측정은 제외했다.

`CODER_HANDOFF_P7_TO_P13.md`는 퍼즐 완주 보상을 요구하고, 생성기는 의도적으로 우회 통로를 낸다. 4층 보상은 생성 시부터 바닥에 있다. 보상을 퍼즐 해결에 종속할지 판단이 필요하다. 필수라면 미해결·해결 상태에서 같은 보상 칸의 접근성을 대조하고, 선택이라면 플레이 안내에 명시한다. [요구](../CODER_HANDOFF_P7_TO_P13.md#L331) · [통로와 바위](../../crates/aihack-runtime/src/campaign_map.rs#L682) · [보상](../../crates/aihack-runtime/src/bootstrap.rs#L776) · [시험의 위치 이동](../../tests/stage7_p13.rs#L270)

## Pass 2 · 저장 호환성, 결정론과 문서

### A37-05 · Major · P12의 정상 V3 캠페인을 P13에서 재개할 수 없음

이전 작성기는 HEAD 커밋에서 소스를 별도 디렉터리에 추출해 `cargo build -p aihack-headless --locked`로 만들었다. 각 작성기가 공개 CLI로 생성한 **수정하지 않은 파일**을 동일 조건에서 읽었다. 아래는 시드 42, 3턴 저장 뒤 `--load … --turns 4 --policy wait-v1`의 결과다.

| 파일 | 스키마·층 | P12 독자 | 현행 P13 독자 |
| --- | --- | --- | --- |
| P12 Knight 캠페인 | 3 · 8층 | 성공, `a64b264db8dbb379` | 종료 2, `persisted level IDs do not match the active registry` |
| P13 Knight 캠페인 | 3 · 24층 | 미실행 | 성공, `f2bba85fb1c2f165` |
| P12 레거시 | 1 · 2층 | 성공, `c2e8fa971be6ba79` | 성공, 같은 해시 |

양쪽 작성기가 V3를 표시하는데 현행 로더는 현행 층 집합의 정확한 일치를 요구한다. 정상 구세대 파일과 손상 파일이 같은 오류를 받는다. 구버전 캠페인의 지원·이행 정책은 확인되지 않았다. 원본 보존, 지원 세대 식별, 재개 절차를 먼저 정하고 이전 작성기 파일을 호환성 회귀에 포함한다. P12 원본의 SHA-256은 `822490777ad16a95e1f923007bf04dd12a4621bc0a4f490d23d6118371e45199`다. [세대 판정](../../crates/aihack-runtime/src/save.rs#L346) · [층 집합 검사](../../crates/aihack-runtime/src/save.rs#L605) · [캠페인 계약](../campaign_spec.md#L342)

### A37-06 · Major · 후속 결과가 다른 두 상태의 스냅샷 해시가 동일

두 상점 물품을 든 A37-02 대조군의 복사본에서 두 번째 물품의 `unpaid_level` 한 필드만 변경했다. 두 상태 모두 현행 저장 검증을 통과했고 `snapshot().stable_hash()`가 **`8f35bbd61b334592`**였다. 같은 대상에 같은 완드 명령을 제출한 뒤 원래 상태의 채무는 80, 원산지를 바꾼 상태는 0이었다. 이 비교에는 선언적으로 만든 상태 쌍을 사용했다.

엔티티 스냅샷은 `unpaid`를 기록하지만 정산에 쓰이는 `unpaid_level`은 생략한다. 해시가 일치해도 해당 게임 상태가 같다고 판단할 수 없다. 캠페인 해시 입력에 실제 정산 상태를 반영하고 레거시 V1 골든 해시 보존 및 저장·재개·리플레이를 검증한다. [스냅샷 필드](../../crates/aihack-runtime/src/snapshot.rs#L93) · [아이템 투영](../../crates/aihack-runtime/src/snapshot.rs#L269) · [해시 계산](../../crates/aihack-runtime/src/snapshot.rs#L343)

### A37-07 · Minor · 현행 빌드 가이드의 저장 버전과 감사 상태 표기

빌드 가이드의 현재 실행 안내는 캠페인 스키마를 2로 설명하지만 공개 CLI가 쓴 파일의 값은 3이다. 또한 제품 명세 첫머리는 현 HEAD 날짜가 고지 기간을 벗어난다고 적는다. 실제 HEAD 날짜와 고지 종료일은 모두 2026-11-28이다. 문서 테스트는 특정 과거 보고서 번호를 현행 권위로 고정한다. 기존 기록은 보존하고 현행 실행·배포 안내 및 그 안내의 테스트를 실제 조건과 동기화한다. 미커밋 작업의 양 OS 번들 승인 여부는 별도다. [빌드 가이드](../../BUILD_GUIDE.md#L16) · [현행 상태 설명](../../spec.md#L14) · [고지 기간](../../MODIFICATIONS.md#L4) · [문서 회귀](../../tests/r8_documentation.rs#L275)

## Pass 3 · 로컬 LLM, 파일 경계와 공급망

- LLM 전송 코드는 루프백 주소 검증과 고정, 프록시·리다이렉트 차단, 시간 제한, canonical JSON 크기 검사, 제한된 응답 읽기를 수행한다. 미식별 인벤토리의 BUC와 인챈트는 런타임 관찰에서 마스킹하고 LLM에 투영한다. 이 감사에서는 실제 모델 서버 연결을 생략했다. [전송 경계](../../crates/aihack-llm/src/transport.rs#L73) · [요청과 응답](../../crates/aihack-llm/src/transport.rs#L195) · [미식별 값 처리](../../crates/aihack-runtime/src/observation.rs#L384)
- `ArtifactStore`의 상대경로 검사, 저장 전 검증과 바이트 제한을 대조했다. A37-01은 정상 게임 상태에서 작성기가 저장에 실패하는 가용성 문제다. 저장 검증을 일괄 완화하면 파일 무결성에 영향을 준다. [경로 검증](../../crates/aihack-runtime/src/save.rs#L63) · [저장 진입점](../../crates/aihack-runtime/src/save.rs#L154)
- 로컬 RustSec DB 공지 1,571건, lockfile 의존성 318개 기준 `cargo audit --no-fetch` 종료 0. 허용 경고 2건은 `rustix 1.1.4 / RUSTSEC-2026-0486`과 `tokio 1.52.4 / RUSTSEC-2026-0386`다. 원격 공지 동기화는 수행하지 않았다. `cargo deny --offline check licenses bans sources`도 통과했다. 별도 라이선스 예외 `DEP-EXC-0001`의 만료일은 2027-02-20이다.

## 감사 36 시정 재검증

| 이전 항목 | 이번 확인 |
| --- | --- |
| IMP-F001, SEC-F003, XPF-F001 | 아스트랄 제단의 좌표·성향 불변과 비정렬 봉헌 회귀 통과 |
| IMP-F002~004, IMP-F006 | 실패 시 MP·숙련도, 마법서, 미스릴, NPC·펫, 투명·소원 규칙에 대한 기존 시정 회귀 통과. 실제 소원 완드 공급은 A37-03 |
| IMP-F005 | P13 구현 요약, 캠페인 계약, 출처 문서의 예외 기한 반영 확인. 현행 빌드·감사 안내는 A37-07 |
| DBG-F001~002, DBG-F004~005 | 문 잠금·소코반 두 번째 바위 저장, 비굴착 행동 필터, 기타 런타임 시정 회귀 통과. 모든 TOML 문법 조합을 새로 시험하지는 않음 |
| DBG-F003 | 오라클 및 두 물품을 계속 소지한 원래 사례 통과. 소비가 개입한 상점 정산은 A37-01·02 |
| DBG-F006, SEC-F001, XPF-F002 | 문을 변경한 세션을 직렬화하도록 수정됐고 성장 검증 회귀 통과. 물약을 주입한 테스트와 실제 플레이 진행의 증거 범위를 구분 |
| SEC-F002, SEC-F004 | 미식별 행동·UI·로그의 시정 회귀 통과. 기존 의존성 경고 두 건은 잔존 |

## 실행 기록과 제외 범위

| 게이트 | 실제 확인 결과 |
| --- | --- |
| 메타데이터·포맷·정적 검사 | 8개 패키지. `cargo metadata --locked --offline --no-deps`, `cargo fmt --all -- --check`, `cargo clippy -j 2 --workspace --all-targets --locked -- -D warnings` 종료 0 |
| 릴리스 빌드·출처 | `cargo build -j 2 --workspace --all-targets --release --locked`, Git Bash R7·R8 체크포인트, Cargo.lock 변경 검사 종료 0 |
| Headless 장기 실행 | `survival-v1` 3개 시드 모두 1,000회 제출·1,000턴 수락·Playing. 시드 42 `bea676237ad7a9ad`, 7 `ef6a43bbb7264ee8`, 1234 `e200496ee9b45b0c` |
| 독립 감사 자료 | `shop37.rs`와 `shop37.log`에 거래 대조 3종, `catalog37.rs`와 `catalog37.log`에 직업·시드 표본, `sokoban37.json`에 층별 거리. 구세대 원본과 CLI 결과는 증거 폴더의 `runtime/`에 보관 |
| 전체 테스트 | Windows 로컬 샌드박스에서 `cargo test -j 4 --workspace --all-targets --locked --no-fail-fast` 실행. **104개 테스트 바이너리, 563 통과·19 실패·0 무시, 종료 101**. 캠페인 투어를 포함한 `campaign` 12개, `audit36_remediation` 11개, 실제 Windows ConPTY 4개는 이 실행에서 통과 |
| 실패 대상 재실행 | 같은 PC의 샌드박스 밖에서 같은 디버그 테스트 바이너리 대상으로 `provenance_manifest` 14/14, `release_archive_security` 2/2, `release_bundle_windows` 7/7, `release_gate` 7/7 통과. 두 재실행 명령 모두 종료 0. 첫 실행 실패 이름 19개를 재실행 통과 이름과 전수 대조. **두 실행 환경을 합쳐 고유 테스트 582개 모두 통과**, 단일 전체 명령의 종료 코드는 위와 같이 101 |

이전 실행은 18:59:45 시작한 PowerShell PID 41268의 작업 트리로 확인했다. 컴파일 중 사용자 요청으로 10개 프로세스를 종료하고 잔류 0개를 확인했다. 그 실행의 부분 로그 `test.log`는 보존했다. 재개한 실행은 저장소의 기존 빌드 캐시를 사용했다. 외부 게시, 실모델 추론, 실제 키보드 조작, 현재 미커밋 변경을 포함한 동일 커밋의 Ubuntu·Windows 실배포 번들은 검증 범위 밖이다.

첫 실행에서는 Git Bash의 `CreateFileMapping` 및 Python 임시 압축 해제 접근에 Win32 오류 5가 기록됐다. `local-suite.log`, `local-suite.exit`, `local-native-provenance.log`, `local-native-release-tests.log`, `verified-summary.json`이 최초 실패와 재실행 결과를 보존한다. 감사 전 존재했던 수정·미추적 파일 64개의 내용 해시도 종료 전에 다시 대조했다.

### 공개 CLI 재현

아래 PowerShell 명령의 시작 위치는 저장소 루트다. 첫 줄에서 감사 디렉터리로 이동하므로 명령이 생성하는 보고서는 격리된 `runtime/`에 기록된다. P12 원본 파일과 현재 파일은 덮어쓰지 않는다.

```powershell
Set-Location -LiteralPath 'target/audit37-20261128-1859'
.\p12\target\debug\aihack-headless.exe --load p12-run37.json --turns 4 --policy wait-v1 --report verify-old-reader.json
..\release\aihack-headless.exe --load p12-run37.json --turns 4 --policy wait-v1 --report verify-new-reader.json
..\release\aihack-headless.exe --load campaign-now.json --turns 4 --policy wait-v1 --report verify-current-control.json
..\release\aihack-headless.exe --load p12-v1-run37.json --turns 4 --policy wait-v1 --report verify-legacy-control.json
```

## 판정과 코더 인계

**제품 판정 HOLD.** 우선 정상 거래가 저장되고 상점별 채무와 결정론 해시가 일치하도록 A37-01·02·06을 함께 수정한다. 이어 A37-05의 구세대 지원 정책과 A37-03의 일반 플레이 획득 경로를 확정한다. A37-04는 설계 판단, A37-07은 문서 정합성 항목이다. 배포는 최종 동일 커밋의 양 OS 번들과 별도 승인 조건을 따른다.

코더는 `docs/audit/audit_report_37.md`의 각 지적을 현행 계약과 코드에 대조하고, 각 항목의 양성·음성 대조가 기대한 결과를 내는지 기록한다. 비교에 사용한 이전 세대 원본과 기존 감사 보고서는 보존한다.
