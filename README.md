# AIHack

> Current code: Cargo 0.3.0 · Release audit target: 0.3.0 · Updated: 2026-08-25

## 한국어

### 멀티 감사 2 플레이 수정 (2026-09-05)

Playing에서 `q`는 물약 선택, `Q`는 종료 확인이다. `?` 또는 하단 All은 전체 명령 메뉴, `i`는 전체 인벤토리, `M`은 최근 메시지다. 선택 메뉴는 방향/물건을 고른 뒤 실행하며 Esc는 턴 소비 없이 취소한다. 페이지는 PgUp/PgDn, `<`/`>`, 휠 또는 표시된 버튼으로 이동한다. `S` 저장·`L` 로드는 기본 `runtime/tui/quick-save.json`을 사용하며 `--save-dir <directory>`로 바꿀 수 있다. 종료 시 자동 저장하지 않는다.

적·바닥 물건 표시와 최소 화면 로그/food 수치를 복구했다. 생성 화면의 **13역할(`Knight`~`Archeologist`) × 5종족(`Human`/`Elf`/`Dwarf`/`Gnome`/`Orc` + 암시야) × 3성향(`Lawful`/`Neutral`/`Chaotic`)** 조합은 `80×21` Generator v3 본던전 6층+광산 2층(`SAVE_SCHEMA_VERSION_V3 = 3`), 6대 능력치(`Str`/`Dex`/`Con`/`Int`/`Wis`/`Cha`), 에너지 버킷 속도 스케줄링, 10개 장비 슬롯·통합 하강 AC(`AC:<val>`·`x` 장비 해제 메뉴), 확장 내성(`RES:`·`Telepathy`·반사/마법저항/갈등/텔레포트 제어/탐색)과 7종 상태이상(혼란·실명·둔화/가속·부유·투명·환각), 일반화 시체(`Corpse`, 부패·내성 부여·트롤 부활), P9 도구(`a` 열쇠·곡괭이 채굴·오일 램프·눈가리개·호루라기·유니콘 뿔)·컨테이너(`C` 자루/보관 가방/함정 상자)·`#dip`(`v` 성수 축복/연금술/희석/녹/분수 `Excalibur`)·`#engrave`(`E` `Elbereth`)·지형(분수 `{`/`g` 마시기·싱크 `}`·물/용암 `~`)·12종 함정, P10 심화 신앙(`Main 3` 동맹/`Mines 2` 적대 제단·`O` 개종/아티팩트 하사·이교 기도 응징)·6종 고유 아티팩트(`Excalibur`/`Sunsword`/`Magicbane`/`Cleaver`/`Stormbringer`/`Sting`, `V` 발동)·8계통 숙련도(`X` `#enhance`), P11 반려동물(시작 펫 `Kitten`/`LittleDog`/`Pony`·우호 자리바꿈·음식 투척 길들이기·진화·저주 회피·상점 절도·계단 동반)·실제 NPC(`Shopkeeper`/`TemplePriest`/`Oracle`/`Watchman`, `m` 대화·`D`/`$` 채무 정산/기부/상담·`DEBT:` HUD)·고급 몬스터 AI(문 열기·장비/물약/완드 줍기·사용·훔치기+텔레포트·부식·분열·소환·실명 방어 시선·오크 vs 드워프 파벌전), P12 마법 심화 및 대규모 도감(**아이템 130종·몬스터 85종** `build.rs` 정적 TOML 코드 생성, 7학파 **28주문** `Z` 동적 캐스팅·마법서 `r` 학습/20,000턴 감퇴·금속 방어구/방패 실패율, 18종 완드·소원 `z`, 12종 반지·6종 부적/`AmuletLifeSaving` 부활·크기별 무기 피해·은 특공), 시드 기반 미식별 외형·미감정 BUC/인챈트 은닉·대상 지정 감정(`r`→두루마리→대상)·소모품/투척물 중첩(`<qty>x`)·52슬롯(`a-z`/`A-Z`) 빈 문자 재활용, 탐험 기억 Fog of War, 허기 페널티·캠페인 자연 HP 회복, 문(`+`/`-`)·잠긴 문(`Kick` 해제)·비밀문·독/치료(`POIS:` HUD)·상점(`$`, `G` 골드), 추가 무기 5종(`LongSword`/`Katana`/`BattleAxe`/`Quarterstaff`/`BroadSword`)·귀환 추격전 캠페인을 시작한다. Main 3 위 계단에서 B로 광산에 들어간다. Main 6의 아뮬렛을 주워 Main 1 위 계단에서 `<`로 귀환하면 승천한다. HUD의 A는 목표 소지 표시다. Enter는 기존 두 층 Adventurer 호환 모드다. [캠페인 계약](docs/campaign_spec.md)과 [검증 기록](docs/campaign_implementation.md)을 참조한다. NetHack 전체 규칙 동등성은 주장하지 않는다.

