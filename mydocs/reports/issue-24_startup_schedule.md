# G05-02 시작 시계·입구·생성 스케줄 결과

작업 이슈: [GitHub #24](https://github.com/edwardkim/ohmylemmings/issues/24). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

- [계약·순서·경계 fixture·독립 모델·재현 명령](../tech_insights/issue-24_startup_schedule.md)
- [진행 기록](../working/issue-24_startup_schedule.md), [작업일지](../orders/20261018.md#issue-24)

## 변경 내용

기존 TickStepper·입력 일정을 재사용하는 StartupScheduler를 추가했습니다. 17진행 틱 시계·초기 PauseGlitch·시작 이벤트·입구 개방·생성 카운트다운을 처리합니다. 생성 요청은 안정된 ID·입구 순번·생성 틱을 포함하며 실제 actor/좌표를 만들지 않습니다. 0~4개 입구·생성 상한·u32 최댓값을 처리합니다.

SetReleaseRate는 초기 RR~99를 수락하고 거부를 명시하며 진행 중인 카운트다운을 유지합니다. 모든 입력을 통지한 뒤 시작/생성 이벤트와 Tick 순서로 전달합니다. 시계 0은 포화하고 전체 종료 판정은 상위 게임 단계가 담당합니다. 코어 no_std·무할당/I/O·외부 의존성 0개와 리소스→코어 로컬 의존 하나를 유지했습니다.

## 검증

| 검사 | 결과 |
|---|---|
| 합성 TDD | 새 17개 중 16개 Red·초기 상태 1개 및 기존 243개 통과 → Green 코어 79개/리소스 181개 |
| make check | Rust 단위 260개·문서 테스트 25개(compile-fail 2개 포함)·Python 8개·포맷·Clippy·빌드·WASM 성공 |
| 경계 fixture | F00 15/34/35/54, F02 RR98/99의 기존 카운트다운, F03 Pause/Resume·glitch 18회·시계/생성 결과 확인 |
| 전수 재생 | 초기 RR 100개×입구 5개×glitch 2종×시나리오 4종 = 4000개·세 패턴 12000회·4080000제어 호출 |
| 독립 모델 | 매 호출 상태와 전체 이벤트 일치, 닫힌식 Python/Rust 서명 81365042e5aa5fed |
| 문서·추적 규칙 | Markdown 87개·로컬 링크 601개·앵커 105개·이슈 문서명 75개·#24 일지 확인 |
| 의존 경계 | 패키지 2개·코어 의존 0개·리소스→코어 로컬 의존 하나·기존 입력/리소스 코드 보존 |
| 문서 재현 | 세 sh 블록 그대로 실행, Rust/Python 서명과 고정 소스 46개·관련 원문 2개 식별 값 확인 |
| 예외/경계 | 모든 범위 밖 설정·최대 분/총수·시간 0·입구/총수 0·순서/거부·초기/반복/임계 Pause·최종 ID·한계 무통지 |

고정 Lemmix를 정적으로 분석하고 TS의 다른 생성 간격/첫 입구만 사용하는 차이를 기록했습니다. 레퍼런스 엔진/DOS 실행을 대조한 것으로 표시하지 않습니다. UI 클램핑·강제 진행·리플레이 헤더와 실시간 Pause 0의 차이·음원·Nuke·전체 성패·actor/지형/리소스 연결·전체 trace는 후속 범위입니다.

## 게시 상태와 다음 범위

구현 커밋 `6094a6ed7f7d2144ffd8d2e7ba65d4233ef3ff4b`을 main에 게시했습니다. [push CI 실행](https://github.com/edwardkim/ohmylemmings/actions/runs/39131350558)의 결론과 세 작업이 모두 success입니다.

| 작업 | 실제 실행 근거 |
|---|---|
| [Python tooling](https://github.com/edwardkim/ohmylemmings/actions/runs/39131350558/job/117687012157) | Python 3.12.15, 테스트 8개 성공 |
| [Rust native](https://github.com/edwardkim/ohmylemmings/actions/runs/39131350558/job/117687012059) | Rust 1.93.1, 포맷·Clippy·빌드·단위 260개·문서 테스트 25개·F00/F02/F03·전수 재생·기존 별칭 거부 성공 |
| [WASM libraries](https://github.com/edwardkim/ohmylemmings/actions/runs/39131350558/job/117687012015) | 고정 Rust·WASM 타깃, 두 라이브러리 컴파일 성공 |

push 이벤트·대상 SHA·실제 로그·테스트 개수·시작 경계/재생/기존 compile-fail·clean-input 성공을 대조했습니다. 원본·기존 target·Cargo 패키지 캐시 없는 러너에서 통과했습니다. 원격 tree API의 파일 144개 모드·blob 해시가 로컬과 일치했고 작업 트리는 깨끗합니다. 원본·참조 소스 사본·빌드 출력을 추적하지 않습니다.

G05-02 범위를 완료했습니다. 최종 문서 게시·CI·상위 #1 갱신·#24 종료는 GitHub 이슈 기록에서 추적합니다.

G05 전체·COMP-04~06은 완료로 처리하지 않습니다. 다음은 별도 이슈의 **G05-03 걷기·낙하·방향 전환·제거**입니다.
