# #208 Native MP3 선택 대화상자·앱 아이콘 / v0.3.1

이슈: https://github.com/edwardkim/ohmylemmings/issues/208

상태: 구현·플랫폼 CI7개·실제 Native UI·병합·v0.3.1 정식 릴리즈/Pages 게시 완료. 최종 고정 계보와 공개 검증은 아래에 기록한다. [계획](../plans/issue-208_native_polish.md) · [v0.3.1 실행 안내](../releases/issue-208_v0.3.1.md). 기준 main `0a7ddaca3675d575a2ffb67ed261fa7e202c5431`.

## 파일 선택 동작

Native에서 F4/Ctrl+O(macOS Cmd+O 포함)로 운영체제 MP3 단일 파일 선택 창을 연다. 대소문자 확장자·공백/유니코드 경로를 지원한다. 기존 드롭/--bgm과 같은 AudioService 선택 경로를 사용하며 취소 시 음악/모드/음량을 바꾸지 않는다. 창 제목에 F4 선택 안내를 표시한다. Alt+F4·일반 O·키 반복/Shift 조합은 선택 창을 열지 않는다.

선택 창이 열리는 동안 SDL 장치 큐를 정지/비우고 Stream/선택을 유지한다. 호스트의 동기 대화상자 처리 동안 게임 틱은 진행하지 않으며, 복귀 시 호스트 시계/표시 deadline을 재설정한다. 세션의 Pause나 시뮬레이션 시계는 변경하지 않는다. 포인터/누른 패널을 해제하고 대화상자의 오래된 키/클릭을 버린다. 창 닫기/표시 사건은 유지한다. 전체화면을 잠시 해제했다가 기존 상태로 복원한다. 결과/음소거·손상 MP3/재시도 등 기존 음악 정책은 유지한다.

rfd0.17.2를 정확히 고정했다. Windows Common Item Dialog·macOS NSOpenPanel·Linux XDG portal/zenity를 사용하며 SDL raw-window-handle로 부모 창을 전달한다. Linux에서는 libdbus와 파일 선택 portal backend 또는 zenity가 필요하다. 선택한 음원은 업로드/번들링/영구 복사하지 않는다.

![실제 Native MP3 선택 대화상자, MP3 필터·선택/취소](issue-208_native_dialog.png)

격리한 Xvfb/D-Bus의 GTK portal에서 캡처했다. 원본 DOS/사용자 음악을 포함하지 않은 합성 게임 테스트다. 일반 데스크톱의 창 장식은 해당 운영체제/창 관리자에 따른다.

## 실행파일·앱 아이콘

![자체 SVG 기반 앱 아이콘](../../apps/desktop/resources/app-icon.png)

원본 DOS 픽셀/다운로드 이미지/폰트 없이 작성한 SVG와 [재생성 안내](../../apps/desktop/resources/README.md)를 제공한다. 프로젝트 MIT이며 [해시/출처](../../assets/manifests/app_icon.json)에 고정했다.

- Windows: embed-resource3.0.12로 EXE에7크기(16/24/32/48/64/128/256) ICO와 Cargo의 FileVersion/ProductVersion을 내장한다. 리소스 컴파일 실패는 빌드를 실패시킨다. 패키징 도구가 실제 PE의 RT_GROUP_ICON101/RT_ICON을 읽고 전부 원본 PNG와 비교한다.
- macOS: 앱 Resources에 ICNS16~1024를 넣고 Info.plist CFBundleIconFile을 연결한다. 서명 이전에 스테이징하고 패키지/서명 검증에 포함한다.
- SDL: 64×64 RGBA를 실행파일에 내장해 작업 디렉터리와 관계없이 창 아이콘을 설정한다. Linux 패키지에는256픽셀 PNG도 제공한다. 시스템 아이콘 설치/사용자 데스크톱 설정 변경은 하지 않는다.

## 검증과 범위

