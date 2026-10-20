# G09-04 브라우저 입력·원본 자산·레벨 전환

이슈: [#68](https://github.com/edwardkim/ohmylemmings/issues/68) · 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1)

기준: [#56 패널](issue-56_skill_panel.md), [#58 포인터](issue-58_pointer_selection.md), [#64 공유 게임](issue-64_shared_game.md), [#66 WASM](issue-66_wasm_browser.md).

계약: 논리 320×200 좌표, 네이티브 키/패널/미니맵 및 우클릭 선택 규칙을 공유 함수로 실행합니다. 틱마다 panel.advance_input과 pointer.refresh를 적용합니다. 원본은 flat 논리 파일명·크기·중복을 검증한 불변 바이트 묶음으로 공급하고 파싱은 Assets::from_source에 위임합니다. 새 LevelRun 구성 성공 후 교체하며 시계·입력·뷰를 초기화합니다. Fun1..30의 기존 표준 그래픽 범위이며 특수 배경은 후속입니다.


## 실제 입력과 경계

키는 물리 code를 받고, 화면에 포커스가 있을 때만 처리합니다. 폼·IME·수정키 입력을 가로채지 않습니다. Space/N/결과 Enter/R의 반복을 억제하고 같은 경계의 의도한 Pause/RR을 사용합니다. Esc는 페이지 안의 중단으로 처리하고 복귀 버튼에서 시간 기준을 새로 잡습니다. 파일을 읽는 동안과 중단 상태에서는 게임 입력을 받지 않습니다.

Canvas의 실제 CSS 사각형으로320×200 좌표를 구하며 외부 좌표를 내부로 clamp하지 않습니다. Pointer capture로 외부 mouse-up을 받고 취소/포커스/숨김 이탈은 누름·우클릭·선택 기억을 지웁니다. 자동 capture 해제는 hover를 유지하면서 버튼만 해제하고 더블클릭 기록을 보존합니다. 전체 폭파 더블클릭은500ms·논리4픽셀 이내의 두 번 누름입니다.

마우스의 두 번째 버튼은 pointerdown 대신 pointermove의 buttons 변경으로 전달됩니다. 이전 buttons와의 차이에서 down/up을 복원해 우클릭 중 왼쪽 클릭을 공유 PointerAssignment로 전달합니다. 일반 hover/드래그의 button=-1에서는 새 클릭을 만들지 않습니다. DOM 테스트와 CDP의 실제 trusted 이벤트에서 이 차이를 확인합니다.

틱마다 공유 panel.advance_input → pointer.refresh → Session.tick → pointer.refresh 순서로 실행합니다. 렌더 빈도는 held-rate 반복에 영향을 주지 않습니다. 원본/합성 모두 같은 CPU View·패널·미니맵과 소유 RGB 복사본을 Canvas에 표시합니다. core/resources/session/game의 규칙 소스는 변경하지 않았습니다. SDL 입력과 WebGame을 함께 구동하는 테스트는 desktop의 dev-dependency만 사용합니다. 새 registry 패키지나 버전 변경은 없습니다.

## 파일과 장면 교체

WebAssets는 허용23개 논리 파일명을 ASCII 대문자로 정규화하고 파일당1MiB·중복·경로를 검사합니다. Rc의 copy-on-write 바이트 묶음으로 로드 시점의 불변 스냅샷을 보존합니다. WebGame은 현재 source와 pacing을 보유하고 Assets::from_source/LevelRun으로 완전히 조립한 뒤 교체합니다. 원본 로드는 DOS pacing이고, 다음 Fun 레벨과 재시작은 같은 묶음을 사용합니다. 실패한 일반 로드는 이전 세션/뷰/시계 전체를 보존합니다. 결과 Continue 실패는 네이티브처럼 기존 결과를 유지하며 오류 표시만 바꿉니다.

페이지는 선택한 File.arrayBuffer만 사용합니다. 서버 경로, 원격 fetch, 외부 업로드, 원본 묶음 배포 기능은 없습니다. 관련 없는 선택 파일은 건너뛰고 필요한 파일 누락·손상·중복을 복구 가능한 오류로 표시합니다. 원본에서 연습 모드로 돌아갈 수 있도록 장면 선택을 비웁니다. 실패/다음/마지막 레벨은 LevelRun.target/prepare 계약을 사용합니다.

## 검증 구분

- 오프라인 Rust: 새 바이트/입력/전환5개와 SDL 대조1개. 직접 native Assets/LevelRun 및 SDL 입력과 상태·입력·지형/브릭/객체·프레임/선택을 비교합니다.
- 실제 WASM: 기본24사례/48회·7742전체 상태·4456RGB/Canvas.8개 기술 포인터·패널·누름 반복·카메라·포커스와 합성 성공/실패/전환, 실제 File 객체를 검사합니다.
- 선택 원본: 고정 manifest의 모든 로컬 파일을 크기/SHA-256 검증한 뒤 Fun01/02/03 초기150틱·기술/카메라/재시작을 추가합니다. 총27사례/54회·8648상태·4978RGB/Canvas. 증거 출력은 assets/local 아래만 허용합니다.
- CDP19222: 동일27사례와 실제 trusted 키/포인터/마우스 chord·더블클릭·외부 해제, 실제 파일 선택/원본 화면/복구와 결과 전환을 별도 확인합니다.

표준 배경 지원 범위는 기존 Fun29개이며 Fun22는 명확한 오류입니다. 원본 한 레벨의 브라우저 성공/실패 통합 완주, 시도별 네이티브 전체 추적은 G09-05에서 진행합니다. 새 DOS 실행,120레벨 인증, 음향이나 배포를 이번 결과로 집계하지 않습니다.
