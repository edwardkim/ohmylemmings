# G09-05 브라우저 통합 완주 계약

이슈: [#70](https://github.com/edwardkim/ohmylemmings/issues/70) · 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1)

[G09 경계](issue-62_shared_session.md), [네이티브 완주](issue-60_native_playthrough.md), [브라우저 입력](issue-68_web_input_assets.md)을 따릅니다. 같은 공통 core/session/game을 사용하며 시뮬레이션 규칙·자산 해석·의존 패키지는 바꾸지 않았습니다.

## 관측과 제어 경계

제품 `app.js`의 선택적 `observe(listener)`는 attach/reset/input/before/after/present를 알립니다. 등록하지 않은 실행은 기존 배치 step을 사용합니다. 등록한 FramePump는 같은 예산 안에서 step(1)마다 관측하며 완료 결과에서 멈추고 남은 시간 빚 규칙을 유지합니다. detach 이후에는 다시 배치 실행합니다. 관측자는 로컬 개발 API이며 화면에 추가 메뉴를 만들거나 파일을 업로드하지 않습니다.

시도는 성공적으로 새 게임을 조립한 reset에서 시작합니다. 새 상태 관측 뒤 입력 해제를 기록하므로 이전 실행의 누름이 새 실행에 섞이지 않습니다. 같은 WebGame 내부가 교체되기 전에 완료 상태·맵·입력 로그를 캐시합니다. 초기 pristine 상태와 입력 큐가 들어간 제어 경계0은 구분합니다. 각 행은 해당 경계 입력을 모두 적용한 뒤, 다음 tick 직전 상태입니다. 마지막 완료 경계는 after에서 한 번 기록하고 이후 진행하지 않습니다. 마지막 Fun02 진행 중 시도는 중단 후 stop에서 남은 경계를 기록합니다.

`state()` 버전1에는 Nuke의 is_nuked/assigning/next_index/max_lemmings를 추가했습니다. 기존 세션·모든 actor 필드·8개 기술·출구/위험/완료·입력 진단은 유지합니다. `input_log()`는 제어 step/동일 경계 순번/명령을 데스크톱 독립 재생 형식으로 내보냅니다. 포인터 명령의 주/차선 ID·우클릭 상태도 직렬화합니다.

## 기록 형식과 검증

`JourneyRecorder`의 version1 기록은 original 플래그와 네 시도를 가집니다. 레벨은 1/1/1/2, 원인은 load(합성 synthetic)/restart/continue/continue입니다. 각 시도는 다음을 보존합니다.

- initial: 입력 전 전체 state/view와 terrain/bricks/objects 원시 맵의 base64.
- rows: 빠짐없는 control_step0..N의 전체 state와 각 맵의 정렬된 [index,old,new] 바이트 변형. 변화가 없는 경계도 보존합니다.
- events: 경계 순서대로 Key/Pointer/Release/직접 command와 Frame 표식. 같은 경계의 순서도 유지합니다.
- frames: 표식 번호·경계·전체 state/view·논리320×200 RGB와 제품 Canvas의 실제 불투명 RGBA.
- final: 마지막 state/view·원시 맵·순번이 있는 안정 ID 명령 로그. 최종 맵은 모든 변형을 누적한 바이트와 같아야 합니다.

검증기는 경계 누락/중복/역순, 초기 인구·시계/입력 기억 누출, 새 ID0 누락, 부분 구출·잘못된 결과/전환, 정렬/이전 바이트/범위/물질 오류, 잘못된 최종 맵·Frame 표식·Canvas 픽셀/alpha를 거절합니다. 의미 검증에 이어 실제 네이티브 실행을 요구하므로 자기 일관성만으로 통과하지 않습니다.

네이티브 `web_trace`는 raw 이벤트를 같은 경계와 순서로 재생합니다. Frame 표식은 그 위치에서 별도 캡처 레코드를 내보내며, 경계의 모든 이벤트 후 상태 행과 구분합니다. 모든 진단 필드와 각 경계의 세 맵 SHA-256, 초기/최종 맵의 실제 바이트, 캡처 state/view/RGB를 비교합니다. Canvas는 RGB와 모든 alpha255를 독립 비교합니다.

추가로 데스크톱 SDL 실행은 input_log의 안정 ID 명령만 받아 각 시도를 render_every1/7/13에서 별도로 재생합니다. 기존 collect 정규화로 모든 Session/actor·기술·Nuke·구출/위험/판정 필드, 각 경계 terrain 삭제/추가·객체 변형과 최종 세 맵을 대조합니다. SDL readback도 논리 RGB와 비교합니다. 검증 직후 도구가 만든 frame-*.rgb/ppm만 지우고 파일 크기/해시 색인과 나머지 trace/맵/설정/입력은 보존합니다.

같은 raw 입력을 WASM에서 60/120/144Hz 시간 분할과 render1/7/13 조합으로 다시 실행합니다. 모든 제어 경계 전체 state와 모든 맵 바이트, 선택된 캡처 RGB를 비교합니다. 낮은 tick 예산3으로 시간 빚도 소비합니다. 이 재생 시간은 통제한 값이며 실제 제품 rAF 조작 증거와 따로 집계합니다.

## 통합 시나리오와 증거 경계

원본 없는 CI는 실제 제품 HTML/DOM·GameInput·WASM을 사용하고 rAF 시각만 제어합니다. 합성 성공→R 재시작→Digger/단일 Nuke 거절/더블 Nuke 실패→결과 클릭 재도전→전원 구출→Enter 다음 레벨→새 ID0→Escape를 실행합니다. Pause/RR 누름 반복·미니맵·카메라·결과 고정도 포함합니다. 기존24사례의 잘못된 숫자·손상/누락 파일·전환 실패 보존·숨김/복귀·시간 분할 회귀와 함께 `make check-web`에서 실행합니다. SDL 독립 재생 때문에 Web CI도 SDL2 개발 헤더를 준비합니다.

사용자 Chromium CDP19222에서는 실제 rAF와 trusted 키/마우스·로컬23파일 선택으로 원본 Fun01을 두 번 10/10 구출하고 중간 조기 Nuke 실패(0/3)를 재현했습니다. Pause/RR·미니맵 끌기·CSS 크기 변경/외부 클릭·숨김/복귀·결과 고정, 다음 Fun02 새 ID0도 확인했습니다. 증거 수치는 [보고서](../reports/issue-70_web_playthrough.md)에 분리합니다.

원본은 [고정 출처](../../assets/manifests/dos_original.json)의 크기/SHA-256을 재생 전후 검증합니다. 원본 파생 기록/맵/픽셀/화면은 ignored assets/local 아래만 출력할 수 있습니다. 공개 PNG는 직접 만든 합성 구출 결과입니다. 기존 #60의 DOS 성공3/실패1·기준 함수56개와 화면 대조는 기존 증거이며 이번 새 브라우저 실행으로 재집계하지 않습니다. 전체120레벨/DOS 인증·특수 배경·음향·배포는 후속 범위입니다.