- TDD: 선택 단축키가 없는 상태에서 실패를 확인한 뒤2개 키 경계 검사를 통과했다. 새 modal 오디오 probe 항목을 요구하는 검사도 먼저 실패하고 구현 후 통과했다.
- Rust 전체726개, Python325개 통과. fmt/clippy·기존 원본 OPL/C/음악/게임 사운드와 COMP/G11/과거 성능 자료 감사도 통과했다. 초기 의존성 개수 고정값30은 새 검토 목록74로 갱신했다.
- 실제 격리 Native UI: F4/Ctrl+O 열기, 선택/취소, 한글·공백 이름/.MP3 선택과 반복, 선택 창2초 동안 표시 캡처 동결, 선택 후 취소 시 음악 유지, 전체화면 복귀·F5 원본 전환을 확인했다. X11 _NET_WM_ICON의64×64 ARGB도 내장 RGBA와 전 픽셀 일치했다.
- 실제 SDL dummy stereo probe는 선택 창 동안 queue pause/clear·선택/모드 보존·복귀 후 재생을 확인했다. 같은 probe를 각 플랫폼의 압축 해제한 패키지 실행 파일에서도 실행한다.
- 실제 LLVM Windows 리소스 컴파일러/PE 링커로 아이콘을 내장한 독립 합성 PE를 만들고7개 RT_ICON 바이트를 비교했다. 손상/누락/잘못된 크기·경계/중복/변조 아이콘과 패키지 입력을 거부하는 오프라인 검사를 추가했다. 플랫폼 CI에는 실제 Windows Shell 아이콘 추출·FileVersion/ProductVersion 일치와 macOS iconutil ICNS 디코딩을 추가했다. 실제 게임 Windows EXE/Mac 앱 판정은 플랫폼 CI 결과를 따른다.

no_std 코어·게임 규칙·기본 OPL/PCM 합성은 변경하지 않았다. 외부 registry74개/고지136개를 기록하며 기존61개 고지/번들 원문을 보존했다. objc2 계열 crate에 누락된 LICENSE.md는 각 archive의 cargo_vcs_info 커밋에 고정한 공식 저장소 원문을 보관했다. 기능 소스 인덱스만 갱신하고 과거 producer/측정값은 유지했다.

사용자19222/8080·기존 미리보기와 Music 원본, 게시된0.1.0/0.2.0/0.3.0 태그·asset·버전 노트를 보존한다. #202 사람 청취 응답과 #113 NOT READY를 별도로 유지한다.

## 최종 CI·병합·v0.3.1 배포

