# G07-08 Floater 진행 기록

이슈: [#42](https://github.com/edwardkim/ohmylemmings/issues/42), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

사용자가 다음 단계 진행을 승인했습니다. main4f629ee·PR #41/이슈 #40 완료와 OPEN 목록을 확인하고 #42의 번호·URL·OPEN을 조회했습니다. issue-42-floater 브랜치를 만들었으며 관계없는 미추적 폴더를 보존합니다.

영구 능력/수량·16단계 펼침/이동·착지/하부 경계와 Floating 중 Bomber 만료를 TDD로 구현했습니다. F키/클릭/ID 재생·원본 우산/발 앵커와 능력/커서 추적을 지원합니다. 준비 작업 이슈는 만들지 않습니다.

## 구현·검증 상태

고정 원문48함수/FloatParametersTable·합성10사례20재생8420상태906캡처, 원본 Fun02 네이티브4실행1204상태92캡처와 실제 F키/클릭 경계54 ID0이 일치합니다. 별도 실제 Pause231/Resume277 실행301상태25캡처도 받은 입력 그대로 기준 실행과 대조했습니다. 원본 긴 낙하194에서 펼치고199에서상승·225에서착지하며300호출 경계까지 검증합니다.

DOS proof-1/2의1351샘플4749게임 프레임과 전체320×160 여덟 장면×두 실행이 일치합니다. 원본 표224바이트의 반복과 별도 능력 바이트255·수량10→9·Floating1024·표 커서/Walking 착지를 관측했습니다. 이전 준비/진단 실패는 최종 성공 횟수에서 제외합니다. 기존 일곱 직업14재생4774상태424캡처와F00 서명, 전체 `make check`293.272초·Rust412/문서27/Python69·SDL·WASM을 통과했습니다.

[계약](../tech_insights/issue-42_floater.md)·[보고서](../reports/issue-42_floater.md)·[148소스/2324산출물 manifest](../../assets/manifests/floater.json)를 동기화했습니다. 최종 바이너리5cbce020·기준a738c516의 검증 근거와 자체 합성 이미지/원본 ignored 자료를 보존합니다.

## 통합 절차

self-review 후 PR을 게시하고 정확한 HEAD의 push/PR CI 세 작업·실제 로그·원격 파일을 확인합니다. 검토한 HEAD만 병합하고 main CI·이슈 종료·상위 상태를 확인합니다. 최종 통합은 [#42](https://github.com/edwardkim/ohmylemmings/issues/42)에서 추적합니다. 다음 후보는 Nuke이며 전체 상호작용·객체/구출/승패/음향·G07 전체/DOS 인증은 후속입니다.

## 통합 완료

PR [#43](https://github.com/edwardkim/ohmylemmings/pull/43)의 검토 HEAD a248f02f20d492f6dd37ce422ef69a9b71b201dc와 push/PR CI를 확인하고 main feab3e8851a30fdf6d32845953f3ac903a3d648e로 병합했습니다. [main CI39221581981](https://github.com/edwardkim/ohmylemmings/actions/runs/39221581981)의 세 작업/실제 로그와 #42 종료·상위 상태를 확인했습니다. 다음 승인된 작업은 [#44 Nuke](issue-44_nuke.md)입니다.
