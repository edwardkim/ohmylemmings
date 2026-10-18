# G05-02 시작 시계·입구·생성 스케줄 진행 기록

작업 이슈: [GitHub #24](https://github.com/edwardkim/ohmylemmings/issues/24). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 범위와 순서

번호·URL·OPEN 상태를 확인했습니다. [#5 시간/입력 계약](../tech_insights/issue-5_compatibility_baseline.md#입력상태-기록-프로토콜)과 [#23 골격](../tech_insights/issue-23_fixed_ticks.md)을 읽고 계약 → 합성 Red/Green → 독립 모델/재생 대조 → 문서·검사·게시·실제 CI 순서로 진행합니다.

생성 요청의 안정된 ID/입구 선택만 다루며 실제 actor·좌표/행동·전체 세션 종료·Nuke·음원 재생·리소스 연결·전체 trace/실제 DOS 실행은 후속 범위입니다. G05 전체는 진행 중입니다.

## 현재 상태

시작 스케줄러의 구현·합성 경계·전수 재생·공통 검사·게시·실제 CI·원격 tree를 확인했습니다. G05 전체는 개체 갱신·상태 추적/기준 실행 대조가 남아 진행 중입니다.

## 수행과 근거

- [계약·독립 모델·재현 명령](../tech_insights/issue-24_startup_schedule.md), [결과 보고서](../reports/issue-24_startup_schedule.md).
- 새 17개 중 16개 Red·초기 상태 1개 및 기존 코어 62개/리소스 181개 통과에서 Green 코어 79개를 확인했습니다.
- 4000개 설정·세 프레임 패턴·12000회 재생·4080000제어 호출의 매 호출 상태와 전체 이벤트를 독립 모델에 대조했습니다. Python/Rust 서명 81365042e5aa5fed가 같습니다.
- make check: Rust 단위 260개·문서 테스트 25개(기존 compile-fail 2개 포함)·Python 8개·포맷·Clippy·빌드·WASM 성공. 테스트 보조 입력의 불필요한 Vec를 배열로 바꿔 Clippy를 충족했습니다.
- 문서의 세 sh 블록을 그대로 실행했고 기존 소스 46개·관련 2개 원문의 크기/SHA-256/Git blob을 확인했습니다.

- Markdown 87개·로컬 링크 601개·앵커 105개·이슈 문서명 75개·#24 일지와 변경 파일 12개의 범위·언어/공백을 확인했습니다. cargo metadata에서 패키지 2개·코어 의존 0개·리소스→코어 로컬 의존 하나를 확인했습니다.

## 다음 행동

구현 `6094a6ed7f7d2144ffd8d2e7ba65d4233ef3ff4b`의 [실제 CI 세 작업](https://github.com/edwardkim/ohmylemmings/actions/runs/39131350558)·로그·원격 파일 144개를 확인했습니다. 최종 문서 게시·CI·상위 #1 갱신·#24 종료는 GitHub 이슈 기록에서 추적합니다. 다음 후보는 별도 이슈의 G05-03 걷기·낙하·방향 전환·제거입니다.