Control note (EN): `q` selects a potion; `Q` confirms exit. `?` commands, `i` inventory (`a-z`/`A-Z` 52 slots with slot reuse & stack-merge), `r` read scroll or study spellbook (targeting menu for identified Identify scrolls, full BUC branching, 20,000-turn spell memory), `M` recent events (with hallucination filter), `P` put on ring, `T` remove ring, `x` take off equipped armor/amulet/ring, `a` apply tool, `C` loot container, `v` `#dip`, `E` `#engrave` (`Elbereth`), `g` drink from fountain/sink, `V` invoke artifact (`25`-turn cooldown), `X` `#enhance` skill proficiency (8 categories × 6 ranks), `m` chat with adjacent NPC/pet, `D` pay shop debt / donate to temple priest / consult Oracle, `$` shop buy/sell/pay menu (randomized appearance & buy price for unidentified stock), `z` zap wand or make curated Wish, `Z` dynamic cast spell menu (with failure %), `O` altar offer menu, `S`/`L` persistent save/load (`--save-dir`). No autosave. At creation choose from **13 Roles (`Knight`–`Archeologist`) × 5 Races (`Human`/`Elf`/`Dwarf`/`Gnome`/`Orc` + infravision) × 3 Alignments (`Lawful`/`Neutral`/`Chaotic`)** for an `80x21` Generator v3 six-floor campaign plus two Mines floors (`SAVE_SCHEMA_VERSION_V3 = 3`) with 6 attributes, energy-bucket turn scheduling, 10 equipment slots & unified Descending AC (`AC:<val>` HUD), resistance flags (`REFLECTION`, `MAGIC_RESISTANCE`, `CONFLICT`, `TELEPORT_CONTROL`, `SEARCHING`, `Telepathy`, etc.) & 7 status timers, generalized corpses, persisted Fog of War, hunger effects & campaign HP regen, hidden unappraised BUC/enchantment, amulet-blocked `ScrollLevelTeleport`, doors/locked doors (`Kick`)/hidden doors, 12 traps, fountains/sinks/water/lava, poison/cure (`POIS:` HUD), living shops (`Main 2`/`Mines 1`, `Shopkeeper`, unpaid wares, `DEBT:` HUD, `G` gold), peaceful NPCs (`TemplePriest`, `Oracle`, `Watchman`), starting pets (`Kitten`/`LittleDog`/`Pony` with friendly swap, food taming, evolution, cursed-item avoidance, shop theft, stair following), advanced monster AI (door opening, item pickup/use/death drop, `Leprechaun`/`Nymph`/`RustMonster`/`BlackPudding`/`Medusa`/`OrcCaptain`/`Dwarf` faction warfare), aligned altars (`Main 3` co-aligned / `Mines 2` cross-aligned, conversion, Divine Wrath, high-devotion gifts), 6 unique artifacts (`Excalibur`/`Sunsword`/`Magicbane`/`Cleaver`/`Stormbringer`/`Sting`), 5 weapons (`LongSword`–`BroadSword`) with size-based damage & silver bonus, **130 items & 85 monster kinds (`build.rs` TOML codegen)**, **28 spells across 7 schools**, **18 wands**, **12 rings & 6 amulets (including Life Saving)**, stackable consumables (`<qty>x`), XP growth, and ascension pursuit. Retrieve the Main 6 Amulet and ascend from Main 1. B enters Mines at Main 3 up stairs. Enter retains the legacy two-floor run.

