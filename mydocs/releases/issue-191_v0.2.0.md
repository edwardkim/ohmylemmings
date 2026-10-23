# Oh My Lemmings 0.2.0 — 2D 리마스터

담당 [#191](https://github.com/edwardkim/ohmylemmings/issues/191). 메인테이너가 확정한 리마스터 버전입니다. **현재 배포 후보 준비 중이며, v0.2.0 태그·공개 릴리즈·Pages 게시 전입니다.** 기존 공개0.1.0은 보존합니다.

## 변경 사항

- 새 캐릭터28동작337프레임·5테마 재질·특수4장면·54객체540키·13조작 아이콘과 HD 글꼴/현재 전체 게임 UI로1920×1080급 화면을 제공합니다. 원본 지형 마스크와 레벨·규칙·입력은 유지합니다.
- Linux/Windows/macOS/Web 전체 게임 패키지에 HD 자산을 포함하고 HD로 시작합니다. Native는 F8 또는 시작 옵션 `--original`, Web은 화면 선택으로 원본 모드를 사용합니다.
- #165에서 공통 OPL의 음표 재시작을 복원하여 진행 중 BGM이 감쇠하던 문제를 수정합니다.60초 곡/효과음 혼합과 C/Native/WASM 결과를 검증하며 실제 청취는 별도로 기록합니다.
- 원본120레벨·음원/효과음 데이터·출처/해시·MIT/제3자 고지·OPL LGPL 대응 소스와 재빌드 안내를 포함합니다. 원본 게임 데이터/상표는 프로젝트 MIT 허가의 대상이 아닙니다.

## 배포 후보 파일과 실행

아래는0.2.0 CI가 생성하는 파일명이며 아직 공개 다운로드 링크가 아닙니다.

| 플랫폼 | 파일 | 실행 |
|---|---|---|
| Windows11 x64 | `ohmylemmings-0.2.0-windows-x86_64.zip` | 모두 풀고 `OhMyLemmings.exe`. data/HD 자산과 SDL2.dll을 함께 유지하세요. |
| macOS Apple Silicon·Intel | `ohmylemmings-0.2.0-macos-universal2.zip` | 앱 전체를 Applications로 옮겨 실행. macOS13+ 대상이며 hosted 검증 OS는15입니다. |
| Linux x86_64 | `ohmylemmings-0.2.0-linux-x86_64.tar.gz` | 모두 풀고 `./lemmings-desktop`. Ubuntu24.04/glibc2.39+·SDL2 런타임이 필요합니다. |
| Web | `ohmylemmings-0.2.0-web.zip` | 모두 푼 폴더를 HTTP 서버로 제공하고 브라우저에서 접속합니다. 소리는 버튼으로 켭니다. |

게임 조작은 기술 버튼/기존 단축키·Pause/Nuke·미니맵·재시작/다음 레벨을 유지합니다. HD 자산 폴더와 `HD-ASSETS.json`/`HD-PRESENTATION`, 원본 data/`GAME-DATA.json`을 함께 보존하세요. Mac은 앱 전체를 유지하세요.

## 확인 범위와 남은 사항

실제0.2.0 소스 트리의7개 플랫폼 CI와 전체 패키지·소스·체크섬을 확인한 뒤 게시 후보를 확정합니다. HD 리마스터는 [#187 보고서](../reports/issue-187_hd_release.md), BGM 수정은 [#165 보고서](../reports/issue-165_bgm.md)를 따릅니다.0.1.0의 당시 수동 인수나 이전 단계의 전체 해법 기록을 이번0.2.0의 새 수동/전체 완주 판정으로 확대하지 않습니다.

Windows10은 지원하지 않습니다. Mac은 ad-hoc 서명/미공증이며 Apple Developer ID 서명은 없습니다.60fps/고성능 호스트 성능은 #113 NOT READY로 유지합니다. 공개 게시 전 이 문서를 공개 다운로드 완료로 해석하지 마세요.
