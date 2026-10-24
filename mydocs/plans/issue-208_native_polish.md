# #208 Native MP3 선택·앱 아이콘 / v0.3.1

이슈: https://github.com/edwardkim/ohmylemmings/issues/208

상태: 구현·플랫폼 CI7개·병합·v0.3.1 릴리즈/Pages를 완료했다. [최종 결과](../reports/issue-208_native_polish.md). 아래는 수행한 계획이다.

메인테이너가 Native MP3 선택 대화상자와 실행파일 아이콘을 요청하고 버전0.3.1을 확정했다. 기준 main `0a7ddaca3675d575a2ffb67ed261fa7e202c5431`. [기존 MP3 기능](../reports/issue-202_local_mp3.md)과 [릴리즈 정책](issue-124_release_policy.md)을 따른다.

1. F4/Ctrl+O(macOS Cmd+O 포함)로 부모 게임 창에 연결한 MP3 단일 파일 선택 창을 연다. rfd0.17.2의 Windows Common Item Dialog·macOS NSOpenPanel·Linux XDG portal을 사용한다. Linux는 libdbus/portal 또는 zenity가 필요하다. 취소 시 음악 선택을 유지하며 대화상자 동안 시뮬레이션/장치 출력을 멈추고 경과시간과 오래된 입력을 재생하지 않는다. 기존 Pause/결과/전체화면·드롭/CLI를 보존한다.
2. 직접 작성한 SVG를 단일 원본으로 PNG/ICO/ICNS/SDL RGBA를 만든다. Windows PE의 RT_GROUP_ICON/RT_ICON을 확인하고 Mac 앱 Resources/Info.plist 및 SDL 창에 연결한다. SVG/재생성 절차·해시를 기록하고 새 빌드/패키지가 아이콘 없이 성공하지 않게 한다.
3. TDD로 단축키/취소·시간·상태 경계와 손상/누락 아이콘/패키지를 확인한다. 실제 Native 선택/취소/음악·전체화면/아이콘과 기존 게임/웹·4플랫폼 CI7개를 검증한다. 의존성 고지/소스/과거 측정 기록은 보존한다.
4. workspace/lock9개 crate를0.3.1로 맞추고 새 노트·문서/일지를 작성한다. self-review·검증·병합 후0.3.1 패키지를 준비하며 기존 공개0.1.0/0.2.0/0.3.0 태그·asset·노트는 덮어쓰지 않는다. 사용자19222/8080/기존 미리보기와 Music 원본을 보존한다.

근거: [rfd API](https://github.com/PolyMeilex/rfd/blob/0.17.2/src/file_dialog.rs)·[고정 의존성/플랫폼](https://github.com/PolyMeilex/rfd/blob/0.17.2/Cargo.toml)·[SDL 창 아이콘](https://docs.rs/sdl2/0.38.0/sdl2/video/struct.Window.html#method.set_icon). no_std 코어·시뮬레이션/원본 OPL PCM은 변경하지 않는다. #202 사람 청취 응답과 #113 NOT READY는 별도다.
