# G03-01 DOS 기본 레벨 레코드 결과

작업 이슈: [GitHub #10](https://github.com/edwardkim/ohmylemmings/issues/10). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

- [형식·API·지원 정책·차이·재현 명령](../tech_insights/issue-10_level_records.md)
- [진행 기록](../working/issue-10_level_records.md), [작업일지](../orders/20261018.md#issue-10)

## 변경 내용

`LevelRecord::parse`로 2048바이트 기본 레벨의 헤더·제목·객체·지형·강철 영역을 읽습니다. 입력 전체를 빌리고 헤더와 예약 바이트를 먼저 검증한 뒤 배치를 순회합니다. 슬롯 번호와 원본 바이트를 보존하고 수치를 보정하지 않습니다. 라이브러리는 `no_std`·무할당·외부 의존성 0개를 유지하며 코어·플랫폼 I/O와 분리합니다.

호스트 예제 `inspect_levels`는 DAT 해제와 레벨 해석을 연결해 JSONL로 메타데이터·모든 배치를 출력합니다. UTF-8 가정 없이 제목을 hex로 기록하고 손상 입력에 파일·구간 문맥을 붙입니다.

## TDD와 로컬 검증

| 검사 | 결과 |
|---|---|
| 골격과 새 테스트 17개로 `cargo test -p lemmings-resources --lib --locked --offline` | Red: 새 17개 실패·기존 37개 통과 |
| 최소 구현 후 리소스 테스트·문서 예제 | Green: 단위 54개·문서 예제 4개 통과 |
| `make check` | 전체 Rust 단위 56개·문서 예제 5개·Python 8개, 빌드·포맷·Clippy·WASM 성공 |
| 원본·고정 소스 `verify` | 원본 50개·소스 46개의 크기·SHA-256 통과 |
| 기본 레코드와 기존 기대값 | 80개 해제 레코드의 고정 SHA-256, 변형 없는 80개 진입점의 수치·제목 및 120개 참조의 정적 필드 일치 |
| Rust DAT→해제→레벨과 독립 Python 분석 | 80개 레코드의 헤더·제목·모든 객체 454개·지형 11964개·강철 392개의 좌표·플래그·원래 슬롯 전부 일치 |
| 검사 예제의 오류 경로 | 인자 누락과 레벨이 아닌 ADLIB.DAT 거부, 파일·구간 문맥 확인 |
| 기술 문서의 재현 코드 그대로 실행 | 원본 80개 기본 레코드·카탈로그 메타데이터·배치 개수 대조 성공 |
| 문서 링크·앵커·파일명 | Markdown 45개, 로컬 링크 261개·앵커 42개, 이슈 문서명 33개 통과 |
| Cargo 메타데이터·공백·미완성 코드 검사 | 로컬 라이브러리 2개·외부 의존성 0개, `git diff --check`, Rust stub 없음 확인 |

합성 테스트는 모든 절단 길이·과대 입력, 헤더 상하한·필드 오류 위치·직업 순서, 제목의 공간·NUL·비 UTF-8 보존, 비연속/마지막 슬롯·최대 항목 수·반복/종료, 객체 좌표·modifier, 512가지 지형 Y·플래그·미사용 비트, 강철 X 비트 분리·너비/높이·예약 바이트 오류를 검증합니다.

## 원본과 레퍼런스 대조

원본 DAT는 새 Rust 예제에서 직접 해제했습니다. 독립 Python 쪽 입력은 고정 SHA-256으로 확인한 기존 해제 파일을 사용하고, 좌표·플래그는 별도의 비트 문자열 분할과 `struct`로 해석했습니다. 결과에는 DAT 구간 위치·32바이트 원문 제목·원래 슬롯 번호도 포함됩니다. 기존 호환성 매니페스트는 변경하지 않았습니다.

강철 392개 전체가 TypeScript의 해석과 다릅니다. Lemmix의 실제 변환·역변환에 맞춘 비트 배치를 채택하고 Python과 합성 fixture로 검증했습니다. 지형 Y=256도 Lemmix 기준 -260으로 처리합니다. 원본은 이 Y 경계를 포함하지 않으므로 해당 보증은 합성 테스트 근거입니다. 원본에 존재하는 미사용 지형 비트와 객체 슬롯 간격을 보존합니다.

## 게시와 후속 범위

구현 커밋 `133708bda68424ac51a551e6c670ed59d5eca5b7`을 `main`에 게시했습니다. [push CI 실행](https://github.com/edwardkim/ohmylemmings/actions/runs/39109553222)의 전체 결론과 세 작업이 모두 success입니다.

| 작업 | 실제 실행 근거 |
|---|---|
| [Rust native](https://github.com/edwardkim/ohmylemmings/actions/runs/39109553222/job/117622526610) | Rust 1.93.1, 포맷·Clippy·빌드, 단위 56개·문서 예제 5개 성공 |
| [Python tooling](https://github.com/edwardkim/ohmylemmings/actions/runs/39109553222/job/117622526608) | Python 3.12.15, 테스트 8개 성공 |
| [WASM libraries](https://github.com/edwardkim/ohmylemmings/actions/runs/39109553222/job/117622526412) | 고정 Rust·WASM 타깃으로 두 라이브러리 컴파일 성공 |

대상 SHA·push 이벤트와 실제 로그의 테스트 개수·clean-input을 대조했습니다. 원본·기존 target·Cargo 패키지 캐시 없는 러너에서 통과했습니다. Git tree API와 로컬 `git ls-tree -r HEAD`의 파일 68개 blob 해시가 모두 일치했으며 원본·해제 결과·빌드 출력은 추적하지 않습니다.

G03-01의 기본 레코드 읽기를 완료했습니다. 상위 #1은 G00·G01·G02만 전체 목표 완료로 체크하고 G03는 진행 상태를 유지합니다. 최종 문서 게시·CI와 #10 종료 기록은 GitHub 이슈에서 추적합니다. 다음 G03-02에서 ODDTABLE과 120개 표시 카탈로그를 연결하고, 그래픽 리소스·지형 재구성은 후속 G03 작업에서 검증합니다. 실제 DOS 실행 비교는 후속 통합 범위입니다.
