# G05-01 고정 틱·입력 순서 골격 결과

작업 이슈: [GitHub #23](https://github.com/edwardkim/ohmylemmings/issues/23). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

- [계약·호출 순서·독립 모델·재현 명령](../tech_insights/issue-23_fixed_ticks.md)
- [진행 기록](../working/issue-23_fixed_ticks.md), [작업일지](../orders/20261018.md#issue-23)

## 변경 내용

어댑터 소유의 `FixedTickClock`이 정수 나노초로 17Hz 제어 호출 수와 잔여 위상을 계산합니다. 프레임 분할과 관계없이 같은 틱 수와 잔여 위상을 반환하며 묵시적인 과부하 틱 폐기를 하지 않습니다. `InputSchedule`은 불변 입력을 `(control_step, sequence)` 순서로 검증하고 경계별로 빌려줍니다. 중복·역행의 첫 인덱스를 오류로 반환하며 payload를 복사하지 않습니다.

`TickStepper`는 경계 0부터 모든 입력을 통지한 뒤 정지하지 않은 경우에만 새 iteration을 통지합니다. Pause 중에도 control_step이 증가해 Resume 입력을 전달합니다. 스냅샷 복원과 카운터 한계 검사, 오류 시 무통지/상태 보존을 제공합니다. 코어의 no_std·무할당/I/O·안전한 Rust·의존 0개와 리소스→코어 로컬 의존 하나를 유지합니다.

## 검증

| 검사 | 결과 |
|---|---|
| 합성 TDD | 새 22개 중 20개 Red·초기/빈 입력 2개 및 기존 221개 통과 → Green 코어 62개/리소스 181개 |
| `make check` | Rust 단위 243개·문서 테스트 23개(compile-fail 2개 포함)·Python 8개, 포맷·Clippy·빌드·WASM 컴파일 성공 |
| 시간 분할 | 0~1000ms의 모든 두 프레임 분할 501501건, 정수 기대 틱/위상 일치 |
| 입력 재생 | 5461개 명령 목록×세 프레임 패턴 = 16383회, 278511번 제어 호출의 통지/상태/앱 결과 일치 |
| 독립 모델 | Python 정수 모델·Rust 스트림 서명 5338d6ae252490ba 일치 |
| 문서·추적 규칙 | Markdown 84개·로컬 링크 578개·앵커 100개·이슈 문서명 72개·#23 일지 확인 |
| 의존 경계 | 패키지 2개·코어 의존 0개·리소스→코어 로컬 의존 하나·원본/매니페스트/리소스 코드 보존 |
| 문서 재현 | 세 sh 블록 그대로 실행, Rust/Python 서명·고정 원문 2개 크기/SHA-256/Git blob 확인 |
| 예외/경계 | 키 중복/역행·극단 값·정지/재개·순서 변경·복원·오류 후 복구·불변 데이터·한계에서 무통지 확인 |

고정 Lemmix의 17프레임 시계·갱신 순서와 TS의 60ms 차이를 기록했습니다. 실제 레퍼런스 엔진/DOS를 실행 대조한 결과가 아니며 기존 원본/리소스 코드/입력 매니페스트를 변경하지 않았습니다.

## 게시 상태와 다음 범위

구현 커밋 `8627dd3ceef2e3b80178c9b13b357405528851ba`을 main에 게시했습니다. [push CI 실행](https://github.com/edwardkim/ohmylemmings/actions/runs/39129905687)의 결론과 세 작업이 모두 success입니다.

| 작업 | 실제 실행 근거 |
|---|---|
| [Python tooling](https://github.com/edwardkim/ohmylemmings/actions/runs/39129905687/job/117682762061) | Python 3.12.15, 테스트 8개 성공 |
| [Rust native](https://github.com/edwardkim/ohmylemmings/actions/runs/39129905687/job/117682761756) | Rust 1.93.1, 포맷·Clippy·빌드·단위 243개·문서 테스트 23개·분할/재생 전수·기존 별칭 거부 성공 |
| [WASM libraries](https://github.com/edwardkim/ohmylemmings/actions/runs/39129905687/job/117682761871) | 고정 Rust·WASM 타깃, 두 라이브러리 컴파일 성공 |

push 이벤트·대상 SHA·실제 로그·테스트 개수·분할/재생/기존 compile-fail·clean-input 성공을 대조했습니다. 원본·기존 target·Cargo 패키지 캐시 없는 러너에서 통과했습니다. 원격 tree API의 파일 139개 모드·blob 해시가 로컬과 일치했고 작업 트리는 깨끗합니다. 원본·참조 소스 사본·빌드 출력을 추적하지 않습니다.

G05-01 범위를 완료했습니다. 최종 문서 게시·CI·상위 #1 갱신·#23 종료는 GitHub 이슈 기록에서 추적합니다.

PauseGlitch·시작 시계/입구/생성·개체/직업·전체 trace·실제 DOS 실행은 후속 범위이며 G05 전체·COMP-04~06은 완료로 처리하지 않습니다. 다음은 별도 이슈의 **G05-02 시작 시계·입구·생성 스케줄**입니다.
