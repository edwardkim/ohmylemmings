# G06-01 네이티브 화면·입력 계약

작업 이슈: [GitHub #27](https://github.com/edwardkim/ohmylemmings/issues/27). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

- 선행: [#19 객체 합성](issue-19_level_objects.md), [#16 스프라이트](issue-16_lemming_sprites.md), [#23 고정 틱](issue-23_fixed_ticks.md), [#26 상태 재생](issue-26_state_replay.md).
- [앱 실행 안내](../../apps/desktop/README.md), [진행 기록](../working/issue-27_native_view.md), [보고서](../reports/issue-27_native_view.md), [작업일지](../orders/20261019.md#issue-27).

## 구조와 의존 경계

`apps/desktop`은 workspace의 std 바이너리입니다. 파일·할당·SDL2·벽시계·키보드를 소유하며 코어와 리소스 API를 조립합니다. 두 라이브러리는 no_std와 기존 의존 방향을 유지합니다. 코어 의존은 0개, 리소스는 코어 로컬 의존 하나이며 SDL2를 참조하지 않습니다. 앱도 workspace의 unsafe_code=forbid를 상속합니다.

| 모듈 | 책임 |
|---|---|
| assets.rs | Fun01 카탈로그·LEVEL009/GROUND0O/VGAGR0/MAIN 해석·충돌 마스크·표시 장면 |
| session.rs | StartupScheduler·ID 순 Actor·정렬된 입력 기록·g05-basic-v1 스냅샷 |
| render.rs | 320×200 프레임버퍼·카메라·발 기준 스프라이트·간단한 상태/조작 표시 |
| input.rs | SDL 이벤트 변환·RR 제한·재생 파일 검증 |
| main.rs | CLI·SDL 창/텍스처·경과 시간·캐치업·캡처/종료 |

SDL2 Rust 바인딩은 **sdl2=0.38.0**으로 고정했습니다. [공식 프로젝트](https://github.com/Rust-SDL2/rust-sdl2)와 [고정 버전 Canvas API](https://docs.rs/sdl2/0.38.0/sdl2/render/struct.Canvas.html)를 확인했습니다. Cargo.lock에 총 7개 레지스트리 패키지의 버전/체크섬을 기록합니다. 번들 SDL 빌드를 사용하지 않으며 시스템 SDL2 SDK가 필요합니다. 초기 `cargo fetch --locked` 후 검사는 `--locked --offline`입니다.

## 원본·표시·시간

지원 진입점은 Fun01 하나입니다. 카탈로그에서 파일/구간을 선택하고 헤더의 시간·총수·RR·screen_x와 ID 1 입구를 읽습니다. 입구 ID는 고정 Dos.Consts.pas의 DOS_OBJECT_ID_ENTRANCE=1입니다. 원본 내용의 인증은 reference_assets.py verify, 앱은 파서의 구조·체크섬·손상 오류를 사용합니다.

지형의 불투명 비트로 충돌 마스크를 만들고 별도의 객체/지형 팔레트로 화면을 합성합니다. 객체는 표시용입니다. 고정 Game.pas IncrementIteration/UpdateInteractiveObjects의 입구 35→frame2→마지막 이후 0과 연속 객체 프레임을 iteration에서 계산합니다. 트리거·출구·구출은 실행하지 않습니다.

레밍은 ID 순으로 생성 틱부터 갱신합니다. 방향/행동/프레임을 MAIN 스프라이트에 연결하고 발 기준 (8,10), 투명 인덱스·화면 경계를 보존합니다. 제거 기록은 상태에 남고 그리기에서는 제외됩니다. 뷰의 캐시는 논리 세션을 변경하지 않습니다.

320×160 플레이 영역 아래 40픽셀에 남은 시간·활성 개체·RR·조작을 표시합니다. SDL RGB24 streaming texture 한 개를 재사용하며 기본 960×600 창에 정수 배율·nearest 보간으로 표시합니다. 창 크기 변경은 letterbox를 사용합니다. 원본 HUD/전체 게임 화면의 픽셀 인증은 아닙니다.

Instant의 경과 시간을 FixedTickClock에 전달해 17Hz의 모든 예정 제어 호출을 처리합니다. 캡처 경계에서 배치를 나누어도 남은 호출을 보존합니다. 렌더링은 별도 주기이며 `--render-every`로 빈도를 바꿀 수 있습니다. Release X11 실행에서는 340호출/20.180초·870렌더, 적은 렌더의 재생에서는 57렌더였고 매 호출의 상태는 같습니다. 이 값은 한 실행의 관측이며 성능 보장이 아닙니다.

## 입력·기록

키/조작은 앱 안내를 따릅니다. Space 반복 이벤트는 무시하고 같은 경계의 빠른 토글/속도 변경은 큐의 마지막 의도를 사용합니다. RR은 초기값~99입니다. 카메라는 표시 상태이며 입력 재생의 게임 상태에 포함하지 않습니다. Esc/창 닫기는 다음 갱신 전에 종료합니다.

입력은 제어 경계/sequence로 기록합니다. 재생 파일은 중복·역순·범위 밖·미지원 명령·잘못된 인자를 거부하며 sequence 간격을 허용합니다. `--replay` 중에는 수동 게임 입력을 끄고 카메라/종료를 유지합니다. `--demo-input`은 fast 검사 전용으로 SDL 큐에 Pause/Resume·RR·카메라·Escape를 주입합니다.

캡처는 기존 디렉터리를 거부합니다. 원본 파생 결과는 ignored assets/local 아래로 제한합니다. 매 제어 호출의 JSONL은 #26의 지원 부분집합이며 부동소수점/시각을 저장하지 않습니다. 초기 상태·입력 재생의 전체 snapshot을 기존 G05 어댑터 및 기록된 Lemmix 실행 SHA-256과 대조합니다. 전체 DOS 게임 상태 인증과 구분합니다.

## 검사와 CI

```sh
cargo fetch --locked
make check
```

SDL2 SDK와 WASM 타깃을 먼저 준비합니다. Makefile은 기존 검사에 실제 SDL 합성 창·이벤트 루프·기록 재생·readback을 추가합니다. `tools/check_desktop.py`는 원본 없는 합성 자원으로 네 번 실행해 렌더 빈도·저장 입력 재생과 G05 상태를 비교합니다. RGB24의 모든 픽셀을 SDL readback의 3배 nearest 결과와 비교합니다.

Rust CI는 빈 target/Cargo cache로 시작해 SDL2 개발 패키지를 설치하고 고정 의존을 fetch한 후 오프라인 검사합니다. WASM CI도 잠금 메타데이터를 fetch하고 두 라이브러리만 빌드합니다. Python 작업에는 SDL2·원본이 필요하지 않습니다. 원작/기존 빌드 출력에 의존하지 않는 검사와 로컬 원본 X11 실행 증거를 분리합니다.

## 실제 X11 증거 재현

Linux에서 별도 Xvfb 디스플레이와 xdotool을 준비합니다. 출력은 새 디렉터리를 사용합니다. 공개 저장소에는 [증거 manifest](../../assets/manifests/native_view.json)만 게시하며 PNG/PPM·전체 원본 추적은 로컬에 둡니다.

```sh
cargo build --release --locked --offline -p lemmings-desktop
python3 tools/reference_assets.py verify
xvfb-run -a python3 tools/verify_native_view.py assets/local/native-view-reproduce --xdotool xdotool
```

Ubuntu에서 추가 검증 도구는 `xvfb xauth xdotool` 패키지로 준비할 수 있습니다. 이미 격리된 X11 디스플레이가 있으면 xvfb-run 없이 실행합니다. 이 관리 환경에서는 SDK·Xvfb·xkbcomp·xdotool 패키지를 apt-get download/dpkg-deb로 로컬에 풀고, proot의 파일 바인딩으로 Xvfb의 고정 xkbcomp 경로를 연결했습니다. SDL2 runtime은 수정하지 않은 공유 라이브러리 2.30.0이며 드라이버 x11, renderer software입니다.

검증 도구는 F00 실시간/적은 렌더·F02-98/99·F03 glitch on을 기존 Lemmix 함수 실행 서명과 대조합니다. X11 키 검사는 실제 창에 키를 전송한 뒤 기록된 정수 경계를 G05에서 재생합니다. 키 전달 시점·프레임 수는 실행마다 달라도 저장된 상태와 그 입력의 재생은 같아야 합니다. 마지막으로 캡처 전체 픽셀을 확인하고 PNG를 생성합니다.

## 미검증과 다음 범위

G06-01은 Linux SDL 화면/입력 연결입니다. 원본 DOS 실행·동일 장면 대조를 **G06-02**로 남겨 G06 전체는 진행 중으로 유지합니다. 직업·객체 트리거·구출/승패·음향·Windows 및 실제 GPU/물리 키보드 검증은 후속 목표입니다. X11 입력 증거는 격리된 가상 디스플레이에서 전송한 키입니다.
