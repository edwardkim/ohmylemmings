# G02-01 DAT 구간 읽기 결과

작업 이슈: [GitHub #8](https://github.com/edwardkim/ohmylemmings/issues/8). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

- [형식·API·오류·재현 명령](../tech_insights/issue-8_dat_sections.md)
- [진행 기록](../working/issue-8_dat_sections.md), [작업일지](../orders/20261018.md#issue-8)

## 변경 내용

`lemmings-resources`에 `DatSections`, `DatSection`, `DatHeader`, `DatError`를 추가했습니다.

10바이트 헤더와 비어 있지 않은 압축 본문을 빌려 순회하고, 크기·예약 워드·0~8 비트 수·출력 크기·XOR를 확인합니다. 잘린 입력과 뒤쪽 데이터도 명시적인 오류로 보고하며 첫 오류 후 종료합니다. `no_std`, 할당·외부 의존성 없는 라이브러리를 유지하고 파일 I/O는 호스트 예제 `inspect_dat`에 둡니다.

## TDD와 로컬 검증

| 검사 | 결과 |
|---|---|
| API 골격 + 새 테스트 14개로 `cargo test -p lemmings-resources --lib --locked --offline` | Red: 13개 실패, 기존 3개·빈 iterator 1개 통과 |
| 최소 구현 후 같은 테스트와 문서 예제 | Green: 리소스 단위 17개·문서 예제 2개 통과 |
| `make check` | Rust 단위 19개·문서 예제 3개, Python 8개, 빌드·포맷·Clippy·WASM 통과 |
| 원본·소스 매니페스트 `verify` | 50개 원본·46개 소스의 크기·SHA-256 검증 통과 |
| Rust `inspect_dat` 출력과 독립 Python `struct.unpack_from('>BB4H')`/XOR 분석 대조 | 원본 21개 파일·102개 구간의 메타데이터 전부 일치 |
| 호스트 예제의 인자 누락·정상 구간 뒤 손상 헤더 | 실패 종료·파일/구간 위치 오류 확인 |
| 문서 링크·앵커·파일명 검사 | Markdown 39개, 로컬 링크 202개·앵커 32개, 이슈 문서명 27개 통과 |
| Cargo 메타데이터·공백 검사 | 로컬 라이브러리 2개·외부 의존성 0개, `git diff --check` 통과 |

새 테스트는 다중 구간·빌린 주소·big-endian 해석, 빈 입력·반복 종료, 모든 부분 헤더/본문, 크기 0~10과 최대 u16, 출력 0, 모든 u8 비트 값, 예약 워드, XOR 오류·0 체크섬, 뒤쪽 1~10바이트, 후속 구간의 파일 위치·오류 뒤 종료를 검증합니다. 합성 fixture는 압축 명령 스트림이 아닌 구간 구조용 데이터입니다.

원본 대조는 입력 크기·SHA-256을 먼저 확인한 뒤 Rust가 출력한 각 행의 파일·구간 번호·위치·비트·체크섬·해제 크기·구간 크기·본문 크기를 Python 결과와 비교했습니다. CI에는 원본을 포함하지 않습니다.

## 게시 검증

문서 검사·원격 파일 대조·실제 GitHub Actions는 게시 단계에서 결과를 추가합니다. 이슈 #8은 해당 확인이 끝난 뒤 완료 처리합니다.

## 다음 작업과 한계

G02-01의 결과는 구간 읽기입니다. 압축 명령·해제 바이트의 정확성은 아직 검증하지 않았으며 G02 전체는 진행 상태입니다. 다음 G02-02에서 역방향 비트 읽기·명령 해제·잘린 명령/잘못된 복사 오류를 TDD로 구현하고 기존 81개 해제 구간의 크기·SHA-256 기대값을 대조합니다. G04-01 지형 기반은 별도 경로로 준비 상태입니다.