操作メモ (JA): 作成画面で **13職業(`Knight`〜`Archeologist`) × 5種族(`Human`/`Elf`/`Dwarf`/`Gnome`/`Orc`・暗視) × 3属性(`Lawful`/`Neutral`/`Chaotic`)** を選び、`80×21` Generator v3・6能力値・エネルギーバケット行動順・10装備枠と下降AC(`AC:<val>`・`x`装備解除)・拡張耐性(`REFLECTION`/`MAGIC_RESISTANCE`/`CONFLICT`/`TELEPORT_CONTROL`/`SEARCHING`・`Telepathy`)と7状態異常・汎用死体・道具(`a`)/容器(`C`)/`#dip`(`v`)/`#engrave`(`E` Elbereth)・泉/流し(`g`)/水辺/溶岩と12種罠・祭壇信仰(`Main 3`同盟/`Mines 2`敵対・`O`改宗/神託下賜)・6種固有アーティファクト(`Excalibur`等・`V`発動)・8系統熟練度(`X` `#enhance`)・P11ペット同伴/手なずけ/進化/呪い回避/店泥棒/階段追従・NPC(`Shopkeeper`/`TemplePriest`/`Oracle`/`Watchman`、`m`会話・`D`/`$`精算/寄付)・高度モンスターAI(扉開閉・道具使用・盗み/錆/分裂/召喚/視線/派閥戦)・P12魔法体系深化と大規模図鑑(**全130種アイテム・全85種モンスター** `build.rs`静的TOML生成・7学派**28呪文**`Z`動的詠唱と失敗率・魔法書`r`学習/20,000ターン減衰・18種杖と願い`z`・12種指輪・6種護符/救命復活・サイズ別武器/銀特効)・未鑑定BUC/強化隠匿・店(`$`)・巻物BUC分岐と対象指定鑑定(`r`)・指輪(`=`/`P`/`T`)・消耗品スタック・V3キャンペーン保存を含む本坑6階＋鉱山2階、XP成長、昇天を遊べます。Enterは旧2階モード。qは薬、Qは終了確認、S/Lは永続保存・読込。自動保存はありません。

操作備註 (繁體中文): 建立角色時可選擇 **13職業（`Knight`～`Archeologist`）× 5種族（`Human`/`Elf`/`Dwarf`/`Gnome`/`Orc` 含黑暗視覺）× 3陣營（`Lawful`/`Neutral`/`Chaotic`）**，進入包含 `80×21` Generator v3（`SAVE_SCHEMA_VERSION_V3 = 3`）、六大屬性、能量桶排程、10裝備欄位與統一下降AC（`AC:<val>` 與 `x` 卸裝）、擴展抗性（含 `REFLECTION`/`MAGIC_RESISTANCE`/`CONFLICT`/`TELEPORT_CONTROL`/`SEARCHING`/`Telepathy`）與7狀態異常、通用屍體、P9工具（`a`）／容器（`C`）／`#dip`（`v`）／`#engrave`（`E` Elbereth）／噴泉・水槽（`g`）／水域・岩漿與12種陷阱、P10祭壇信仰（`Main 3` 同陣營／`Mines 2` 異陣營、`O` 改宗與神器賜予、異教祈禱神罰）、6件唯一神器（`Excalibur` 等、`V` 發動）、8系熟練度（`X` `#enhance`）、P11寵物同行／馴服／進化／迴避詛咒／商店竊取與活體NPC（`Shopkeeper`/`TemplePriest`/`Oracle`/`Watchman`、`m` 對話、`D`/`$` 結帳與捐獻）・進階怪物AI（開門、拾取裝備/藥水/魔杖、偷竊/生鏽/分裂/召喚/石化視線/陣營交戰）、P12法術深化與大型圖鑑（**130種道具與85種怪物** `build.rs` 靜態TOML生成、7學派**28法術** `Z` 動態施法與失敗率・法術書 `r` 學習／20,000回合遺忘、18種魔杖與許願 `z`、12種戒指與6種護符／救命復活、體型武器傷害／銀傷）、52格重用與堆疊、戰爭迷霧、未鑑定BUC隱藏與商店（`$`）與升天追擊的主地城6層＋礦坑2層。Enter保留舊2層模式。q選藥水、Q確認退出、S/L持久存讀檔，無自動存檔。

操作备注 (简体中文): 创建角色时可选择 **13职业（`Knight`～`Archeologist`）× 5种族（`Human`/`Elf`/`Dwarf`/`Gnome`/`Orc` 含夜视）× 3阵营（`Lawful`/`Neutral`/`Chaotic`）**，进入包含 `80×21` Generator v3（`SAVE_SCHEMA_VERSION_V3 = 3`）、六大属性、能量桶速度排程、10个装备槽位与统一下降AC（基础10、`AC:<val>` HUD 与 `x` 卸装）、扩展抗性（含 `REFLECTION`/`MAGIC_RESISTANCE`/`CONFLICT`/`TELEPORT_CONTROL`/`SEARCHING`/`Telepathy`）与7种状态异常、通用尸体、P9工具（`a`）／容器（`C`）／`#dip`（`v`）／`#engrave`（`E` Elbereth）／喷泉・水槽（`g`）／水域・岩浆与12种陷阱、P10祭坛信仰（`Main 3` 同阵营／`Mines 2` 异阵营、`O` 改宗与神器赐予、异教祈祷神罚）、6件唯一神器（`Excalibur` 等、`V` 发动）、8系熟练度（`X` `#enhance`）、P11宠物同伴／驯服／进化／回避诅咒／商店偷窃与活体NPC（`Shopkeeper`/`TemplePriest`/`Oracle`/`Watchman`、`m` 对话、`D`/`$` 结账与捐献）・进阶怪物AI（开门、拾取装备/药水/魔杖、偷窃/生锈/分裂/召唤/石化视线/阵营交战）、P12法术深化与大型图鉴（**130种道具与85种怪物** `build.rs` 静态TOML生成、7学派**28法术** `Z` 动态施法与失败率・法术书 `r` 学习／20,000回合遗忘、18种魔杖与许愿 `z`、12种戒指与6种护符／救命复活、体型武器伤害／银伤）、52格（`a-z`/`A-Z`）重用与堆叠、战争迷雾、未鉴定BUC隐藏与商店（`$`）与升天追击的主地城6层＋矿坑2层，支持XP成长与升天。Enter保留旧2层模式。q选药水、Q确认退出、S/L持久存读档，没有自动存档。

