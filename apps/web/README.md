# Web preview

공통 game/session을 실제 WASM으로 실행하는 브라우저 게임입니다. [#66 WASM 계약](../../mydocs/tech_insights/issue-66_wasm_browser.md)과 [#68 입력·자산 계약](../../mydocs/tech_insights/issue-68_web_input_assets.md)을 따릅니다. 합성 연습과 사용자가 선택한 원본 네 난이도 일반 레벨에서 기술 패널·키·포인터·미니맵, 재시작·다음 레벨을 지원합니다.

## 준비와 실행

Linux x86_64, Rust1.93.1, Python3, Chrome/Chromium을 사용합니다. 최초 준비 때만 고정 crate와 공식 wasm-bindgen CLI를 내려받습니다. 게임 빌드/검사에 Node/NPM·번들러는 필요하지 않습니다. 통합 검사의 독립 SDL 재생에는 SDL2 개발 헤더도 필요합니다([설치 안내](../desktop/README.md)).

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

`WebGame(scene, pacing)`는 `builder/basher/climber/floater/win/loss/mixed/clock/zero/particles` 합성과 `reference/dos` pacing을 받습니다. 합성 승리 장면의 다음 레벨은 실패 장면이며 원본은 Fun/Tricky/Taxing/Mayhem 각1..30과 특수4개 배경을 지원합니다. 전수 완주 인증은 G11 후속입니다.

- `due(integer_microseconds)`는0..3600000000의 유한 정수 시간을 받고 호출 수만 계산합니다. 실제 진행은 `step(count)`로 하며0..1000 유한 정수만 받습니다. 완료 결과에서 step은0을 돌려주고 입력을 거절합니다.
- `command(name, value)`는 Pause/Resume/Nuke의 인자0, 레벨 초기값..99의 SetReleaseRate,8개 Assign*의0..u32::MAX 정수 ID를 검증한 후 같은 Session에 큐잉합니다. 잘못된 명령/숫자는 큐를 바꾸지 않습니다. `paused()`는 현재 경계에 큐잉한 Pause/Resume까지 반영합니다.
- `reset(scene,pacing)`는 새 게임 조립 성공 후 교체합니다. 실패하면 이전 게임·View·시간 위상을 보존합니다.
- `input_log()`는 데스크톱 재생 형식의 step/순번/명령을 반환합니다. `state()`의 Nuke 진단은 is_nuked/assigning/next_index/max_lemmings를 포함합니다.
- `frame()`은320×200 RGB 복사본, terrain/bricks/objects는 각 원시 맵 복사본을 반환합니다. `state()`는 버전1 진단 JSON으로 세션·모든 actor 필드·스킬/완료 보조 상태·입력 기록을 제공합니다. 렌더 호출은 시뮬레이션을 진행하지 않습니다. 사용을 마친 인스턴스는 `free()`로 해제합니다.

`WebAssets`는 `insert(name, Uint8Array)`로 바이트를 복사합니다. MAIN.DAT/ODDTABLE.DAT/VGALEMMI.EXE, LEVEL000..009.DAT, GROUND0O..4O.DAT, VGAGR0..4.DAT만 허용하고 ASCII 대소문자를 통일합니다. 경로·미지원 이름·대소문자 중복·1 MiB 초과 파일은 변경 전에 거절합니다. 로드한 게임은 불변 바이트 묶음을 유지하므로 이후 은행에 파일을 추가해도 재시작/전환에 영향을 주지 않습니다. 사용한 은행은 `free()`할 수 있습니다.

- `load_original(bank, level)`은 Fun1..30 유한 정수와 파일을 검사하고 새 DOS pacing 게임을 조립한 후 교체합니다. `start_level(level)`도 Fun 호환 진입점입니다. 새 `load_original_rank(bank, difficulty, level)`·`start_level_rank(difficulty, level)`로 난이도를 지정하며 `difficulty()`·View difficulty/next_difficulty로 전체 ID를 조회합니다. 누락/손상 파일 교체 실패는 기존 실행·View·시계를 보존합니다.
- `restart_level()`은 진행 중에도 현재 레벨의 새 세션을 만듭니다. `transition('restart'|'continue')`는 완료 결과에서만 작동하며 실패면 같은 레벨, 성공이면 다음 레벨, 마지막 성공이면 false입니다. 전환 로드 실패는 결과와 세션을 보존하고 결과 오류 표시만 켭니다.
- `key(code, repeat)`와 `pointer(kind, x, y, button, buttons, clicks)`는 플랫폼 이벤트를 공유 패널/포인터/큐에 연결합니다. 좌표는 논리 정수, button0..2/buttons0..31/clicks0..255를 검증합니다. 반환값 none/restart/continue/exit는 호스트가 처리하는 장면 명령입니다. `release_input()`은 포커스/숨김 이탈 시 누름과 선택 기억을 지웁니다. `view_state()`는 별도의 표현 진단 JSON입니다.

## 원본 파일과 조작

웹 페이지의 **게임 파일 선택**에서 보유한 DOS 게임 폴더의 DAT와 VGALEMMI.EXE를 함께 선택합니다. 관련 없는 파일은 건너뛰며 파일 내용은 네트워크로 보내지 않습니다. 필요한 파일 누락은 이름과 함께 화면에 표시됩니다. 다음 레벨에 필요한 파일도 처음에 함께 선택하면 레벨 열기/재시작/전환에 같은 바이트를 사용합니다. 새로고침하면 다시 선택해야 합니다. 이 설명은 `make web` 개발 출력에 해당합니다. `make package-web` 다운로드는 검증된 원본28개를 동봉하여 자동 로딩하며 새로고침해도 별도 파일 선택 없이 원본 Fun01을 시작합니다. 특수 배경 Fun22/Tricky14/Taxing15/Mayhem22에는 VGASPEC0..3.DAT도 함께 선택합니다.

화면을 클릭해 키보드 포커스를 둡니다. C/F/E/K/U/B/M/D는 등반/낙하산/폭파/차단/계단/수평/대각/아래 굴착입니다. 기술 선택 후 레밍을 클릭합니다. Space 일시 정지, ↑↓ 출현 속도, ←→/Home/End 화면 이동, N 전체 폭파, Esc 중단 후 버튼으로 복귀합니다. 하단 패널 속도 버튼은 누름 반복, 전체 폭파는 더블클릭, 작은 지도는 끌기로 이동합니다. 우클릭을 누른 채 왼쪽 버튼을 눌러 기존 선택 규칙도 사용할 수 있습니다. 결과에서 Enter/왼쪽 클릭은 다음/재도전, R은 현재 레벨 재시작입니다.

브라우저 FramePump는 rAF 절대 시각을 정수 마이크로초로 양자화하고 차이를 전달합니다. 첫 프레임은 시간 기준만 잡습니다. 프레임당64틱을 소비하고 남은 빚을 보존합니다. 숨김/복귀 때 시간 기준만 새로 잡아 숨김 구간을 공급하지 않으며 이미 공급한 빚은 보존합니다. 완료 결과는 추가 틱을 받지 않습니다. Canvas는 RGB를 불투명 RGBA로 표시합니다.

## 검증

```sh
make check-web
make check
```

`check-web`은 네이티브 추적 바이너리와 실제 JS/WASM을 별도 Chrome/Chromium 임시 프로필에서 실행합니다.29개 합성/창작 바이트 사례의 각 전체 상태와 모든 지형/브릭/객체 바이트의 SHA-256, 렌더1/7의 모든 논리 RGB·Canvas readback, 복사본/오류/결정성/60·120·144Hz/프레임 빚/숨김 경계를 비교합니다. 원본이나 CDP19222가 없어도 실행됩니다. 사용자 CDP19222의 실제 키/마우스/파일 선택과 원본 Fun01/02 화면, 합성 승리→다음→실패→재도전은 #68에 기록합니다.

검증된 로컬 원본의 초기150틱·기술 선택/할당·카메라·재시작을 추가 비교하려면 아래 명령을 사용합니다. 원본 파생 자료는 ignored assets/local에만 기록하도록 제한합니다.

```sh
python3 tools/check_web.py --evidence assets/local/web-original-check --original-assets assets/local/dos-original
```

이 선택 검사는 기존 Fun1~3에 Tricky1/Taxing1/Mayhem1을 추가합니다. 정확한 실행 수치는 [#87 보고서](../../mydocs/reports/issue-87_rank_selection.md)와 증거 목록에서 확인합니다. 원본 한 레벨 브라우저 성공·실패 통합 완주와 시도별 네이티브 비교는 [G09-05 / #70](../../mydocs/reports/issue-70_web_playthrough.md)에서 검증했습니다.


`check-web`은 추가로 `tools/check_web_journey.py`를 실행합니다. 실제 제품 DOM과 WASM의 합성 성공→재시작→Nuke 실패→재도전→전원 구출→다음 레벨 새 ID0을656개 전체 상태/57Canvas에서 네이티브와 비교하고, 독립 SDL12회와 WASM12회 시간/렌더 재생을 검사합니다. 원본/CDP 없이 실행됩니다. 증거를 남기려면 `python3 tools/check_web_journey.py --evidence assets/local/web-journey-check`를 사용합니다.

로컬 개발 관측은 `app.js`의 `observe(listener)`로 attach/reset/input/before/after/present를 받으며 반환 함수로 해제합니다. 기본 실행에는 관측자가 없습니다. `tools/web/journey_recorder.js`는 전체 시도/순서 입력/매 경계 맵 변형과 실제 Canvas를 기록하고 `journey_replay.js`는 같은 입력의 시간/렌더 분할을 검증합니다. 자세한 초기/최종 경계·독립 재생·원본 보관 계약은 [#70 계약](../../mydocs/tech_insights/issue-70_web_playthrough.md)을 따릅니다.

## 사운드 / #80

새로고침 후 소리 켜기 버튼 또는 S로 활성화합니다. 음량 slider/PageUp/PageDown, 명시적 음소거·숨김/복귀·결과/재시작을 처리합니다. 음악은 Pause 중에도 계속되며 음소거/숨김 후 현재 곡을 처음부터 재개합니다. 원본 파일 선택에 ADLIB.DAT를 포함합니다. API/자원/장치 오류는 음향 상태에 표시하고 게임을 유지합니다. 연습 모드는 새로 만든 음원입니다. AudioBufferSource 예약 경로로 LAN HTTP를 지원합니다.

`WebAudio`의 prepare/render는 게임을 tick하지 않는 owned PCM API입니다. `AudioHost`가 사용자 활성화와 bounded node/큐를 담당합니다. Footer의 라이선스 페이지에서 LGPL 전문과 이 빌드의 소스 묶음·WASM/소스 해시를 제공합니다. 추출본은 Git 없이 수정/재링크할 수 있습니다. [상세 계약](../../mydocs/tech_insights/issue-80_audio_output.md).

통합 사운드 검사는 `python3 tools/sound_playthrough.py`이며 `make check-web`에도 포함합니다. 선택적 실제 DOS/원본 검증과 남은 청취 범위는 [#82](../../mydocs/tech_insights/issue-82_audio_playthrough.md)를 따릅니다.

[#87 난이도 계약](../../mydocs/tech_insights/issue-87_rank_selection.md): UI 난이도/번호 선택·동일 ID 재시작·성공 시 난이도 경계 전환과 Mayhem30 종료를 처리합니다. 창작 바이트의 실제 제품 DOM8개 선택/재시작·3개 경계 Continue·파일 교체 실패를 검사합니다. 새 journey 기록은 difficulty를 포함하며 과거 누락 필드는 Fun으로 읽습니다.

초기 진단 `asset_info()`·`materials()`는 현재 실행의 불변 초기 수치/팔레트와 강철/일방향 셀 복사본을 제공합니다. 실제 SDL/시간 분할/WASM/Canvas 전수 재현은 [#91 계약](../../mydocs/tech_insights/issue-91_initial_replay.md)을 따릅니다.

## 전체 게임 HTTP 배포 (#122)

`python3 tools/reference_assets.py fetch` 후 `make package-web`. `ohmylemmings-0.1.0-web.zip` 전체를 풀고 정적 HTTP 서버에서 제공합니다. 하위 경로도 지원합니다. ZIP을 직접 열거나 index.html만 복사하지 마세요. 원본28개·120레벨·음원·대응 소스와 라이선스가 포함됩니다. 소리는 사용자 버튼으로 켭니다. `GAME-DATA.json`/data 누락은 시작 실패로 표시합니다. 패키징 시 SHA256과 크기를 고정 원본 manifest로 검사하며 HTTP 로더는 목록·크기와 WASM 파서를 사용하므로 LAN HTTP에서도 실행됩니다. 원본 제외 연습 패키지는 `tools/release_package.py web --development --runtime assets/local/web`으로 명시합니다. [검증 범위](../../mydocs/reports/issue-122_linux_web_full_game.md).
