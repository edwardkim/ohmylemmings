# G01-01 Rust workspace와 모듈 경계

작업 이슈: [#6](https://github.com/edwardkim/ohmylemmings/issues/6), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 구성과 의존 방향

[전체 구현 계획](issue-1_implementation_plan.md)의 G01-01 산출물입니다. [G00 호환성 계약](../tech_insights/issue-5_compatibility_baseline.md)을 기준으로 도메인 로직, 바이너리 형식, 플랫폼 입출력을 나눕니다.

| 위치 | 책임 | 현재 상태 |
|---|---|---|
| [crates/lemmings-core](../../crates/lemmings-core/Cargo.toml) | 검증된 도메인 자료형, 이후 지형·틱·개체·게임 규칙 | `LevelId`, `Difficulty`, 번호 오류 구현 |
| [crates/lemmings-resources](../../crates/lemmings-resources/Cargo.toml) | 제공받은 바이트의 읽기·형식 해석·오류 위치 | `ByteReader`, `ReadError` 구현 |
| `apps/desktop/` (G06 예정) | 파일 읽기, 데이터 조립, SDL2 화면·입력·오디오 | 실행 어댑터 도입 시 생성 |
| `apps/web/` (G09 예정) | 브라우저 리소스 공급, WASM 경계, Canvas·입력·오디오 | 실행 어댑터 도입 시 생성 |

현재 두 라이브러리에는 서로를 포함한 패키지 의존성이 없습니다. 플랫폼 조립부가 리소스 표현을 코어의 도메인 자료형으로 변환하고 코어에 데이터를 전달합니다. 코어는 파서·파일·SDL2·브라우저 API를 호출하지 않습니다. 리소스 라이브러리도 파일을 열지 않고 호출자가 준 바이트만 읽습니다. 새 공통 자료형이 필요하면 사용 지점과 변환 책임을 해당 구현 이슈에서 정합니다.

두 라이브러리는 항상 `#![no_std]`이며 현재 힙 할당을 사용하지 않습니다. 테스트 실행기는 호스트에서 동작하고 라이브러리 자체는 네이티브·WASM 모두 컴파일됩니다. `no_std` 선택의 의미는 [Rust 공식 설명](https://doc.rust-lang.org/stable/embedded-book/intro/no-std.html)을 참고합니다.

## Cargo와 도구 체인

- 루트는 실행 패키지가 없는 [virtual workspace](https://doc.rust-lang.org/cargo/reference/workspaces.html#virtual-workspace)입니다. 멤버 2개와 `resolver = "3"`을 명시합니다.
- [rust-toolchain.toml](../../rust-toolchain.toml)은 검증한 Rust 1.93.1, 최소 프로필, rustfmt·Clippy를 고정합니다. Edition은 2024입니다.
- [Cargo.lock](../../Cargo.lock)을 커밋하고 두 패키지는 `publish = false`로 관리합니다. 레지스트리 패키지는 없습니다.
- workspace의 `unsafe_code = "forbid"`를 두 멤버가 상속합니다. 빌드 산출물 `target/`은 기존 ignore 규칙을 사용합니다.

## 최소 API 계약

`LevelId::new(difficulty, number)`는 표시 번호 `1..=30`만 받습니다. 필드는 비공개로 유지해 범위 밖 식별자를 만들 수 없게 합니다. `index()`는 그룹 안의 0부터 시작하는 번호, `global_index()`는 Fun→Tricky→Taxing→Mayhem 순서의 0..119 번호입니다. 이 번호는 DAT 구간이나 ODDTABLE 인덱스와 다릅니다.

`ByteReader`는 입력 슬라이스를 빌립니다. `read_bytes(count)`와 `read_u16_be()`는 성공했을 때만 위치를 전진시킵니다. 절단 입력은 `UnexpectedEof { offset, requested, remaining }`으로 반환합니다. 빈 읽기는 끝에서도 성공하고 `usize::MAX` 요청은 오버플로 없이 오류로 반환합니다. 바이트 순서는 현재 원작 레벨 메타데이터에 맞춘 빅 엔디언 API에 명시합니다.

DAT 압축·레벨·그래픽 파서와 시뮬레이션은 후속 목표에서 구현합니다. 현재 API는 작은 범위 검증과 입력 처리 기반입니다.

## 개발과 검증

Rustup과 고정 도구 체인을 설치한 뒤 저장소 루트에서 실행합니다. 현재 Cargo 패키지 빌드·테스트에는 네트워크와 원본 게임 파일이 필요하지 않습니다.

```sh
cargo build --workspace --locked --offline
cargo test --workspace --locked --offline
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
```

개별 라이브러리와 WASM 컴파일 확인:

```sh
cargo build -p lemmings-core --lib --locked --offline
cargo build -p lemmings-resources --lib --locked --offline
rustup target add wasm32-unknown-unknown
cargo build --workspace --lib --target wasm32-unknown-unknown --locked --offline
```

WASM 표준 라이브러리 설치에는 최초 네트워크 접근이 필요할 수 있습니다. 이 명령은 라이브러리 컴파일 검증이며 브라우저 실행은 G09에서 도입합니다. SDL2 설치와 게임 실행 명령은 G06 어댑터 도입 시 함께 문서화합니다.

## 완료 기준과 후속 작업

행동 테스트를 먼저 작성해 실패를 확인하고 구현 후 단위 테스트 5개·문서 예제 2개를 통과했습니다. 레벨 검증은 각 그룹의 모든 u8 입력을 검사하고 120개 정상 식별자의 표시/내부 인덱스를 대조합니다. 바이트 읽기는 정상·끝·절단·빈·과대 요청과 실패 후 재시도를 검증합니다.

[G01-02 공통 검증과 CI](issue-7_continuous_integration.md)에서 이 명령을 연결하고 깨끗한 체크아웃·실제 Actions 성공까지 확인했습니다. 현재 G01 전체 완료 조건을 충족했습니다.

- [작업일지](../orders/20261018.md#issue-6)
- [진행 기록](../working/issue-6_rust_workspace.md)
- [검증 보고서](../reports/issue-6_rust_workspace.md)
