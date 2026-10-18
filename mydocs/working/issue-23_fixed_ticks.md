# G05-01 고정 틱·입력 순서 골격 진행 기록

작업 이슈: [GitHub #23](https://github.com/edwardkim/ohmylemmings/issues/23). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 범위와 순서

번호·URL·OPEN 상태를 확인했습니다. [호환성 기준](../tech_insights/issue-5_compatibility_baseline.md#입력상태-기록-프로토콜)의 경계 입력·제어/진행 순서를 읽고 계약 → 합성 Red/Green → 프레임 분할·재생 대조 → 문서·검사·게시·실제 CI 순서로 진행합니다.

시간 누적기는 어댑터 소유이며 코어 상태와 분리합니다. Pause/Resume은 진행 통지만 제어하고 PauseGlitch·시작 시계/입구/생성·개체/직업·전체 trace·실제 DOS 실행은 후속 범위입니다. G05 전체는 진행 중입니다.

## 현재 상태

시간 누적기·입력 일정·경계 처리기의 구현·합성 전수 대조·공통 검사·게시·실제 CI·원격 tree를 확인했습니다. G05 전체는 시작 시계/입구/생성·개체 갱신·기준 실행 대조가 남아 진행 중입니다.

## 수행과 근거

- [계약·독립 모델·재현 명령](../tech_insights/issue-23_fixed_ticks.md), [결과 보고서](../reports/issue-23_fixed_ticks.md).
- 새 22개 중 20개 Red·초기/빈 입력 2개 및 기존 코어 40개/리소스 181개 통과에서 Green 코어 62개를 확인했습니다.
- 모든 두 프레임 시간 분할 501501건과 명령 목록 5461개·16383회 재생·278511번 제어 호출을 독립 모델과 대조했습니다. Python/Rust 서명 5338d6ae252490ba가 같습니다.
- make check: Rust 단위 243개·문서 테스트 23개(기존 compile-fail 2개 포함)·Python 8개·포맷·Clippy·빌드·WASM 성공.
- 고정 Lemmix/TS의 시간·정지·갱신 순서 차이를 기록했고 기존 소스 46개를 확인했습니다. 실제 레퍼런스/DOS 실행 호환성 판정은 후속입니다.

- 문서의 세 sh 블록을 그대로 실행해 테스트·Python 서명·고정 원문 2개의 크기/SHA-256/Git blob을 재현했습니다.

- Markdown 84개·로컬 링크 578개·앵커 100개·이슈 문서명 72개·#23 일지와 변경 파일 13개의 범위·언어/공백을 확인했습니다. cargo metadata에서 패키지 2개·코어 의존 0개·리소스→코어 로컬 의존 하나를 확인했습니다.

## 다음 행동

구현 `8627dd3ceef2e3b80178c9b13b357405528851ba`의 [실제 CI 세 작업](https://github.com/edwardkim/ohmylemmings/actions/runs/39129905687)·로그·원격 파일 139개를 확인했습니다. 최종 문서 게시·CI·상위 #1 갱신·#23 종료는 GitHub 이슈 기록에서 추적합니다. 다음 후보는 별도 이슈의 G05-02 시작 시계·입구·생성 스케줄입니다.
