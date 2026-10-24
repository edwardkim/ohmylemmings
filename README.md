# Oh My Lemmings (Rusty Lemmings)

**한국어** | [English](README.en.md)

[![CI](https://github.com/edwardkim/ohmylemmings/actions/workflows/ci.yml/badge.svg)](https://github.com/edwardkim/ohmylemmings/actions/workflows/ci.yml)
[![Play on GitHub Pages](https://github.com/edwardkim/ohmylemmings/actions/workflows/pages.yml/badge.svg)](https://edwardkim.github.io/ohmylemmings/)

고전 DOS 게임 Lemmings의 데이터와 동작을 Rust로 재구현하는 비상업적 토이프로젝트입니다. 결정론적 게임 코어를 렌더링·입출력과 분리하고, SDL2 기반 Native와 WebAssembly 기반 Web에서 공유합니다.

## 공개 목적과 개발 방식

이 프로젝트는 Reverse Engineer Anything(REA)을 사용하지 않고, AI 코딩 에이전트와 함께 고전 게임의 재구현 과정을 탐구하기 위해 공개했습니다. 구현·테스트·플레이 검증·사용자 피드백을 반복하는 개발 과정을 공유하는 것이 목적입니다. REA 미사용은 메인테이너가 밝힌 개발 방식이며, 특정 도구를 사용하지 않았다는 사실이 법적 권한이나 독립 창작을 입증하는 것은 아닙니다.

공개된 **Lemmix** 구현을 물리 규칙·틱·상태 전이의 참고 자료로, **Lemmings.ts**를 데이터 파싱·웹 구조의 참고 자료로 사용했습니다. 원본 게임의 데이터·동작과 비교했으며, OPL 합성기는 기존 오픈소스 구현을 활용·수정했습니다. 참고 소스의 버전과 출처는 [레퍼런스 지도](mydocs/tech_insights/issue-3_reference_code_map.md)와 [고정 소스 목록](assets/manifests/reference_sources.json)에 기록합니다.

따라서 **“역공학 없이 구현했다”, “원본이나 타 구현을 전혀 참고하지 않았다”, “완전한 클린룸 구현이다”라는 주장을 하지 않습니다.** 원본 관찰·데이터 해석과 공개 참조 구현을 활용한 재구현입니다.

## 라이선스와 원본 자산의 경계

- 자체 작성 기여에는 [MIT](LICENSE)가 적용됩니다. 외부 코드에는 각각의 라이선스가 적용되며 [제3자 고지 전문](THIRD_PARTY_NOTICES.txt)과 [적용 범위](third_party/README.md)를 확인하세요.
- OPL 합성기는 **LGPL-2.1-or-later**, Native MP3 디코더 Symphonia는 **MPL-2.0**, Noto Sans 글꼴은 **OFL**을 유지합니다. 소스 제공·수정·재링크 안내는 [OPL 문서](crates/lemmings-opl/README.md)와 배포본의 `REBUILD.md`를 따릅니다.
- **원본 DOS 그래픽·레벨·음악·효과음·캐릭터·상표에는 프로젝트 MIT 허가가 적용되지 않습니다.** HD로 다시 그리거나 AI로 생성했다는 사실만으로 원작 관련 권리가 해소되는 것은 아닙니다.
- **현재 v0.3.2 다운로드와 Pages에는 원본 게임 데이터가 포함되어 있습니다.** 확인한 프로젝트 자료에는 그 원본 자산의 공개 재배포 허락 근거가 없습니다. 출처·해시와 라이선스 고지 검사는 배포 권한의 확인을 의미하지 않습니다.
- 연구·학습·비상업적 목적이라는 설명도 원본 자산의 이용·재배포 허가를 대신하지 않습니다. 이 프로젝트는 공식 제품이나 권리자의 승인·제휴를 주장하지 않습니다.
- 사용자 로컬 MP3는 업로드하거나 배포본에 포함하지 않습니다. 이 기능은 음악의 저작권 자체를 없애거나 모든 이용을 허용하지 않습니다.

이 안내는 개발 취지와 현재 확인 범위를 설명합니다. 배포 권한 확보나 원본 자산 제거가 완료됐다는 공지가 아닙니다.

## 플레이와 현재 상태

2026-10-25 기준 최신 정식 버전은 **v0.3.2**입니다. 위의 원본 자산 관련 경계를 함께 확인하세요.

- [웹에서 플레이](https://edwardkim.github.io/ohmylemmings/) · [Windows/macOS/Linux/Web 다운로드](https://github.com/edwardkim/ohmylemmings/releases/tag/v0.3.2)
- [v0.3.2 실행 안내](mydocs/releases/issue-219_v0.3.2.md) · [배포·공개 검증](mydocs/reports/issue-219_release_v032.md)
- 원본 120레벨, 8가지 기술, HD/원본 화면 전환, 지형 질감·수직벽 개선을 제공합니다.
- 굴착 충격·분진·효과음, 원형 대상 표시, 추락사 충격 표현과 수량0 기술 선택 차단을 제공합니다.
- Web은 현재 플레이에 필요한 HD 자산을 먼저 받고 다음 레벨 자산을 백그라운드로 받습니다. 진행률·재시도·캐시 재사용을 지원합니다.
- F3으로 FPS를 표시하고 F2로 피드백 요약을 복사합니다. [#113 성능·메모리 검증](https://github.com/edwardkim/ohmylemmings/issues/113)은 남아 있으며 60fps를 보장하지 않습니다.

Windows는 **Windows 11 x64**를 지원하며 Windows 10은 지원하지 않습니다. macOS는 **13+ Apple Silicon/Intel universal2**이며 ad-hoc 서명·미공증 상태입니다. Linux는 SDL2 runtime이 필요합니다. 압축 파일은 전체를 풀어서 사용하세요.

Web에서는 **MP3 선택**, Native에서는 **F4/Ctrl+O**(macOS **Cmd+O**도 지원), 파일 드롭 또는 `--bgm FILE`로 로컬 음악을 선택합니다. 음악·효과음 음량을 따로 조절할 수 있습니다. Web의 소리는 화면의 **소리 켜기**로 활성화합니다.

## 개발 및 실행

[rust-toolchain.toml](rust-toolchain.toml)이 Rust **1.93.1**과 rustfmt·Clippy를 고정합니다. GNU Make, Python 3.12, SDL2 개발 라이브러리와 WASM 타깃이 필요하며 웹 검사는 Chrome/Chromium을 사용합니다. 플랫폼별 준비는 [Native 안내](apps/desktop/README.md)와 [Web 안내](apps/web/README.md)를 참고하세요.

```sh
rustup target add wasm32-unknown-unknown
cargo fetch --locked
python3 tools/web_build.py fetch
make check
```

`make check-rust`, `make check-python`, `make check-wasm`, `make check-desktop`, `make check-web`으로 나누어 실행할 수 있습니다. [CI 안내](mydocs/plans/issue-7_continuous_integration.md)와 [Makefile](Makefile)에 세부 명령이 있습니다.

원본을 이용할 권한이 있는 경우의 데이터 준비·식별 절차는 [자산 안내](assets/README.md)에 있습니다. 도구가 파일을 취득할 수 있다는 사실은 이용·재배포 허가가 아닙니다. 로컬에 준비한 데이터로 실행하려면 다음을 사용합니다.

```sh
cargo run --release --locked --offline -p lemmings-desktop -- --assets assets/local/dos-original
```

F/C/E/K/U/M/B/D는 Floater/Climber/Bomber/Blocker/Builder/Miner/Basher/Digger입니다. 기술을 고르고 레밍을 클릭합니다. Space는 일시정지, 위/아래는 생성 속도, 좌/우는 카메라, Esc는 종료입니다. 상세 키와 오디오·HD 옵션은 플랫폼별 안내를 따릅니다.

## 구조와 기여

| 경로 | 역할 |
|---|---|
| [lemmings-core](crates/lemmings-core/src/lib.rs) | `no_std` 자료형·지형·결정론적 시뮬레이션 |
| [lemmings-resources](crates/lemmings-resources/src/lib.rs) | `no_std` DOS 자원 파싱·이미지·레벨 조립 |
| [lemmings-session](crates/lemmings-session/README.md) | 공유 게임 세션·시간·입력 처리 |
| [lemmings-game](crates/lemmings-game/README.md) | 자산·장면·패널·HD 표시·오디오 연결 |
| [apps/desktop](apps/desktop/README.md) / [apps/web](apps/web/README.md) | SDL2 및 JS/WASM 플랫폼 어댑터 |
| `tools/` / `tools/tests/` | 취득·검증·패키징 도구와 오프라인 테스트 |

[기여 지침](AGENTS.md)과 [문서 안내](mydocs/README.md)를 먼저 읽어주세요. Hyper-Waterfall 절차에 따라 이슈를 등록하고 관련 문서를 읽은 뒤 TDD·검증·self-review·PR·결과 기록을 진행합니다. `mydocs/` 작업 문서는 `issue-<번호>_<주제>.md`, 날짜별 일지는 `mydocs/orders/yyyymmdd.md`로 관리합니다. 다운로드한 바이너리와 개인 검증 자료는 ignored `assets/local/`, 출처·해시는 `assets/manifests/`에 둡니다.

[프로젝트 기획](mydocs/plans/issue-1_rusty_lemmings.md) · [구현 계획](mydocs/plans/issue-1_implementation_plan.md) · [기술 분석](mydocs/tech_insights/README.md) · [검증 보고서](mydocs/reports/README.md) · [버전·태그 정책](mydocs/plans/issue-124_release_policy.md) · [릴리즈 기록](mydocs/releases/README.md)

[이전 README의 상세 개발 이력](https://github.com/edwardkim/ohmylemmings/blob/a37aecd/README.md)은 당시 기록으로 보존합니다. 거기에 있는 “예정/후속” 문구는 현재 미구현 상태를 뜻하지 않습니다.