[정식/latest 다운로드](https://github.com/edwardkim/ohmylemmings/releases/tag/v0.3.1) · [공개 Web](https://edwardkim.github.io/ohmylemmings/) · [deployment.json](https://edwardkim.github.io/ohmylemmings/deployment.json).

| 항목 | 고정 값 |
|---|---|
| 구현 PR | [#209](https://github.com/edwardkim/ohmylemmings/pull/209), self-review·실제 merge 완료 |
| 최종 head | `3769a4a9a6d834df7353c31fc6a990b42b1c111a` |
| 실제 CI checkout | `a882732b6987dae01c7602e722d5bb5a902e1484` |
| main/tag commit | `8828e94f82d730e78a763bd0fbe53159d69910ae` |
| 공통 source tree | `e55538ac619dfe05139e9834487e0b12b78cd594` |
| annotated tag object | `7900819580cfe3e4b1323f293583f7e757416689` |
| CI7개 | [40067006448](https://github.com/edwardkim/ohmylemmings/actions/runs/40067006448), 모두 SUCCESS |
| 태그 전 prepare-only | [40068383688](https://github.com/edwardkim/ohmylemmings/actions/runs/40068383688), SUCCESS·게시/Pages SKIPPED |
| 정식 게시/Pages | [40068722434](https://github.com/edwardkim/ohmylemmings/actions/runs/40068722434), publish/Pages 모두 SUCCESS |

CI Web의 실제 checkout log와 로컬 merge ref/main tree를 대조했다. Windows 전체 패키지·macOS ARM64/Intel·Linux·Web·WASM·Python7개를 전부 통과했다. Windows Shell 아이콘 추출/FileVersion·ProductVersion0.3.1과 macOS 두 아키텍처의 iconutil ICNS 디코딩도 성공했다. 배포 노트 링크를 독립 PLAY/Release에서 열리는 절대 링크로 바로잡은 최종 head3769a4a를 사용하며 이전/중복 취소 실행을 성공으로 집계하지 않았다.

전체 Native 회귀(8직업/상호작용/객체/승패/패널/포인터·오디오·표시 스모크)와 WASM 로컬 검증을 완료했다. 메인테이너 제공 MP3 두 파일도0.3.1 실행파일의 실제 SDL stereo/장치 소비·모달 pause/복귀 probe를 통과했고 원본 내용은 유지했다. 실제 사람 청취 응답은 #202에서 별도다.

4개 아카이브·각 체크섬·공통 소스·PLAY.md·RELEASE-MANIFEST.json·SHA256SUMS의12파일을 검증했다. 각 플랫폼은1231개 대응 소스를 포함하며 사전 검증/게시/공개 manifest와 아카이브 digest가 같다.

| 플랫폼 | 공개 아카이브 SHA-256 |
|---|---|
| linux-x86_64 | `74485c5a42dbde47228f451fb4071fb4f1525b42e5abc75f420da1b64437b4a7` |
| web | `b6658a9a56f8ff47b9df2cbdcbb62a193b27f1390e0ed775050823bfc90c9ea2` |
| windows-x86_64 | `68dcafea2d0f97dd19ad8baa9647c6dbbbc07dab070977267d9e0be41179cf23` |
| macos-universal2 | `1cba22765858bf654a2749b7e8ffa1f963bbcbf71691eff1fd70cbb222f3c076` |

공개 [RELEASE-MANIFEST.json](https://github.com/edwardkim/ohmylemmings/releases/download/v0.3.1/RELEASE-MANIFEST.json) · [SHA256SUMS](https://github.com/edwardkim/ohmylemmings/releases/download/v0.3.1/SHA256SUMS).

draft의12파일을 재다운로드해 일치를 확인한 후 공개했고, 게시 도구가12개 공개 URL도 인증 없이 전부 검증했다. 별도 검사에서도 공개 PLAY/MF/체크섬·Web ZIP의 모든 내부 pin과 WASM/대응 소스 쌍, 배포된 HTML/JS/WASM/설정18파일·WASM MIME을 확인했다. Pages는 재빌드 없이 동일 Web ZIP109파일을 승격했다.

실제 공개 HTTPS0.3.1에서 기본 HD1920×1080·95글꼴·현재99캐릭터 프레임/전체337카탈로그·진행률·8필수 페이지19,751,567바이트·원본 전환을 확인했다. Fun01 664틱/665상태·23표시에서 원본과 상태가 같고10구출 Won이었다. 원본 WebAudio context running·4,096제출/4,036비영 PCM·예외0을 확인했다. Web MP3 기능 코드는0.3.0과 동일하며 기존 검증을 유지하고 전체 플랫폼 CI를 통과했다.

v0.1.0/0.2.0/0.3.0의 tag object/target·각12asset ID/크기/digest/게시·수정 시각·릴리즈 본문을 전후 대조해 불변을 확인했다. 기존/새0.3.1 버전 노트도 게시 후 수정하지 않는다. 로컬 main/Native/Web을0.3.1로 동기화하고 실제 공개 ZIP의 WASM/대응 소스를 사용한다. 사용자19222/8080·preview-172/165/196/202와 Music 원본을 보존했다. 개인 검증 자료는 ignored assets/local/에만 유지한다.

#208 구현·게시 요구를 완료한다. #202 사람 청취 응답 OPEN과 #113 NOT READY는 별도다.

후속 최종 인수(2026-10-24): 메인테이너가 #202에 대해 “확인은 했습니다.”라고 사용 확인을 전달했다. [#202 최종 인수](issue-202_local_mp3.md#메인테이너-최종-인수-완료-2026-10-24)를 완료했으며, 위 #202 OPEN/청취 대기는 배포 당시 기록이다. #208과 #202의 기능·배포/사용 인수를 모두 완료했다. #113 성능 NOT READY는 유지한다.
