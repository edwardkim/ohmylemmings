# Linux·Web 전체 게임 패키지

담당 [#122](https://github.com/edwardkim/ohmylemmings/issues/122), 상위 [#115](https://github.com/edwardkim/ohmylemmings/issues/115).

사용자 승인으로 Linux/Web 다운로드에 원본28개(그래픽·120개 레벨·음원)를 동봉한다. 기본 원본 Fun01, GAME-DATA 출처/고정 SHA·크기, 대응 소스·OPL 라이선스·내외부 체크섬을 제공한다. 패키지 입력은 명시 목록만 사용하며 원본/캡처는 ignored assets/local 또는 artifact에 둔다. `--development`는 기존 원본 없는 연습 패키지를 명시적으로 만든다.

Linux: `ohmylemmings-0.1.0-linux-x86_64.tar.gz` 전체를 풀고 `./lemmings-desktop` 실행. Ubuntu24.04/glibc2.39+·SDL2 런타임이 필요하며 Ubuntu는 `sudo apt-get install libsdl2-2.0-0`. 실행 파일 옆 data를 사용하므로 다른 CWD에서도 실행된다. `--difficulty tricky --level 1`로 선택한다. cargo 개발 출력 옆 data가 없으면 기존 assets/local을 사용한다.

Web: `ohmylemmings-0.1.0-web.zip` 전체를 HTTP 서버에 배치한다. 예: `python3 -m http.server 8080` 후 http://localhost:8080/. 하위 경로/LAN HTTP도 지원한다. 동봉 원본 자동 로딩과 사용자 버튼 음향 활성화를 유지한다. 패키지 HTML은 원본 목록이 필수임을 표시해 목록 누락 시 연습으로 바뀌지 않는다. HTTP 로더는 목록·크기와 WASM 파서를 검사한다. 암호학적 SHA256 검사는 packager/verifier 단계이며 secure-context API를 HTTP 시작 조건으로 요구하지 않는다. 파일 누락/로딩 실패는 파일명과 화면 오류로 알린다.

패키징3개와 Linux 기본경로1개 red→green; 기존 Windows/Mac/개발 패키징 포함20개 PASS. Desktop41개·Clippy PASS. 실제 새 압축 해제 Linux는 한글/공백 경로·다른 CWD에서 기본 Fun01 첫 전체 RGB/SDL 확대 readback·120개 로딩·저장 Fun01 입력 성공→Restart/Continue와 새 시도 상태/지형·비영 원본 SDL PCM·MAIN 누락을 통과했다. Web은 실제 WASM/DOM/Canvas에서 같은 첫 프레임 SHA(1539ca2a…),120개 선택·원본 WebAudio PCM·Fun01 결과 UI 전환·새로고침·하위 경로·소스/라이선스 HTTP 전달을 통과했다. 로더 합성 오류8개와 실제 필수 목록/MAIN 누락도 통과했다. Python224개와 감사 pins, 기존 합성 WASM/native66회·11,960상태/6,877화면·Canvas 및 입력/음향 명령 회귀를 통과했다.

Linux 검증은 SDL dummy video/audio·readback이며 WebAudio는 실제 AudioContext에 자동 활성화 플래그를 사용한다. 사용자 물리 스피커 청취·60fps 인증은 별도다. 기존120개 해법/240사례를 새로 전수 완주했다고 주장하지 않는다. #113 성능 이월·G11 NOT READY를 유지한다. CI는 기존 합성 회귀 다음 원본50개 acquisition/verify와 새 압축 해제 검증을 수행하고14일 Linux/Web archive·sha·숫자 요약·원본 PNG/PPM 화면을 보관한다. 정확한 source/run/artifact와 PR/병합은 #122에 기록한다.

공인 Developer ID가 없는 Mac은 자체 tap의 소스 빌드 Formula를 후속으로 사용할 수 있다. 현재 사전 빌드 `.app`을 Cask로 설치해도 Gatekeeper 요구가 없어지지 않으며 공식 Cask는 기본 macOS 설정에서 통과해야 한다. [Formula](https://docs.brew.sh/Formula-Cookbook), [Cask 신뢰 모델](https://docs.brew.sh/Homebrew-Security-and-Supply-Chain#casks-have-a-different-trust-model). tap 구현/공식 제출·유료 인증서 발급·정식 릴리스 게시를 이번 작업의 완료로 표시하지 않는다.
