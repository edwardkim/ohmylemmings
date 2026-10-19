# G08-06 포인터 선택·RightClickGlitch

이슈: [#58](https://github.com/edwardkim/ohmylemmings/issues/58) · 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1)

## 목표

고정 Lemmix/DOS의 포인터 선택 범위·우선순위와 우클릭 원격 할당을 확인하고 네이티브 클릭/호버를 같은 선택 결과에 연결합니다. 함정 사망 예외, 입력 상태 초기화, 안정 ID 재생을 포함합니다.

## 진행 상태

- 원격 main `7396503` 동기화 후 #58 번호/URL을 확인하고 분기했습니다.
- no_std 선택·안정 ID 대체/원격 할당·삭제 대상 보존·HUD/커서/경계 기록을 연결했습니다.
- 원문53함수/표의 선택44604·원격2·대체3888, 총48494행이 일치합니다. 합성8재생2728상태288캡처와 실제 X11 합성/함정/원본 Fun03 입력의 독립 재생을 확인했습니다.
- [계약](../tech_insights/issue-58_pointer_selection.md)·[보고서](../reports/issue-58_pointer_selection.md)·[출처 해시](../../assets/manifests/pointer_selection.json)를 기록했습니다.
- 코드 자체 검토와 재시작/삭제 대상/극값 회귀를 마쳤습니다. 정확한 공통 검사·한글 PR/CI·병합/이슈 완료 상태는 #58에서 추적합니다.
- 다음 개발 후보는 G08 한 레벨 통합 완주 검증입니다.

## 검증 계획

경계/겹침/행동 우선순위·우클릭·함정 사망·포커스/결과/재시작을 합성 테스트로 고정합니다. 원문 실제 실행 또는 DOS 관측과 독립 대조하고 SDL 실제 입력·안정 ID 재생·전체 관련 회귀·자체 검토·한글 PR/CI를 확인합니다. 원본 파생 자료는 ignored 경로에만 보관합니다.

G08 전체 완료와 한 레벨 통합 완주는 후속입니다.
