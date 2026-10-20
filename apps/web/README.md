# Web preview

공통 game/session을 실제 WASM으로 실행하는 합성 브라우저 연습장입니다. [#66 계약](../../mydocs/tech_insights/issue-66_wasm_browser.md)을 따릅니다. 현재 합성 장면 선택·일시정지/재개·처음부터 실행을 지원합니다. 원본 파일 선택, 게임 화면의 키/포인터/패널 조작과 원본 레벨 전환은 G09-04에서 연결합니다.

## 준비와 실행

Linux x86_64, Rust1.93.1, Python3, Chrome/Chromium을 사용합니다. 최초 준비 때만 고정 crate와 공식 wasm-bindgen CLI를 내려받습니다. Node/NPM·번들러는 필요하지 않습니다.

```sh
source assets/local/dev-tools/env.sh  # 로컬 준비 스크립트가 있는 환경
rustup target add wasm32-unknown-unknown
cargo fetch --locked
python3 tools/web_build.py fetch
make web
make serve-web
```

브라우저에서 `http://127.0.0.1:8080/`을 엽니다. 서버는 loopback에서 생성된 웹 폴더만 제공합니다. 포트 변경은 `python3 tools/web_build.py serve --port 8082`로 합니다. `make web`과 검사 빌드는 준비 후 Cargo 오프라인 모드로 실행하며, 누락된 CLI는 fetch 안내와 함께 실패합니다.

wasm-bindgen crate/CLI는0.2.129로 맞추고 [도구 명세](../../assets/manifests/web_tools.json)의 아카이브·정확한 실행 파일 크기/SHA-256을 확인합니다. 실행 파일은 ignored `assets/local/web-tools/`, 생성 JS/WASM/타입 선언·실행 페이지는 ignored `assets/local/web/`에 둡니다. 생성물과 원본 파일을 저장소에 커밋하지 않습니다.

## 호스트 API

`WebGame(scene, pacing)`는 `builder/basher/climber/floater/win/loss/mixed/clock/zero/particles` 합성과 `reference/dos` pacing을 받습니다. 현재 API는 합성 장면 전용입니다.

- `due(integer_microseconds)`는0..3600000000의 유한 정수 시간을 받고 호출 수만 계산합니다. 실제 진행은 `step(count)`로 하며0..1000 유한 정수만 받습니다. 완료 결과에서 step은0을 돌려주고 입력을 거절합니다.
- `command(name, value)`는 Pause/Resume/Nuke의 인자0, 레벨 초기값..99의 SetReleaseRate,8개 Assign*의0..u32::MAX 정수 ID를 검증한 후 같은 Session에 큐잉합니다. 잘못된 명령/숫자는 큐를 바꾸지 않습니다. `paused()`는 현재 경계에 큐잉한 Pause/Resume까지 반영합니다.
- `reset(scene,pacing)`는 새 게임 조립 성공 후 교체합니다. 실패하면 이전 게임·View·시간 위상을 보존합니다.
- `frame()`은320×200 RGB 복사본, terrain/bricks/objects는 각 원시 맵 복사본을 반환합니다. `state()`는 버전1 진단 JSON으로 세션·모든 actor 필드·스킬/완료 보조 상태·입력 기록을 제공합니다. 렌더 호출은 시뮬레이션을 진행하지 않습니다. 사용을 마친 인스턴스는 `free()`로 해제합니다.

브라우저 FramePump는 rAF 절대 시각을 정수 마이크로초로 양자화하고 차이를 전달합니다. 첫 프레임은 시간 기준만 잡습니다. 프레임당64틱을 소비하고 남은 빚을 보존합니다. 숨김/복귀 때 시간 기준만 새로 잡아 숨김 구간을 공급하지 않으며 이미 공급한 빚은 보존합니다. 완료 결과는 추가 틱을 받지 않습니다. Canvas는 RGB를 불투명 RGBA로 표시합니다.

## 검증

```sh
make check-web
make check
```

`check-web`은 네이티브 추적 바이너리와 실제 JS/WASM을 별도 Chrome/Chromium 임시 프로필에서 실행합니다.13개 합성 사례의 각 전체 상태와 모든 지형/브릭/객체 바이트의 SHA-256, 렌더1/7의 모든 논리 RGB·Canvas readback, 복사본/오류/결정성/60·120·144Hz/프레임 빚/숨김 경계를 비교합니다. 원본이나 CDP19222가 없어도 실행됩니다. 사용자 CDP19222에서의 실제 화면·일시정지/재개·구출/재실행 검증은 #66에 별도로 기록합니다.
