# G07-07 Climber 진행 기록

이슈: [#40](https://github.com/edwardkim/ohmylemmings/issues/40), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

사용자가 E키 Bomber 확인 후 다음 단계 진행을 승인했습니다. main1438c74·PR #39/이슈 #38 완료와 OPEN 작업을 확인하고 #40의 번호·URL·OPEN을 조회한 뒤 issue-40-climber 브랜치를 만들었습니다. 기존 #38 사용자 확인 문서 변경과 관계없는 미추적 폴더를 보존합니다.

영구 Climber 능력/수량·벽 오르기/Hoisting·경계·기존 여섯 직업/Bomber 상호작용을 no_std 코어에 구현합니다. C키/클릭/ID 재생·원본 표시와 실제 기준 함수·합성/원본/DOS 두 실행·회귀/전체 검사·self-review/게시/CI를 검증합니다. Floater/Nuke·구출/승패/음향·G07 전체/DOS 인증은 후속입니다.

현재 고정 AssignClimber/HandleWalking/HandleClimbing/HandleHoisting·메타데이터와 DOS 메커니즘을 읽고 TDD 계약을 고정합니다. 준비 작업 이슈는 만들지 않습니다.

## 구현·검증 상태

코어 새10개 테스트·네이티브 새2개와기존 숫자 앵커의 Climbing/Hoisting 확장·Python 새3개를 검증했습니다. 영구 능력·벽/머리/상단·Hoisting·Shrugging Once 버그·Bomber 만료·Blocker/제거 거부를 구현했습니다. 고정46함수 실제 실행과 합성10사례20재생8420상태1100캡처가 일치합니다. 전체 `make check`227.933초·Rust400/문서27/Python65·SDL·WASM과 기존 여섯 직업12재생4092상태304캡처/F00 회귀를 통과했습니다.

reference-1은 누락한 호스트 Blocker 플래그 준비 오류로 제외하고 reference-2를 최종으로 사용합니다. native-proof-1의 잘못된 Fun04 Hoisting 발생 전제, native-proof-2의 중간 바이너리 기록은 최종 native-final과 구분합니다. 실제 Fun04는 등반 후 머리 돌출에 반전·낙하하며 Hoisting은 합성/실제 원문 실행에서 검증합니다. DOS proof-1은 수량 전제1→실제10, proof-2는 등반 코드0x10→실제0x20 판정 보정, proof-3은 교체 중인 X11 창 핸들 때문에 최종 횟수에서 제외합니다. 같은 수정 컨트롤러의 proof-4/5를 최종 두 실행으로 검증합니다.

최종 native-final의4실행1364상태240캡처와 실제 C키/클릭 입력80 ID0이 모두 일치했습니다. DOS proof-4/5의1278샘플4761게임 프레임과136/160/180/200/220/260/300/340 전체320×160 장면×2회가 일치합니다. 원본/소스·각 DOS 사본·139소스/2255산출물 해시를 고정했습니다. [계약](../tech_insights/issue-40_climber.md)·[보고서](../reports/issue-40_climber.md)·[manifest](../../assets/manifests/climber.json)를 동기화했습니다.

## 통합 절차

self-review·문서/해시 확인을 마쳤습니다. PR을 게시하고 정확한 HEAD의 push/PR CI 세 작업·실제 로그·원격 파일을 확인합니다. 검토한 HEAD만 병합하고 main CI·이슈 종료·상위 상태를 확인합니다. 최종 통합은 [#40](https://github.com/edwardkim/ohmylemmings/issues/40)에서 추적합니다. 다음 후보는 Floater이며 G07 전체는 진행 중입니다.
