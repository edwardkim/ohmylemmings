# Oh My Lemmings 0.3.0 — 내 MP3 배경 음악

사용자가 고른 로컬 MP3를 배경 음악으로 재생하는 릴리즈입니다. 원본 효과음을 유지하면서 음악을 교체하고 각각의 음량을 조절할 수 있습니다. Windows/macOS/Linux/Web에 원본120레벨과2D HD 자산을 포함하며 Fun01부터 바로 시작합니다.

담당 이슈: [#205 릴리즈·Pages](https://github.com/edwardkim/ohmylemmings/issues/205), 기능 [#202](https://github.com/edwardkim/ohmylemmings/issues/202). 실제 소스·CI·아카이브 해시는 동봉 `RELEASE-MANIFEST.json`을 따릅니다.

## 주요 변경

- Web에서 **MP3 선택**으로 내 음악을 불러옵니다. **원본 음악 / 내 MP3 / BGM 끄기**, 음악·효과음 개별 음량, 한 곡 반복, 준비/재생/오류/재시도·파일 해제를 제공합니다.
- Native는 MP3를 **게임 창에 끌어놓거나 `--bgm FILE`**로 선택합니다. 창 제목에 준비·재생·오류 상태를 표시하며 한 곡을 반복합니다. 별도 파일 선택 대화상자는 없습니다.
- 사용자가 선택한 음원을 업로드하거나 패키지에 복사하지 않습니다.512MB 이하 DRM 없는 MP3를 대상으로 Web 미디어 버퍼링/Native의 제한된 디코딩 큐를 사용하며 전곡을 PCM으로 선디코딩하지 않습니다. 사용자 음원은 배포에 포함하지 않습니다.
- 원본 BGM과 효과음의 합성 채널 출력 음량을 분리했습니다. 기본 설정의 기존 PCM을 보존하고 음악을 꺼도 효과음과 게임은 계속됩니다. 게임 일시정지 중 음악은 계속되며 결과/숨김에서 정지, 새 레벨/재시작/복귀에서 처음부터 재생합니다.
- v0.2.0의2D HD 리마스터·원본 화면 선택·점진적 다운로드와 실제 진행률/재시도·다음 레벨 백그라운드 준비를 유지합니다. 첫 Fun01 HD는 약19.8MB이며 엔진/게임 데이터는 별도입니다.
- 자체 코드는 MIT, OPL은 LGPL-2.1-or-later, Native MP3 디코더 Symphonia는 MPL-2.0입니다. 제3자 고지·대응 소스/정확한 버전 소스 URL·재빌드 안내를 포함합니다. 원본 게임 데이터/상표의 권리는 별도입니다.

## 다운로드와 실행

| 플랫폼 | 받을 파일 | 실행 |
|---|---|---|
| Windows11 x64 | [ohmylemmings-0.3.0-windows-x86_64.zip](https://github.com/edwardkim/ohmylemmings/releases/download/v0.3.0/ohmylemmings-0.3.0-windows-x86_64.zip) | 모두 풀고 `OhMyLemmings.exe`. data·HD 자산·SDL2.dll을 함께 유지하세요. |
| macOS Apple Silicon·Intel | [ohmylemmings-0.3.0-macos-universal2.zip](https://github.com/edwardkim/ohmylemmings/releases/download/v0.3.0/ohmylemmings-0.3.0-macos-universal2.zip) | 앱 전체를 Applications로 옮겨 실행합니다. macOS13+ 대상이며 hosted 검증 OS는15입니다. |
| Linux x86_64 | [ohmylemmings-0.3.0-linux-x86_64.tar.gz](https://github.com/edwardkim/ohmylemmings/releases/download/v0.3.0/ohmylemmings-0.3.0-linux-x86_64.tar.gz) | 모두 풀고 `./lemmings-desktop`. Ubuntu24.04/glibc2.39+·SDL2 런타임이 필요합니다. |
| Web | [ohmylemmings-0.3.0-web.zip](https://github.com/edwardkim/ohmylemmings/releases/download/v0.3.0/ohmylemmings-0.3.0-web.zip) | 모두 푼 폴더를 HTTP 서버로 제공하고 브라우저에서 접속합니다. 소리는 버튼으로 켭니다. |

[브라우저에서 플레이](https://edwardkim.github.io/ohmylemmings/). 자동 Source code ZIP 대신 플랫폼별 전체 패키지를 받으세요. ZIP 안에서 실행하지 말고 모두 풀어 주세요. 원본 data/`GAME-DATA.json`, HD 폴더/`HD-ASSETS.json`/`HD-PRESENTATION`을 함께 보존하세요. Mac은 앱 전체를 유지합니다.

## 내 음악 사용

Web은 **MP3 선택** 후 **소리 켜기**를 누릅니다. 브라우저가 재생을 차단하면 **음악 재생**으로 다시 시도하세요. 음악 시작 사건에 맞춰 재생하며 음악·효과음 slider로 균형을 조절합니다. **파일 해제**는 원본 음악으로 복귀합니다. 파일은 해당 탭에서만 사용하고 새로고침 후 다시 선택합니다.

Native는 파일 관리자가 선택한 MP3를 창으로 끌어놓거나 `./lemmings-desktop --bgm /path/to/song.mp3`로 실행합니다. Windows도 실행 파일에 `--bgm "C:\Music\song.mp3"`를 전달할 수 있습니다. F6 원본/MP3/BGM 끄기, F7 파일 재시도, F5 파일 해제/원본 복귀, `[`/`]` 음악·`,`/`.` 효과음 음량을 지원합니다. S는 전체 음소거/소리, PageUp/PageDown은 전체 음량입니다.

## 게임 조작

기술 버튼을 고른 뒤 레밍을 클릭합니다. C 등반, F 낙하산, E 폭파, K 차단, U 계단, B 수평 굴착, M 대각 굴착, D 아래 굴착, Space 일시정지, N 전체 폭파, 방향키 화면/생성속도, Esc 중단을 지원합니다. 결과 화면 R은 재시작, Enter는 다음 레벨/재도전입니다. 작은 지도로 화면을 옮길 수 있습니다. Native F8/Web 화면 선택으로 원본/HD를 전환합니다.

## 확인 범위와 제약

같은 릴리즈 소스 tree의7개 CI·4플랫폼 새 설치·120원본 레벨 로딩·HD/원본·MP3 장치/브라우저 검증과 소스/체크섬을 확인합니다. [#202 검증](https://github.com/edwardkim/ohmylemmings/blob/v0.3.0/mydocs/reports/issue-202_local_mp3.md)에는 메인테이너 제공 두 MP3의 Web 재생/곡 끝 반복/교체/효과음 동시 출력·Native 실제 SDL 출력과 파일 내용 보존을 기록했습니다. 메인테이너의 새 MP3 실제 청취 응답은 #202에서 계속 추적하며, 게시 승인을 청취 합격으로 간주하지 않습니다. 원본 BGM #165는 메인테이너 청취 인수를 완료했습니다.

Windows 지원 대상은 Windows11이며 Windows10은 지원하지 않습니다. hosted Windows 검증 OS는 Server2022입니다. Mac은 ad-hoc 서명/미공증이며 Apple Developer ID 서명은 없습니다. 차단되는 경우 앱을 열어 본 뒤 시스템 설정 > 개인정보 보호 및 보안의 «그래도 열기»는 [Apple 안내](https://support.apple.com/102445)를 참고하세요. 성능/60fps는 #113 NOT READY로 유지하며 디지털 출력 검증을 실제 스피커 청취·전체120레벨의 새 HD 완주 판정으로 확대하지 않습니다.

## 체크섬·대응 소스

[SHA256SUMS](https://github.com/edwardkim/ohmylemmings/releases/download/v0.3.0/SHA256SUMS), archive별 `.sha256`, [RELEASE-MANIFEST.json](https://github.com/edwardkim/ohmylemmings/releases/download/v0.3.0/RELEASE-MANIFEST.json)에서 출처를 확인하세요. 각 패키지와 [대응 소스](https://github.com/edwardkim/ohmylemmings/releases/download/v0.3.0/ohmylemmings-0.3.0-source.tar.gz)에 라이선스·OPL 변경/재빌드/재링크 안내와 registry 의존성의 고정 버전/소스 URL을 제공합니다. v0.1.0/v0.2.0의 태그·배포 파일·출시 노트는 이전 그대로 보존합니다.
