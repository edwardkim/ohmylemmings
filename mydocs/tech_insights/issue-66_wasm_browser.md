# G09-03 JS/WASM 경계·실제 합성 브라우저 진입

이슈: [#66](https://github.com/edwardkim/ohmylemmings/issues/66) · 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1) · 선행 [#64](issue-64_shared_game.md)

기준 main `bdbed5c053ca19cd84fd2e9cbe238bd6dd4a104e`. `apps/web`의 `lemmings-web`은 core/game을 사용하고 wasm32에서만 정확한 wasm-bindgen0.2.129를 의존합니다. cdylib/rlib로 빌드하며 unsafe 코드를 작성하지 않습니다. core/resources/session/game과 desktop의 Rust 소스는 그대로입니다. 호스트는 시간·브라우저 이벤트·Canvas/HTTP, game/session은 기존 장면·시뮬레이션을 담당합니다.

## 바인딩 계약

[웹 안내](../../apps/web/README.md)의 WebGame API를 사용합니다. 합성 scene/pacing 문자열을 엄격히 해석하고 새 게임을 조립합니다. due는 유한 정수 마이크로초0..1시간, step은 유한 정수0..1000을 검증한 뒤 변환합니다. JS에서 u32 인자를 먼저 형변환하면 소수가 잘릴 수 있으므로 step/명령 인자를 f64로 받아 검증합니다. due는 실행하지 않고 step만 실행하며 완료 결과에서 추가 틱·입력을 받지 않습니다.

잘못된 명령·시간·숫자는 상태/큐/시간 위상을 바꾸지 않습니다. 명령은 같은 control boundary 순서로 Session에 큐잉합니다. paused는 큐잉한 의도를 포함해 빠른 연속 Pause/Resume도 같은 경계에서 처리합니다. reset은 새 게임 조립 성공 후 교체합니다. Vec 반환값은 JS가 소유하는 복사본이며 원시 포인터나 장기 생존 WASM 메모리 뷰를 노출하지 않습니다.

state는 버전1 진단 JSON입니다. 기존 세션·보조 스냅샷, 전체 actor 필드와 입력 기록을 포함하며 지형/브릭/원시 객체 맵은 별도 바이트 복사본으로 읽습니다. actor/input의 추가 진단 문자열은 고정 ASCII Rust 필드/enum 이름과 숫자/불리언의 Debug 표현입니다. 게임 규칙 재구현·브라우저 전용 시뮬레이션은 없습니다.

## 브라우저 시간·표시

rAF의 절대 timestamp를 정수 마이크로초로 양자화한 뒤 차이를 전달하므로60/120/144Hz 분할에서도 같은1초17틱을 만듭니다. FramePump는64틱 예산과 미소비 빚을 보유하며 다음 프레임으로 넘깁니다. 숨김/복귀는 시각 기준만 지우고 이미 공급한 빚은 보존합니다. 숨긴 구간은 공급하지 않습니다. 완료 결과는 시뮬레이션을 동결합니다.

View의 논리320×200 RGB를 불투명 RGBA로 변환해 Canvas에 표시합니다. UI는 합성 장면 선택·일시정지/재개·처음부터 실행을 제공합니다. 실패는 실행을 멈추고 화면에 오류 상태를 표시합니다. 게임 내부 구현 정보는 화면에 출력하지 않습니다.

## 빌드와 자동 검사

wasm-bindgen crate/CLI0.2.129와 Cargo.lock의 추가11개 registry 패키지를 고정했습니다. 기존 고정 패키지 버전은 바꾸지 않습니다. 공식 Linux x86_64 musl CLI 아카이브는 GitHub release API digest와 대조하고 정확한 실행 파일도 별도 해시로 고정합니다. tar 전체를 풀지 않고 명시한 단일 regular member만 크기/해시 검증 후 기록합니다. 도구·생성물은 assets/local에만 저장합니다.

`make check-web`은 원본 없이 네이티브 추적과 실제 headless Chromium WASM을 비교합니다. native host 계약6개는8개 기술의 직접 core 명령/안정 ID·지형·프레임, 잘못된 값의 보존, 시간 위상·새 게임·동결 결과를 검사합니다. Python acquisition5개는 손상 아카이브/멤버·누락/변조·tar 링크를 검사합니다. CI에 깨끗한 Web browser job을 추가했습니다. 사용자 제공 CDP19222에서는 별도의 visible 브라우저 검증을 수행합니다.

## 참조

- [wasm-bindgen0.2.129·MSRV](https://docs.rs/crate/wasm-bindgen/0.2.129): Rust1.93.1과의 지원 범위 확인.
- [공식 CLI](https://wasm-bindgen.github.io/wasm-bindgen/reference/cli.html)·[web 출력](https://wasm-bindgen.github.io/wasm-bindgen/reference/deployment.html)·[번들러 없는 실행](https://wasm-bindgen.github.io/wasm-bindgen/examples/without-a-bundler.html): ES module과 명시적 init/HTTP 로드.
- [MDN requestAnimationFrame](https://developer.mozilla.org/en-US/docs/Web/API/Window/requestAnimationFrame): timestamp 기반 진행·숨김 시 중단·첫 호출 시각 기준.
- [GitHub Ubuntu24.04 runner](https://github.com/actions/runner-images/blob/main/images/ubuntu/Ubuntu2404-Readme.md): 기존 Chrome 실행 파일 사용. 버전은 각 실제 실행 로그에 기록합니다.

G09 전체는 진행 중입니다. 다음 G09-04는 실제 키/포인터/기술 패널·원본 바이트 공급/선택과 레벨 재시작/전환 UX, G09-05는 원본 동일 레벨의 브라우저 완주·네이티브 전체 상태 대조입니다. 이번 합성 성공 화면을 원본 완주나 DOS 전체 호환성으로 집계하지 않습니다.