AIHack은 NetHack 3.6.7의 관찰 가능한 규칙을 시나리오와 테스트로 재구현하는 Rust 로그라이크다. 줄 단위 C-to-Rust 번역이 아니라 결정론적 core, 출처가 추적되는 호환 규칙, 격리된 local LLM adapter를 목표로 한다.

### 현재 상태

- report 26의 최종 verifier successor SHA `1e84a94`는 [Actions `32660514315`](https://github.com/Yupkidangju/AIHack/actions/runs/32660514315)에서 Ubuntu/Windows 각 19개 step과 actual bundle을 모두 PASS했다. report 27은 그 기준선에서 allocator/registry, field-only causal, archive/calendar, TUI input과 local action 경계를 재개방했다.
- Rust 1.94.1과 ratatui 0.30/crossterm 0.29 계열은 repository에 고정됐다. report 24 implementation `2519bc8e0ede81c39f46b5778e62a41d4ca66901`은 [GitHub Actions run `32107862171`](https://github.com/Yupkidangju/AIHack/actions/runs/32107862171)에서 Ubuntu/Windows quality gate를 모두 PASS했다.
- R1 build 재현성과 R2 private state/transaction은 local gate를 통과했다.
- R3의 fallible `ContentRegistry` bootstrap과 R4의 policy 기반 headless runner는 local 검증을 통과했다.
- `survival-v1`은 seed 42, 7, 1234에서 각각 1000 accepted turn과 3회 동일 hash를 검증한다. R5 workspace·문서 시정은 `audit_report_9.md` 재감사를 통과했다.
- `audit_report_11.md`는 보고서 10의 public/schema contract와 evidence 재현성 시정을 모두 Verified하고 R6 checkpoint를 PASS로 종결했다. 실제 model provider smoke는 비차단 고려 대상이다.
- R7의 공식 checksum inventory, legacy 격리 gate, NH367-C001..C010 record/test와 독립 재감사는 완료되어 `PASS WITH KNOWN RISKS`다. 2026-07-20 프로젝트 소유자는 AIHack을 NetHack 3.6.7의 AI-assisted semantic rewrite 파생물로 분류하고 전체 배포에 NGPL을 승인했다.
- 실제 외부 게시에는 R8 기술 감사 `PASS`, `LICENSE`, `NOTICE`, 해당 바이너리를 만든 커밋의 complete corresponding source archive가 필요하다.
- project-owner 결정은 `AIHACK-OWNER-2026-07-20-NGPL-01`로 추적하며, release bundle은 `PROJECT_OWNER_LICENSE_APPROVAL.md`, `MODIFICATIONS.md`, commit-bound `RELEASE-METADATA`와 `SHA256SUMS`를 포함한다. metadata key는 정확히 한 번 존재하고 owner/modification ID의 전체 값이 bundled record와 일치해야 하며 qualified legal opinion은 별도로 주장하지 않는다.
- `audit_report_21.md`는 report 20의 `IMP-F016`/`DBG-F008` 시정을 PASS로 종결했다. report 20의 재감사 대기는 더 이상 현재 상태가 아니다.
- `docs/audit/audit_report_24.md`와 implementation SHA `2519bc8e`의 양 OS CI는 report 23/24 시정을 역사적으로 종결했다. report 23 재감사 대기는 현재 gate가 아니다.
- 현재 권위는 `docs/audit/audit_report_32.md`다.
- Report 30 technical successor SHA `ed02dbf`의 [Actions `32733235414`](https://github.com/Yupkidangju/AIHack/actions/runs/32733235414)는 전체 gate와 clean same-SHA Ubuntu/Windows actual bundle을 PASS했고 Report 31이 public visibility와 Report 29 기술 회귀를 독립 확인했다.
- Report 31 summary lifecycle 시정과 FIN-F012는 successor `8c042d48/32741917348` 및 Report 32 독립 재감사에서 종결됐다.
- Report 32는 current HEAD 날짜가 modification period를 넘긴 R32-DBG-F001/FIN-F015를 열었다. 2026-08-25 Notice ID/period와 actual HEAD-date gate, final same-SHA 양 OS bundle 및 후속 독립 PASS와 별도 게시 승인 전에는 program/외부 게시 HOLD다.
- Windows all-target test는 dev-only `portable-pty 0.9.0` ConPTY에서 실제 TUI의 one-key state, mouse click, Inventory/Esc와 terminal restore를 검증한다. release binary dependency에는 포함되지 않는다.
- GitHub YAML action pin gate는 dev-only `saphyr 0.0.12`로 모든 workflow/composite node와 repository-root local action을 재귀 검사하며 release binary dependency에는 포함되지 않는다.
- R9 콘텐츠 인과 폐쇄는 음식·시체 섭취, 콘텐츠 기반 armor/monster behavior, 가격·난이도·gold·score, 기도·luck 전이를 실제 월드 상태에 연결한다. seed 42/7/1234 장기 테스트는 9종 semantic witness와 1000 accepted turn, 반복 hash를 함께 검증한다.

### v0.3.0 목표

- Rust 1.94.1, locked dependency, Linux/Windows CI
- private state와 transaction/invariant 기반 turn commit
- embedded TOML `ContentRegistry` 실연결
- accepted turn 1000개의 multi-seed deterministic 검증
- core/content/AI contract/LLM/TUI/headless workspace 분리
- loopback local LLM narrative, legal-action suggestion, presentation-only soft judgment
- NetHack 3.6.7 출처와 NH367 compatibility scenario 추적

### 현재 실행

```bash
cargo run --locked -- --seed 42
cargo run --locked -p aihack-headless --bin aihack-headless -- --seed 42 --turns 1000 --policy survival-v1
```

TUI는 `default-run = "aihack"`로 선택된다. headless 실행에는 package와 binary를 `-p aihack-headless --bin aihack-headless`로 명시한다.

### 구현 순서

1. `docs/audit/audit_report_32.md` R32-DBG-F001/FIN-F015의 ADR-0042 release-date evidence 시정과 final same-SHA 양 OS 검증
2. 후속 독립 PASS 뒤 별도 승인에 따른 외부 게시

### 문서

1. [Master specification](spec.md)
2. [Implementation tasks](IMPLEMENTATION_SUMMARY.md)
3. [Gap register](GAP_CLOSURE_ROADMAP.md)
4. [Runtime and UI design](designs.md)
5. [Architecture decisions](DESIGN_DECISIONS.md)
6. [Build guide](BUILD_GUIDE.md)
7. [Audit gates](audit_roadmap.md)
8. [Provenance policy](PROVENANCE.md)
9. [Compatibility template](docs/compatibility/README.md)
10. [Change history](CHANGELOG.md)
11. [R0 documentation audit](DOCUMENTATION_AUDIT_REPORT.md)
12. [R8 remediation closure](docs/audit/audit_report_21.md)
13. [Current re-audit](docs/audit/audit_report_32.md), [current remediation](docs/audit/audit_report_32_remediation.md), [Report 31 independent closure](docs/audit/audit_report_31_remediation.md), [historical final audit](docs/multi_audit/1/final_audit_report_1.md)
14. [NetHack General Public License](LICENSE)와 [derivative-work notice](NOTICE)
15. [Project-owner license decision](PROJECT_OWNER_LICENSE_APPROVAL.md)과 [modification manifest](MODIFICATIONS.md)

과거 Phase 계획은 `.archive/`의 불변 snapshot으로 보존한다.

## English

AIHack is a Rust roguelike that reimplements observable NetHack 3.6.7 behavior through traced scenarios and tests. It targets a deterministic core, provenance-backed compatibility rules, and an isolated local-LLM adapter—not a line-by-line C-to-Rust translation.

### Current status

- Report 26's final verifier successor SHA `1e84a94` passed all 19 steps and the actual bundle on both Ubuntu and Windows in [Actions `32660514315`](https://github.com/Yupkidangju/AIHack/actions/runs/32660514315). Report 27 reopened allocator/registry, field-only causal, archive/calendar, TUI input, and local-action boundaries on that baseline.
- Rust 1.94.1 and the ratatui 0.30/crossterm 0.29 family are pinned. Report 24 implementation `2519bc8e0ede81c39f46b5778e62a41d4ca66901` passed both OS quality gates in [GitHub Actions run `32107862171`](https://github.com/Yupkidangju/AIHack/actions/runs/32107862171).
- R1 build reproducibility and R2 private-state transaction gates pass locally.
- R3's fallible `ContentRegistry` bootstrap and R4's policy-driven headless runner pass local verification.
- `survival-v1` verifies 1,000 accepted turns and three matching hashes for seeds 42, 7, and 1234. The R5 workspace and documentation remediation passed the `audit_report_9.md` re-audit.
- `audit_report_11.md` verified the public/schema-contract and reproducibility remediation from report 10 and closed the R6 checkpoint with PASS. A real-provider smoke remains non-blocking.
- R7 passes its engineering scope with known risks after verified checksums, a fail-closed legacy gate, and ten strengthened NH367 traces. On 2026-07-20 the project owner classified AIHack as an AI-assisted semantic-rewrite derivative of NetHack 3.6.7 and approved NGPL for the whole distribution.
- External publication still requires an R8 technical-audit PASS plus LICENSE, NOTICE, and the complete corresponding source for the released binaries.
- The project-owner decision is traceable as `AIHACK-OWNER-2026-07-20-NGPL-01`; release bundles carry PROJECT_OWNER_LICENSE_APPROVAL.md, MODIFICATIONS.md, commit-bound RELEASE-METADATA, and SHA256SUMS. Metadata IDs must resolve to the bundled records. No qualified legal opinion is claimed.
- `audit_report_21.md` closed report 20's documentation remediation with PASS; report 20 is no longer the current pending authority.
- Report 24 and the two-OS CI for implementation SHA `2519bc8e` historically closed report 23/24 remediation; report 23 is no longer the current pending gate.
- The current authority is `docs/audit/audit_report_32.md`.
- Report 30 technical successor SHA `ed02dbf` passed the full gate and clean same-SHA Ubuntu/Windows actual bundles in [Actions `32733235414`](https://github.com/Yupkidangju/AIHack/actions/runs/32733235414), and Report 31 independently verified the public-visibility and Report 29 technical regressions.
- Report 31 summary-lifecycle remediation and FIN-F012 were independently closed by Report 32 with historical successor evidence `8c042d48/32741917348`.
- Report 32 opened R32-DBG-F001/FIN-F015 because the current HEAD date exceeds the modification period. Program PASS and publication remain blocked until the 2026-08-25 notice/period, actual HEAD-date gate, final same-SHA dual-OS bundles, an independent PASS, and separate publication approval are complete.
- Windows all-target tests use dev-only `portable-pty 0.9.0` ConPTY to exercise real TUI one-key states, mouse input, Inventory/Esc, and terminal restoration; it is not a release-binary dependency.
- The action-pin gate structurally scans every workflow/composite YAML node and recursively resolves repository-root local actions with dev-only `saphyr 0.0.12`; it is not a release-binary dependency.
- R9 causal closure connects food/corpse consumption, content-driven armor and monster behavior, price/difficulty/gold/score, and prayer/luck to observable world-state changes. Long runs for seeds 42/7/1234 now require all nine semantic witnesses as well as 1,000 accepted turns and repeatable hashes.

### v0.3.0 target

- Rust 1.94.1, locked dependencies, Linux/Windows CI
- Private state and transaction/invariant turn commits
- Runtime-backed embedded TOML `ContentRegistry`
- Deterministic 1,000-accepted-turn multi-seed verification
- Core/content/AI-contract/LLM/TUI/headless workspace boundaries
- Loopback local-LLM narrative, legal-action suggestions, and presentation-only soft judgments
- Traceable NetHack 3.6.7 sources and NH367 compatibility scenarios

### Run the current code

```bash
cargo run --locked -- --seed 42
cargo run --locked -p aihack-headless --bin aihack-headless -- --seed 42 --turns 1000 --policy survival-v1
```

The TUI is selected through `default-run = "aihack"`; select the headless package and binary with `-p aihack-headless --bin aihack-headless`. See the Korean section for the ordered document set.

## 日本語

AIHack は、NetHack 3.6.7 の観察可能な挙動を、出典付きシナリオとテストで再実装する Rust ローグライクです。C ソースの行単位変換ではなく、決定論的 core、追跡可能な互換ルール、隔離された local LLM adapter を目標にします。

### 現在の状態

- report 26 の最終 verifier successor SHA `1e84a94` は Actions `32660514315` で Ubuntu/Windows 各 19 step と実 bundle をすべて PASS しました。report 27 はその基準から allocator/registry、field-only causal、archive/calendar、TUI input、local action 境界を再開しました。
- Rust 1.94.1 と ratatui 0.30/crossterm 0.29 は固定済みです。R5 workspace と文書是正は `audit_report_9.md` の再監査を通過しました。
- `audit_report_11.md` は、report 10 の public/schema contract と証拠再現性の是正をすべて Verified とし、R6 checkpoint を PASS で完了しました。実 provider smoke は非 blocking の検討事項です。
- AIHack は NetHack 3.6.7 の AI-assisted semantic rewrite による派生物としてプロジェクト所有者に承認され、配布物全体に NGPL を適用します。外部公開には R8 技術監査 PASS と LICENSE、NOTICE、対応する完全なソースが必要です。
- R9 の因果閉包では、食料・死体、armor、monster behavior、価格・難易度・gold・score、祈り・luck を実際の world state 変化へ接続し、seed 42/7/1234 の長期 test で semantic delta を検証します。
- report 24 と implementation SHA `2519bc8e` の両 OS CI により report 23/24 の修正は履歴上完了し、report 23 は現在の保留 gate ではありません。
- 現在の権威は `docs/audit/audit_report_32.md` です。
- Report 30 の technical successor SHA `ed02dbf` は Actions `32733235414` で全 gate と clean same-SHA Ubuntu/Windows 実 bundle を PASS し、Report 31 は public visibility と Report 29 の技術回帰を独立確認しました。
- Report 31 summary lifecycle 是正と FIN-F012 は historical successor `8c042d48/32741917348` を根拠に Report 32 で独立完了しました。
- Report 32 は current HEAD 日付が modification period を超えた R32-DBG-F001/FIN-F015 を開始しました。2026-08-25 notice/period、actual HEAD-date gate、final same-SHA 両 OS bundle、独立 PASS、別途公開承認までは program PASS と外部公開を保留します。
- Windows の all-target test は dev-only `portable-pty 0.9.0` ConPTY で実 TUI の一キー遷移、mouse、Inventory/Esc、terminal 復元を検証し、release binary には含めません。
- action pin gate は dev-only `saphyr 0.0.12` で全 workflow/composite YAML node と repository-root local action を再帰検査し、release binary には含めません。

### 現在の実行

```bash
cargo run --locked -- --seed 42
cargo run --locked -p aihack-headless --bin aihack-headless -- --seed 42 --turns 1000 --policy survival-v1
```

TUI は `default-run = "aihack"` で選択されます。headless は `-p aihack-headless --bin aihack-headless` で package と binary を指定します。実装順序と文書一覧は韓国語セクションを参照してください。

## 繁體中文

AIHack 是以具來源追蹤的情境與測試，重新實作 NetHack 3.6.7 可觀察行為的 Rust roguelike。目標不是逐行翻譯 C，而是 deterministic core、可追蹤的相容規則，以及隔離的 local LLM adapter。

### 目前狀態

- report 26 的最終 verifier successor SHA `1e84a94` 已在 Actions `32660514315` 通過 Ubuntu/Windows 各 19 個 step 與實際 bundle。report 27 在此基準上重新開啟 allocator/registry、field-only causal、archive/calendar、TUI input 與 local action 邊界。
- Rust 1.94.1 與 ratatui 0.30/crossterm 0.29 已固定；R5 workspace 與文件修正已通過 `audit_report_9.md` 複審。
- `audit_report_11.md` 已驗證 report 10 的 public/schema contract 與證據可重現性修正，並以 PASS 完成 R6 checkpoint；實際 provider smoke 仍為非阻斷考量。
- 專案所有者已將 AIHack 核准為 NetHack 3.6.7 的 AI-assisted semantic rewrite 衍生作品，整體散布適用 NGPL。對外發布仍需 R8 技術稽核 PASS、LICENSE、NOTICE 與對應完整原始碼。
- R9 因果閉合把食物／屍體、armor、monster behavior、價格／難度／gold／score、祈禱／luck 連到實際 world state 變化，並以 seed 42/7/1234 長期測試驗證 semantic delta。
- report 24 與 implementation SHA `2519bc8e` 的雙作業系統 CI 已在歷史上結束 report 23/24 修正；report 23 不再是目前待處理 gate。
- 目前權威是 `docs/audit/audit_report_32.md`。
- Report 30 technical successor SHA `ed02dbf` 的 Actions `32733235414` 已通過完整 gate 與 clean same-SHA Ubuntu/Windows 實際 bundle，Report 31 也獨立驗證 public visibility 與 Report 29 技術回歸。
- Report 31 summary lifecycle 修正與 FIN-F012 已由 Report 32 依 historical successor `8c042d48/32741917348` 獨立結案。
- Report 32 因 current HEAD 日期超出 modification period 而開啟 R32-DBG-F001/FIN-F015。完成 2026-08-25 notice/period、actual HEAD-date gate、final same-SHA 雙 OS bundle、獨立 PASS 與另行發布核准前，program PASS 與對外發布維持暫停。
- Windows all-target 測試使用僅供開發的 `portable-pty 0.9.0` ConPTY 驗證實際 TUI 的單鍵狀態、滑鼠、Inventory/Esc 與終端復原，不納入 release binary 相依性。
- action pin gate 使用僅供開發的 `saphyr 0.0.12` 結構化掃描全部 workflow/composite YAML node，並遞迴解析 repository-root local action，不納入 release binary 相依性。

### 目前執行

```bash
cargo run --locked -- --seed 42
cargo run --locked -p aihack-headless --bin aihack-headless -- --seed 42 --turns 1000 --policy survival-v1
```

TUI 由 `default-run = "aihack"` 選取；headless 以 `-p aihack-headless --bin aihack-headless` 指定 package 與 binary。實作順序與文件列表請參閱韓文段落。

## 简体中文

AIHack 是通过带来源追踪的场景和测试，重新实现 NetHack 3.6.7 可观察行为的 Rust roguelike。目标不是逐行翻译 C，而是 deterministic core、可追踪的兼容规则，以及隔离的 local LLM adapter。

### 当前状态

- report 26 的最终 verifier successor SHA `1e84a94` 已在 Actions `32660514315` 通过 Ubuntu/Windows 各 19 个 step 与实际 bundle。report 27 在此基线上重新打开 allocator/registry、field-only causal、archive/calendar、TUI input 与 local action 边界。
- Rust 1.94.1 和 ratatui 0.30/crossterm 0.29 已固定；R5 workspace 与文档修正已通过 `audit_report_9.md` 复审。
- `audit_report_11.md` 已验证 report 10 的 public/schema contract 与证据可复现性修正，并以 PASS 完成 R6 checkpoint；实际 provider smoke 仍为非阻断考虑项。
- 项目所有者已将 AIHack 批准为 NetHack 3.6.7 的 AI-assisted semantic rewrite 衍生作品，整体分发适用 NGPL。对外发布仍需 R8 技术审计 PASS、LICENSE、NOTICE 与对应完整源代码。
- R9 因果闭合将食物／尸体、armor、monster behavior、价格／难度／gold／score、祈祷／luck 连接到实际 world state 变化，并用 seed 42/7/1234 长期测试验证 semantic delta。
- report 24 与 implementation SHA `2519bc8e` 的双操作系统 CI 已在历史上结束 report 23/24 修正；report 23 不再是当前待处理 gate。
- 当前权威是 `docs/audit/audit_report_32.md`。
- Report 30 technical successor SHA `ed02dbf` 的 Actions `32733235414` 已通过完整 gate 与 clean same-SHA Ubuntu/Windows 实际 bundle，Report 31 也独立验证了 public visibility 与 Report 29 技术回归。
- Report 31 summary lifecycle 修正与 FIN-F012 已由 Report 32 根据 historical successor `8c042d48/32741917348` 独立结案。
- Report 32 因 current HEAD 日期超出 modification period 而开启 R32-DBG-F001/FIN-F015。完成 2026-08-25 notice/period、actual HEAD-date gate、final same-SHA 双 OS bundle、独立 PASS 与另行发布批准前，program PASS 和对外发布保持暂停。
- Windows all-target 测试使用仅供开发的 `portable-pty 0.9.0` ConPTY 验证实际 TUI 的单键状态、鼠标、Inventory/Esc 与终端恢复，不纳入 release binary 依赖。
- action pin gate 使用仅供开发的 `saphyr 0.0.12` 结构化扫描全部 workflow/composite YAML node，并递归解析 repository-root local action，不纳入 release binary 依赖。

### 当前运行

```bash
cargo run --locked -- --seed 42
cargo run --locked -p aihack-headless --bin aihack-headless -- --seed 42 --turns 1000 --policy survival-v1
```

TUI 由 `default-run = "aihack"` 选择；headless 使用 `-p aihack-headless --bin aihack-headless` 指定 package 和 binary。实现顺序和文档列表请参阅韩文部分。
