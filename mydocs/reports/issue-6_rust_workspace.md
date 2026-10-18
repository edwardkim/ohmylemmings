# G01-01 Rust 개발 골격 결과

작업 이슈: [#6](https://github.com/edwardkim/ohmylemmings/issues/6), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

- [설계·API·개발 명령](../plans/issue-6_rust_workspace.md)
- [작업일지](../orders/20261018.md#issue-6), [진행 기록](../working/issue-6_rust_workspace.md)

## 변경 결과

Rust 1.93.1·Edition 2024의 Cargo workspace를 도입했습니다. `lemmings-core`는 120개 레벨의 검증된 식별자, `lemmings-resources`는 빌린 바이트 입력·빅 엔디언 읽기·오류 위치를 제공합니다. 두 라이브러리는 `no_std`이며 패키지 의존성이 없고 현재 힙 할당을 사용하지 않습니다.

플랫폼은 두 라이브러리에 데이터를 공급·조립하고 입출력을 담당하도록 경계를 정의했습니다. 원작 파서·게임 로직·실행 어댑터는 각각 후속 목표에서 구현합니다.

## 테스트와 검증

| 명령·검사 | 결과 |
|---|---|
| TDD Red: API 골격과 테스트 작성 후 `cargo test --workspace --locked --offline --no-fail-fast` | 미구현 동작으로 단위 테스트 5개 실패, 종료 코드 101 |
| TDD Green: 최소 구현 후 `cargo test --workspace --locked --offline` | 단위 테스트 5개와 문서 예제 2개 통과 |
| `cargo build --workspace --locked --offline` | 네이티브 라이브러리 2개 빌드 성공 |
| `cargo fmt --all -- --check` | 통과 |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | 경고 없이 통과 |
| `cargo build -p lemmings-core --lib --locked --offline` 및 resources 개별 빌드 | 각각 성공 |
| `cargo build --workspace --lib --target wasm32-unknown-unknown --locked --offline` | 라이브러리 2개 WASM 컴파일 성공 |
| `cargo metadata --locked --offline --format-version 1` 검사 | 라이브러리 멤버 2개, 의존성 0개, 실행 바이너리 없음, 도구 체인·Edition 일치 |
| 문서 링크·앵커·파일명 검사 | Markdown 33개, 로컬 링크 151개, 앵커 24개, 이슈 문서명 21개 통과 |
| `git diff --check` | 통과 |

번호 검증은 4개 그룹의 모든 u8 입력과 정상 120개 인덱스를 대조합니다. 바이트 입력은 절단·빈 입력·`usize::MAX` 요청에 대해 오류 문맥·위치 보존·후속 읽기를 검증합니다. 모든 테스트가 합성 입력을 사용합니다.

원격 게시와 상위 이슈 상태는 확인 후 추가 기록합니다.

## 현재 한계와 다음 작업

이번 결과는 라이브러리 기반이며 게임 실행 파일·SDL2·웹 호스트가 없습니다. 원본 호환성 실행 판정은 G00의 책임 목표에 따라 진행합니다. 다음 G01-02에서 CI와 지속적인 개발 검증을 연결하며 G01 전체는 진행 상태로 유지합니다.
