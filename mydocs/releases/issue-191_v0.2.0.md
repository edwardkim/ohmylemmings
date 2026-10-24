# Oh My Lemmings 0.2.0 — 2D 리마스터

DOS 레밍즈를 Rust로 옮긴 토이프로젝트의2D 리마스터 릴리즈입니다. 새 캐릭터·지형·시설·글꼴·게임 UI를 HD로 표시하며 원본120레벨과 원래 게임 규칙을 유지합니다. 전체 게임 데이터와 HD 자산이 포함되어 Fun01부터 바로 시작합니다.

버전 적용 [#191](https://github.com/edwardkim/ohmylemmings/issues/191), 정식 게시·Pages [#194](https://github.com/edwardkim/ohmylemmings/issues/194). 실제 릴리즈 소스·CI·아카이브 해시는 동봉 `RELEASE-MANIFEST.json`을 따릅니다.

## 주요 변경

- 캐릭터28동작337프레임·일반5재질·특수4장면·54객체540키·13조작 아이콘,95개 AA 글꼴과 게임 패널/결과 UI를1920×1080급 화면에 합성합니다. 원본 지형 마스크·충돌·레벨·입력은 유지합니다.
- Linux/Windows/macOS/Web 패키지에 전체 HD 카탈로그와 원본 게임을 넣고 HD로 시작합니다. Native는 F8 또는 `--original`, Web은 화면 선택으로 같은 게임의 원본 화면을 사용할 수 있습니다.
- 공통 OPL의 음표 재시작을 복원해 진행 중 BGM이 감쇠하던 문제를 수정했습니다. 원본21곡×60초·효과음 혼합, 고정 C buffered 및 Native/WASM 대조를 수행했습니다. 실제 사람의 청취 확인은 #165에서 계속 추적합니다.
- 프로젝트 MIT·제3자 고지·글꼴 OFL·OPL LGPL 대응 소스/재빌드 안내와 원본 데이터의 출처/해시를 포함합니다. 원본 게임 데이터와 상표는 프로젝트 MIT 허가의 대상이 아닙니다.

## 다운로드와 실행

| 플랫폼 | 받을 파일 | 실행 |
|---|---|---|
| Windows11 x64 | [ohmylemmings-0.2.0-windows-x86_64.zip](https://github.com/edwardkim/ohmylemmings/releases/download/v0.2.0/ohmylemmings-0.2.0-windows-x86_64.zip) | 모두 풀고 `OhMyLemmings.exe`. data/HD 자산과 SDL2.dll을 함께 유지하세요. |
| macOS Apple Silicon·Intel | [ohmylemmings-0.2.0-macos-universal2.zip](https://github.com/edwardkim/ohmylemmings/releases/download/v0.2.0/ohmylemmings-0.2.0-macos-universal2.zip) | 앱 전체를 Applications로 옮겨 실행. macOS13+ 대상이며 hosted 검증 OS는15입니다. |
| Linux x86_64 | [ohmylemmings-0.2.0-linux-x86_64.tar.gz](https://github.com/edwardkim/ohmylemmings/releases/download/v0.2.0/ohmylemmings-0.2.0-linux-x86_64.tar.gz) | 모두 풀고 `./lemmings-desktop`. Ubuntu24.04/glibc2.39+·SDL2 런타임이 필요합니다. |
| Web | [ohmylemmings-0.2.0-web.zip](https://github.com/edwardkim/ohmylemmings/releases/download/v0.2.0/ohmylemmings-0.2.0-web.zip) | 모두 푼 폴더를 HTTP 서버로 제공하고 브라우저에서 접속합니다. 소리는 버튼으로 켭니다. |

[브라우저에서 플레이](https://edwardkim.github.io/ohmylemmings/). GitHub의 자동 Source code ZIP 대신 플랫폼별 전체 패키지를 받으세요. ZIP 내부에서 실행하지 말고 모두 풀어 주세요. HD 자산 폴더와 `HD-ASSETS.json`/`HD-PRESENTATION`, 원본 data/`GAME-DATA.json`을 함께 보존하세요. Mac은 앱 전체를 유지하세요.

## 조작

기술 버튼을 고른 뒤 레밍을 클릭합니다. C 등반, F 낙하산, E 폭파, K 차단, U 계단, B 수평 굴착, M 대각 굴착, D 아래 굴착, Space 일시 정지, S 소리/음소거, 방향키 화면/생성속도, N 전체 폭파, Esc 중단을 지원합니다. 결과 화면 R은 재시작, Enter는 다음 레벨/재도전입니다. 작은 지도로 화면을 옮길 수 있습니다.

## 확인 범위와 제약

HD 리마스터는 [#187](../reports/issue-187_hd_release.md), BGM은 [#165](../reports/issue-165_bgm.md), 릴리즈는 [#194](../reports/issue-194_release_v020.md)를 따릅니다. 동일 릴리즈 소스 tree의7개 CI·4플랫폼 새 설치·120원본 레벨 로딩·HD/원본 선택·재시작/다음 레벨·소스/체크섬을 검증합니다. hosted Windows 환경은 Server2022이며 이전 Windows11 인수를 이번 버전의 새 수동 판정으로 확대하지 않습니다. Windows10은 지원하지 않습니다.

Mac은 ad-hoc 서명/미공증이며 Apple Developer ID 서명은 없습니다. 차단되는 경우 앱을 열어 본 뒤 시스템 설정 > 개인정보 보호 및 보안의 «그래도 열기»는 [Apple 안내](https://support.apple.com/102445)를 참고하세요.60fps/고성능 호스트 성능은 #113 NOT READY로 유지합니다. 디지털 PCM·장치 smoke를 실제 스피커 청취나 전체120레벨의 새 HD 완주 판정으로 간주하지 않습니다.

## 체크섬·대응 소스

[SHA256SUMS](https://github.com/edwardkim/ohmylemmings/releases/download/v0.2.0/SHA256SUMS), 개별 archive `.sha256`, [RELEASE-MANIFEST.json](https://github.com/edwardkim/ohmylemmings/releases/download/v0.2.0/RELEASE-MANIFEST.json)에서 출처를 확인하세요. 각 패키지와 [대응 소스](https://github.com/edwardkim/ohmylemmings/releases/download/v0.2.0/ohmylemmings-0.2.0-source.tar.gz)에 LGPL 고지·수정 내용·재빌드/재링크 안내를 제공합니다.0.1.0의 태그·배포 파일·출시 노트는 이전 버전 그대로 보존합니다.
