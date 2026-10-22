# Oh My Lemmings 0.1.0

오래된 DOS 레밍즈를 Rust로 옮겨 재미로 플레이해 보는 토이프로젝트의 첫 다운로드 릴리즈입니다. 원본 그래픽·전체120레벨·음악과 효과음이 포함되어 있으며 **Fun 01부터 바로 시작**합니다.

## 다운로드와 실행

| 플랫폼 | 받을 파일 | 실행 |
|---|---|---|
| Windows x64 | [ohmylemmings-0.1.0-windows-x86_64.zip](https://github.com/edwardkim/ohmylemmings/releases/download/v0.1.0/ohmylemmings-0.1.0-windows-x86_64.zip) | 모두 압축 해제 후 `OhMyLemmings.exe` 실행. data/SDL2.dll을 함께 유지하세요. |
| macOS Apple Silicon·Intel | [ohmylemmings-0.1.0-macos-universal2.zip](https://github.com/edwardkim/ohmylemmings/releases/download/v0.1.0/ohmylemmings-0.1.0-macos-universal2.zip) | 모두 압축 해제 후 `OhMyLemmings.app`을 Applications로 옮겨 실행. macOS13 이상 대상이며 실제 검증은 macOS15에서 진행했습니다. |
| Linux x86_64 | [ohmylemmings-0.1.0-linux-x86_64.tar.gz](https://github.com/edwardkim/ohmylemmings/releases/download/v0.1.0/ohmylemmings-0.1.0-linux-x86_64.tar.gz) | 모두 압축 해제 후 `./lemmings-desktop`. Ubuntu24.04/glibc2.39 이상과 SDL2 런타임이 필요합니다. Ubuntu: `sudo apt-get install libsdl2-2.0-0`. |
| Web | [ohmylemmings-0.1.0-web.zip](https://github.com/edwardkim/ohmylemmings/releases/download/v0.1.0/ohmylemmings-0.1.0-web.zip) | 모두 압축 해제 후 해당 폴더에서 `python3 -m http.server 8080`, 브라우저로 http://localhost:8080/ 접속. 소리는 버튼을 눌러 켭니다. |

GitHub가 별도로 표시하는 Source code ZIP 대신 위 플랫폼별 파일을 받으세요. ZIP 안에서 바로 실행하지 말고 전체를 풀어 주세요. Linux/Windows는 `--difficulty tricky --level 1` 같은 옵션으로 다른 레벨을 고를 수 있고, 웹은 난이도/레벨 선택을 제공합니다.

## 조작

하단 기술 버튼을 고른 뒤 레밍을 클릭합니다. 키보드는 C 등반, F 낙하산, E 폭파, K 차단, U 계단, B 수평 굴착, M 대각 굴착, D 아래 굴착입니다. Space 일시 정지, S 소리 켜기/음소거, 방향키 화면/생성속도, N 전체 폭파, Esc 중단을 지원합니다. 결과 화면 R은 재시작, Enter는 다음 레벨/재도전입니다. 작은 지도를 끌어 화면을 옮길 수 있습니다.

## 확인된 제약

Mac 앱은 ad-hoc 서명이며 Apple Developer ID 서명/공증이 없습니다. 다운로드 후 차단되는 경우 앱을 열어 본 뒤 시스템 설정 > 개인정보 보호 및 보안 > 그래도 열기는 [Apple 안내](https://support.apple.com/102445)를 참고하세요. 기본 보안 설정을 전역 해제할 필요는 없습니다.

플랫폼별 실제 CI에서120개 레벨 로딩·원본 화면/음향 데이터·재시작/다음 레벨을 검증했습니다. 모든 PC에서60fps를 보장하는 릴리즈는 아니며 성능 검증은 #113에서 계속합니다. 저장 해법을 이용한 기능 검증과 실제 스피커 청취는 구분합니다.

## 파일 확인·소스

`SHA256SUMS`와 각 archive의 `.sha256`, `RELEASE-MANIFEST.json`에서 버전·소스 커밋·검증 CI·체크섬을 확인할 수 있습니다. 각 패키지와 `ohmylemmings-0.1.0-source.tar.gz`에 대응 소스·OPL LGPL 라이선스/재빌드 안내가 들어 있습니다. 원본 게임 데이터의 출처/해시는 동봉 GAME-DATA.json에 기록되어 있습니다. 이 릴리즈로 프로젝트 전체나 원본 게임에 새로운 라이선스를 부여하지 않습니다.

담당 [#124](https://github.com/edwardkim/ohmylemmings/issues/124). [문제 제보](https://github.com/edwardkim/ohmylemmings/issues).
