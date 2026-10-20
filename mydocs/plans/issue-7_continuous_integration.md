# G01-02 로컬·CI 공통 개발 검증

작업 이슈: [#7](https://github.com/edwardkim/ohmylemmings/issues/7), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 검증 진입점

[Makefile](../../Makefile)을 로컬과 GitHub Actions가 함께 사용합니다. Rustup·고정 Rust 도구 체인, GNU Make, Python 3.12, SDL2 개발 라이브러리, C99 컴파일러(`cc`)와 WASM 표준 라이브러리를 준비합니다.

```sh
rustup show active-toolchain
rustup target add wasm32-unknown-unknown
cargo fetch --locked
python3 tools/web_build.py fetch
make check
```

Rustup의 최초 도구 체인·타깃 설치에는 네트워크가 필요할 수 있습니다. 데스크톱의 고정 레지스트리 의존을 최초 fetch한 뒤 Cargo 검사 명령은 `--locked --offline`을 사용하고 Python 테스트는 표준 라이브러리와 합성 입력만 사용합니다. 원본 파일을 확보하는 `fetch`는 검증에 포함하지 않습니다.

| 명령 | 검사 |
|---|---|
| `make check` | 아래 검사 전체 |
| `make check-rust` | rustfmt, Clippy 경고 금지, workspace 빌드, 단위·예제·문서 테스트, 기록된 참조 상태와 재생 대조 |
| `make check-python` | `tools/tests/`의 unittest와 개체 이동 독립 모델/golden 검사 |
| `make check-desktop` | 빌드된 앱의 실제 SDL dummy/software 이벤트·재생·픽셀 readback 검사 |
| `make check-wasm` | `wasm32-unknown-unknown`용 core/resources/session/game/web 라이브러리 빌드 |
| `make check-web` | 고정 CLI로 실제 WASM/JS 생성, headless Chrome/Chromium의 합성 상태·맵·RGB/Canvas 대조 |

개별 Cargo 명령과 API 경계는 [G01-01 설계](issue-6_rust_workspace.md)를 참고합니다. WASM libraries 작업은 라이브러리 컴파일이고, #66에서 추가한 Web browser 작업은 실제 WASM/브라우저 실행입니다. Linux x86_64 CLI·Chrome/Chromium 준비와 실행 안내는 [웹 안내](../../apps/web/README.md)를 따릅니다.

[#25 이동 검증](../tech_insights/issue-25_actor_motion.md)의 독립 Python 모델은 원본 없이 전체 합성 golden을 다시 계산합니다. Rust 테스트는 저장된 기대값과 구현을, Python 검사는 그 기대값과 독립 모델을 대조합니다.

[#26 상태 재생](../tech_insights/issue-26_state_replay.md)은 make check-rust에서 Python 표준 라이브러리로 Rust 예제를 실행하고, 실제 Pascal 함수 하네스에서 기록한 합성 상태 해시와 세 렌더 주기를 대조합니다. CI에 원문·FPC·원본은 필요하지 않습니다.

## GitHub Actions

[ci.yml](../../.github/workflows/ci.yml)은 push·pull_request·workflow_dispatch에서 실행합니다. 경로 필터 없이 문서 변경에도 같은 검사 이름을 유지합니다. 실행 구성은 [GitHub workflow 문법](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax)을 따릅니다.

| 작업 이름 | 실행 명령 | 환경 |
|---|---|---|
| Rust native | `make check-rust check-desktop` | Ubuntu 24.04, rust-toolchain.toml의 Rust 1.93.1·rustfmt·Clippy·Python 3 |
| Python tooling | `make check-python` | Ubuntu 24.04, Python 3.12 계열 |
| WASM libraries | `make check-wasm` | Ubuntu 24.04, Rust 1.93.1·WASM 타깃 |
| Web browser | `make check-web` | Ubuntu 24.04, Rust1.93.1·고정 wasm-bindgen CLI·기존 Chrome |

네 작업 모두 체크아웃 직후 `assets/local/`과 `target/`이 없는지 확인합니다. Rust/WASM/Web 작업은 `$RUNNER_TEMP/cargo-home`도 없는지 검사한 뒤 `GITHUB_ENV`로 CARGO_HOME을 지정해 패키지 캐시를 공유하지 않습니다. 도구 체인은 체크아웃의 설정 파일을 읽는 `rustup show active-toolchain`으로 설치·선택합니다. Ubuntu의 Rustup 제공은 [runner 이미지 목록](https://github.com/actions/runner-images/blob/main/images/ubuntu/Ubuntu2404-Readme.md)에 명시되어 있습니다.

권한은 `contents: read`, checkout의 `persist-credentials`는 false입니다. [#54](https://github.com/edwardkim/ohmylemmings/issues/54)의 결과 전환 회귀를 포함해 Rust native 제한 시간은 15분, Python/WASM/Web은 10분이며 같은 브랜치/PR의 새 실행이 이전 실행을 취소합니다. 현재 dependency·빌드 캐시는 사용하지 않습니다.

## 고정한 외부 Action

공식 릴리스 태그를 GitHub API로 조회하고 태그가 가리키는 커밋을 확인한 뒤 40자리 SHA로 고정했습니다. 갱신은 별도 이슈에서 릴리스와 실제 CI를 다시 검증합니다.

| Action | 확인한 릴리스 | 고정 커밋 |
|---|---|---|
| [actions/checkout](https://github.com/actions/checkout/releases/tag/v7.0.1) | v7.0.1 | `3d3c42e5aac5ba805825da76410c181273ba90b1` |
| [actions/setup-python](https://github.com/actions/setup-python/releases/tag/v7.0.0) | v7.0.0 | `5fda3b95a4ea91299a34e894583c3862153e4b97` |

## 완료 판정

로컬 정적 검사와 `make check`, 원본·target·Cargo 캐시가 없는 Git tree 내보내기의 `make check`, 게시 커밋의 실제 GitHub Actions 네 작업이 모두 성공해야 완료입니다. 실행 URL과 대상 커밋을 [보고서](../reports/issue-7_continuous_integration.md)에 기록합니다. PR·수동 트리거의 선언과 실제 실행한 이벤트는 구분합니다.

이 조건을 충족하면 G01을 완료하고 G02 DAT 처리와 G04 지형 기반을 각각 후속 이슈로 진행합니다.

- [작업일지](../orders/20261018.md#issue-7)
- [진행 기록](../working/issue-7_continuous_integration.md)

## G06-01 CI 갱신 / #27

[#27 화면 검증](../reports/issue-27_native_view.md)을 추가했습니다. Rust native는 초기 빈 Cargo cache 확인 후 libsdl2-dev와 cargo fetch --locked를 준비하고 오프라인 workspace·SDL 합성 검사를 수행합니다. WASM도 잠금 의존 메타데이터를 fetch하고 두 no_std 라이브러리만 빌드합니다. Python 작업은 그대로 원본/SDL 없이 실행합니다. 실제 원본 X11/키·화면 검증은 로컬 실행 증거이며 CI는 원본 없는 SDL dummy/software를 확인합니다.

## G09-03 실제 웹 검사 / #66

[#66](../reports/issue-66_wasm_browser.md)에서 Web browser 작업을 추가했습니다. 원본/target/Cargo cache가 없는 체크아웃에서 고정 crate와 해시 검증한 공식 CLI만 준비하고 실제 WASM·JS를 빌드합니다.13개 합성 사례·렌더1/7의 전체 상태/지형/브릭/객체 맵·논리 RGB·Canvas readback과 시간/오류/재시작 경계를 검증합니다. Node/NPM 패키지는 사용하지 않습니다.

## G10-03 PCM 독립 대조 / #76

[#76](../reports/issue-76_opl_pcm.md)에서 make check-rust에 고정 LGPL C1.8 소스의 크기/SHA 검증·별도 host 빌드/합성 PCM 전수 비교와 native PCM fixture/청크 검사를 추가했습니다. `cc`가 필요하며 Ubuntu runner의 기본 C 툴체인을 사용합니다. make check-web은 별도 PCM 어댑터의 실제 WASM을 실행해 두 분할의 i16LE SHA를 대조합니다. 새 원본/네트워크·npm/Unicorn 의존성 없이 기존 네 CI 작업에서 실행합니다.

## G10-04 게임 명령·PCM / #78

[#78](../reports/issue-78_game_sound.md)에서 `make check-rust`에 합성98경로·고정 원본 x86 메타데이터97분기/170음악조합과 typed 명령→PCM 검사를, `make check-web`에 같은 경로의 실제 WASM/PCM 검사를 추가했습니다. 기존 Web 검사도 WebGame의 사운드 API/큐/세대를 직접 실행합니다. 새로운 네트워크·외부 패키지·원본/Unicorn·장치 요구 사항은 없습니다.

## G10-05 장치·라이선스 배포 / #80

[#80](../reports/issue-80_audio_output.md)에서 check-desktop에 생산 SDL service의 dummy 큐 소비·음소거/숨김/세대·없는 장치/게임 불변 검사와 fixed6조합249492프레임을 추가했습니다. check-web은 실제 WASM/OfflineAudioContext·실시간 AudioContext graph와 bounded scheduling/활성화 오류·소스/라이선스/WASM 해시 배포를 검사합니다. headless graph만 autoplay 테스트 플래그를 사용하고 trusted gesture/LAN은 별도 실제 로컬 증거입니다. Python은 source archive의 private/symlink 배제·재현성·Git 없는 수정/재빌드를 검사합니다. 새 외부 패키지/원본/물리 장치 의존성은 없습니다.

## G10-06 통합 사운드 / #82

`make check-web`은 원본 없이580경계/1,824,750PCM의 native/실제 WASM 게임·사건·오디오·18맵/RGB와 청크 분할을 비교합니다. Python은 WAV/좌채널 보존과 DRO 손상/주파수/시간 검사를 포함합니다. 원본/DOSBox·PipeWire 녹음/주관적 청취는 선택적 로컬 증거입니다. [재현·범위](../tech_insights/issue-82_audio_playthrough.md).
