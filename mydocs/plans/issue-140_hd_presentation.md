# #140 공통 HD 표시·입력 계약 구현

이슈: https://github.com/edwardkim/ohmylemmings/issues/140 · 상위 [#137](issue-137_2d_remaster.md)

메인테이너의 다음 진행 승인으로 HD-02를 시작한다. 기준 main5a543b3과 이슈 OPEN·번호/URL을 확인했다. [#138 아트/표시 계약](issue-138_hd_art_contract.md)의1920×1080·게임1920×960/패널120을 typed Rust API로 구현한다.

## 구현과 검증

1. 공통 game crate에 플랫폼/I/O 없는 HD layout·표시 snapshot API를 추가한다. 내부 틱·충돌·원본 RGB renderer는 유지한다.
2. 출력 비율 유지/여백과 게임·8직업/생성속도/Pause/Nuke/재시작·미니맵 영역을 정의한다. 호스트는 포인터를 출력 표면의 왼쪽 위 기준으로 바꾸고 크기와 같은 단위로 제공한다. 브라우저는 client 좌표에서 bounding rect의 left/top을 빼며, CSS/물리 픽셀 또는 이미 논리 좌표로 바뀐 SDL 이벤트를 중복 변환하지 않는다.
3. 반열린 경계·resize/DPI·NaN/Infinity/잘못된 크기·카메라 끝과 음수 actor/발 기준점·좌우 반전을 테스트한다. 클릭 영역은 명령 대상만 반환하고 held rate/Nuke 상태 머신을 대체하지 않는다.
4. 검증된 LevelRun에서 읽기 전용 snapshot을 만든다. 논리 collision/표시 terrain·brick·재질·trap과 actor 상태/프레임/발 위치, HUD·선택/완료 정보를 제공한다. 큰 맵은 빌리며 복사하지 않는다.
5. 합성 레벨에서 반복 snapshot·pause/입력 경계·지형 변경·결과/재시작 전후를 확인한다. snapshot 유무가 같은 입력의 상태·마스크·원본 RGB/사운드를 바꾸지 않는지 대조한다.
6. 관련 Rust·format/lint·WASM/기존 증거 감사를 수행하고 소스 영향과 기존 미충족 판정 보존을 기록한다. 실제 CI·self-review·병합 결과를 연결한다.

## 한계와 후속

이번 단계는 Native/Web이 공유할 표시/입력 자료 계약이다. PNG 해제·GPU 합성·HD 자산·실제 HD 모드/전체화면·새 WASM JS wire 형식과 실제 앱 입력 연결은 후속이다. 픽셀/게임 플레이 screenshot은 이번 단계에서 새로 만들지 않는다. 원본320×200 모드·공개 v0.1.0/Pages·#1 CLOSED와 #113 OPEN/저사양60fps 반복 중단을 유지한다.
