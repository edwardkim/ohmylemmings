# Oh My Lemmings 0.3.1 — Native MP3 선택·앱 아이콘

담당 이슈: https://github.com/edwardkim/ohmylemmings/issues/208

네이티브에서 MP3를 고르는 운영체제 파일 선택 창과 앱 아이콘을 추가했습니다. v0.3.0의 로컬 MP3 반복·음악/효과음 개별 음량, 원본120레벨·2D HD와 Web 점진적 다운로드/진행률을 유지합니다.

## MP3 선택

- Windows/Linux: **F4 또는 Ctrl+O**. macOS: **Cmd+O**도 지원합니다. 파일 선택 창에서 MP3 한 곡을 고르세요. 대소문자 확장자와 공백/유니코드 경로를 지원합니다.
- 선택 중 게임과 소리는 잠시 멈추며, 돌아오면 대화상자에 머문 시간을 게임 시간에 더하지 않습니다. 기존 Pause 상태와 전체화면을 복원하고 선택 창에 쓴 키/클릭을 게임에 적용하지 않습니다.
- 취소하면 기존 음악/모드/음량을 유지합니다. 파일이 손상됐거나 읽을 수 없으면 창 제목에서 오류를 안내합니다. F7 재시도·F6 원본 음악 전환을 사용할 수 있습니다.
- 기존 게임 창 파일 드롭과 `OhMyLemmings.exe --bgm "C:\Music\song.mp3"`도 가능합니다. F6 원본/MP3/끄기, F5 해제·원본 복귀, `[`/`]` 음악·`,`/`.` 효과음 음량, S 전체 소리·PageUp/PageDown 전체 음량은 같습니다.
- Linux 파일 선택은 libdbus와 XDG Desktop Portal/파일 선택 backend 또는 zenity가 필요합니다. 대화상자를 열 수 없는 환경에서는 드롭/`--bgm`을 이용하세요.
- Web은 기존 **MP3 선택** 버튼을 사용합니다. 사용자 음원은 업로드하거나 배포본에 포함하지 않습니다.

## 아이콘·배포

직접 작성한 SVG를 바탕으로 Windows EXE의16~256픽셀 내장 아이콘과 Cargo 버전 정보, macOS 앱의16~1024픽셀 ICNS, SDL 창 아이콘·Linux PNG를 제공합니다. 외부 폰트/원본 DOS 픽셀을 아이콘에 사용하지 않았으며 프로젝트 MIT를 따릅니다. 실행 시 별도 아이콘 파일을 고를 필요가 없습니다.

Windows11 x64 ZIP은 전부 풀고 OhMyLemmings.exe, macOS universal2는 앱 전체를 Applications에 복사해 실행합니다. Linux는 압축을 풀고 SDL2 runtime이 설치된 환경에서 `./lemmings-desktop`으로 실행합니다. Web은 [공개 게임](https://edwardkim.github.io/ohmylemmings/) 또는 ZIP 전체를 HTTP로 제공합니다. 자세한 기존 실행/음악 안내는 [v0.3.0 노트](https://github.com/edwardkim/ohmylemmings/blob/v0.3.0/mydocs/releases/issue-205_v0.3.0.md)를 참고하세요.

패키지에는 전체 게임/HD와 대응 소스·체크섬·LICENSE/THIRD_PARTY_NOTICES를 포함합니다. 파일/태그/CI/소스 계보는 동봉 RELEASE-MANIFEST.json을 따릅니다. rfd는 MIT, embed-resource는 MIT이며 Native에만 파일 선택/빌드 자원을 연결합니다. OPL LGPL·Symphonia MPL 고지를 유지합니다.

Windows10은 지원하지 않습니다. macOS13+ 앱은 ad-hoc 서명/미공증이며 차단 시 앱을 열어 본 뒤 시스템 설정 > 개인정보 보호 및 보안의 «그래도 열기»를 사용합니다. #113 성능/60fps NOT READY와 #202 새 MP3 사람 청취 응답은 별도로 유지합니다. 게시된 과거 버전 태그·asset·버전 노트를 덮어쓰지 않습니다.
