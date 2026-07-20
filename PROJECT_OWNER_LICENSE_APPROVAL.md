# AIHack Project-Owner License Decision Record

Approval ID: `AIHACK-OWNER-2026-07-20-NGPL-01`  
Decision date: 2026-07-20 (Asia/Seoul)  
Decision authority: Project owner  
Record status: direct user instruction recorded; immutable Git reference pending commit  
qualified legal opinion: not claimed

## 1. Direct decision source

이 record는 coder가 라이선스 권한을 대신 행사한 문서가 아니라, 프로젝트 소유자가 현재 개발 대화에서 직접 내린 다음 지시를 저장소에 연결한 것이다.

> 이 프로젝트는 nethack 3.6.7의 원본소스를 AI의 추론기능으로 의도를 추춣하여 재작성 했습니다. 때문에 소스상으로는 매우 다를 수 있으나, 원본의 저작권은 존중해야하는 파생물로 보고 있습니다. 이 부분을 고려하여 라이센스를 정리할수 있습니까?

제안된 whole-work NGPL, 공식 NetHack 3.6.7 license 원문, 파생·변경 notice와 corresponding-source 방향에 대해 프로젝트 소유자는 다음과 같이 확인했다.

> 네 지금 방향이 좋습니다. 이렇게 가죠.

Evidence channel: direct user instruction in the AIHack implementation session. 이 문서의 Git object와 commit SHA가 생성되면 해당 commit이 저장소 내 immutable approval reference가 된다.

## 2. Approved scope

프로젝트 소유자의 결정 범위는 다음과 같다.

- AIHack 전체를 NetHack 3.6.7 원본 source에서 의도와 규칙을 추론해 재작성한 파생물로 취급한다.
- AIHack whole-work NGPL 배포와 root 공식 `LICENSE` 사용을 승인한다.
- `PROV-0001..PROV-0012`의 포함·격리 상태와 `NH367-C001..NH367-C010`의 source-review record를 이 결정에 연결한다.
- runtime에 포함되는 AIHack source/content와 배포 문서에는 NGPL, modification notice와 complete corresponding source 제공 계약을 적용한다.
- `legacy_nethack_port_reference/**`는 승인된 runtime/release source가 아니며 계속 격리한다.
- crates.io 개별 publication은 승인하지 않으며 `publish = false`를 유지한다.

## 3. Notice and source decision

배포 bundle에는 다음을 함께 제공한다.

- 공식 NetHack 3.6.7 원문과 일치하는 `LICENSE`
- 파생·수정 사실과 원 저작권을 밝히는 `NOTICE`
- 변경 범위와 날짜를 수신자가 확인할 수 있는 `MODIFICATIONS.md`
- 이 승인 ID와 범위를 수신자가 확인할 수 있는 `PROJECT_OWNER_LICENSE_APPROVAL.md`
- version과 release commit을 식별하는 `RELEASE-METADATA`
- 해당 binary를 만든 clean commit의 complete corresponding source archive
- binary와 문서, source archive를 묶는 `SHA256SUMS`

archive와 output의 `RELEASE-METADATA`는 `owner_approval=AIHACK-OWNER-2026-07-20-NGPL-01`과 `modification_notice=AIHACK-MODIFICATIONS-2026-07-20-01`을 포함하며, 두 값은 함께 배포되는 각 record의 ID와 일치해야 한다.

이 방식은 저장소가 채택한 보수적 engineering distribution contract다. 본 승인 기록은 변호사 또는 별도 qualified license reviewer의 법률 의견이라고 주장하지 않는다.

## 4. Approval boundary

이 결정은 라이선스 방향과 프로젝트 내부 배포 계약을 승인하지만 다음을 자동 승인하지 않는다.

- 독립 R8 감사 `PASS`
- Linux/Windows 동일 commit CI 완료
- 실제 외부 게시, release 또는 배포
- NGPL 의무에 관한 최종 법적 판단

외부 게시 권한은 clean commit package, 양 OS CI와 독립 R8 재감사가 모두 통과한 뒤 별도로 행사한다.
