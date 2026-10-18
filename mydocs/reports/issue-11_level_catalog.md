# G03-02 ODDTABLE·표시 카탈로그 결과

작업 이슈: [GitHub #11](https://github.com/edwardkim/ohmylemmings/issues/11). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

- [연결·형식·API·오류·의존 방향·재현 명령](../tech_insights/issue-11_level_catalog.md)
- [진행 기록](../working/issue-11_level_catalog.md), [작업일지](../orders/20261018.md#issue-11)

## 변경 내용

`CatalogEntry`가 코어의 검증된 `LevelId`를 120개 DOS 원작 진입점에 연결합니다. `DosLevelFile`은 번호·8개 구간 전체·2048바이트 해제 크기를 검증하고, `OddTable`은 56바이트 레코드 전체를 검증합니다. `ResolvedLevel`은 선택 구간을 해제·검증한 뒤 필요한 40개 변형에서 수치와 제목만 적용하며 기본 배치·원문 바이트를 보존합니다.

공통 `LevelProperties`와 초기 수치 검증 함수를 도입했습니다. 리소스→코어의 로컬 의존 하나로 ID를 재사용하며 코어의 역방향 의존은 없습니다. 외부 패키지·라이브러리 할당·I/O를 추가하지 않았습니다. 호스트 예제 `inspect_catalog`로 원본 파일에서 120개를 직접 연결해 JSONL로 검사할 수 있습니다.

## TDD와 로컬 검증

| 검사 | 결과 |
|---|---|
| 골격과 새 테스트 13개로 `cargo test -p lemmings-resources --lib --locked --offline` | Red: 새 13개 실패·기존 54개 통과 |
| 구현 후 리소스 단위 테스트 | Green: 67개 통과 |
| `make check` | 전체 Rust 단위 69개·문서 예제 6개·Python 8개, 빌드·포맷·Clippy·WASM 성공 |
| 원본·소스 `verify` | 원본 50개·소스 46개 크기·SHA-256 통과 |
| 순서표 대조 | Lemmix·TS 설정·기존 목록의 120개 파일/구간/ODD 연결 일치 |
| Rust 원본 로딩과 독립 Python | 120개 유효 수치·제목 원문·연결이 전부 일치, 기존 카탈로그 기대값도 일치 |
| 변형·배치 보존 | 80개 기본 진입점·40개 ODD 변형 확인, 모든 120개 기본 레코드의 전체 바이트·고정 해시 유지 |
| 예제 오류 | 인자 누락/초과, 사용하지 않는 ODD 레코드 손상 거부 확인 |
| 문서의 재현 코드 그대로 실행 | 120개 연결·유효 수치·제목·기본 해시와 40개 변형 확인 |
| 문서 링크·앵커·파일명 | Markdown 48개, 로컬 링크 291개·앵커 47개, 이슈 문서명 36개 통과 |
| Cargo 메타데이터·공백·미완성 코드 검사 | 로컬 라이브러리 2개, 리소스→코어 의존 하나·외부 패키지 0개, 공백·Rust stub 없음 확인 |

합성 테스트는 두 표현의 전체 순서표, Fun 8의 표시 순번/ODD 인덱스 차이, 부분 ODD와 큰 조회 인덱스, 잘못된 수치·원문 제목, 파일 번호·구간 수·후미 손상·해제 크기, 잘못된 파일·누락 ODD의 출력 보존, 변형 후 화면/그래픽/전체 배치 유지, 해제·기본 파서 오류 문맥을 포함합니다. 기존 기본 레벨 테스트도 공통 검증 함수 변경 후 통과했습니다.

## 원본 대조와 한계

원본 ODDTABLE은 4480바이트·80개 레코드이며 모두 유효합니다. 120개 표시 진입점 중 40개가 ODD를 사용합니다. 독립 Python은 고정 Lemmix 순서표를 읽고 TS 설정과 대조한 뒤, 기존 고정 해시의 기본 레코드와 원본 ODD 바이트를 `struct`로 읽었습니다. Rust의 최종 수치·32바이트 원문 제목·기본 레코드 전체와 일치했습니다. 기존 매니페스트는 변경하지 않았습니다.

표시 순서에서 객체/지형/강철 배치 개수 합계는 666/18131/550이며 변형이 기본 배치를 재사용하므로 80개 기본 레코드 합계와 다릅니다. Fun 8의 ODD 인덱스는 6이고 최종 제목은 `Not as complicated as it looks`입니다. 파일 태그는 호출자 제공 번호이며 출처 인증은 로컬 매니페스트 검증으로 별도 수행합니다.

## 게시와 후속 범위

구현 커밋 `5cd95d7fb7b439162c7b3e7632e8e2c58d51725d`을 `main`에 게시했습니다. [push CI 실행](https://github.com/edwardkim/ohmylemmings/actions/runs/39112114667)의 전체 결론과 세 작업이 모두 success입니다.

| 작업 | 실제 실행 근거 |
|---|---|
| [Rust native](https://github.com/edwardkim/ohmylemmings/actions/runs/39112114667/job/117630270574) | Rust 1.93.1, 포맷·Clippy·빌드, 단위 69개·문서 예제 6개 성공 |
| [Python tooling](https://github.com/edwardkim/ohmylemmings/actions/runs/39112114667/job/117630270677) | Python 3.12.15, 테스트 8개 성공 |
| [WASM libraries](https://github.com/edwardkim/ohmylemmings/actions/runs/39112114667/job/117630270668) | 고정 Rust·WASM 타깃, 로컬 의존을 포함한 두 라이브러리 컴파일 성공 |

대상 SHA·push 이벤트·실제 로그의 개수·clean-input 검사를 대조했습니다. 원본·기존 target·Cargo 패키지 캐시 없는 러너에서 통과했습니다. Git tree API와 로컬 `git ls-tree -r HEAD`의 파일 74개 blob 해시가 모두 일치했으며 원본·해제 결과·JSONL·빌드 출력은 추적하지 않습니다.

G03-02의 연결·변형 범위를 완료했습니다. 상위 #1은 G00·G01·G02만 전체 목표 완료로 체크하고 G03는 그래픽 해석·지형 재구성이 남아 진행 중으로 유지합니다. 최종 문서 게시·CI와 #11 종료는 GitHub 이슈 기록에서 추적합니다. 다음은 별도 이슈로 진행할 G03-03 GROUND 메타데이터·팔레트 해석이며 G04 지형 기반도 독립적으로 준비 상태입니다.
