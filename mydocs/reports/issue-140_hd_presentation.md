# #140 공통 HD 표시 snapshot·좌표/입력 구현

이슈: https://github.com/edwardkim/ohmylemmings/issues/140 · 상위 https://github.com/edwardkim/ohmylemmings/issues/137

[#138의 표시 계약](../plans/issue-138_hd_art_contract.md)을 `lemmings_game::hd`의 typed Rust API로 구현했다. Native와 WASM Rust에서 공유하며 I/O·창·GPU·PNG 의존을 추가하지 않았다. 기존 앱은 원본 모드를 계속 사용하고 새 API는 아직 화면에 연결하지 않았다.

## 구현

- `Viewport::fit`은1920×1080의 비율 유지/출력 여백을 계산한다. 포인터와 출력 크기는 같은 단위로 받고 비정상 크기·NaN/Infinity·여백·반열린 끝점을 제외한다. 게임은320×160으로 변환하며 패널은 별도8직업·생성속도·Pause/Nuke·재시작·미니맵 대상을 반환한다.
- 그리기/클릭 영역은 `control_regions`를 공유한다. 기존 패널 Button으로 변환할 수 있지만 클릭 계약 자체는 입력을 큐에 넣거나 held rate/Nuke 상태 머신을 실행하지 않는다. 호스트는 기존 재생/완료/포인터 처리 규칙을 적용해야 한다.
- 미니맵 내용은1664,976,248,88이며 테두리는 입력에서 제외한다. 논리 카메라는 맵 끝으로 clamp하고 화면 배율을 두 번 적용하지 않는다. 프레임 기준점·좌우 반전·trim·음수 좌표는 별도 `sprite_rect` 계산으로 제공한다.
- `snapshot(&LevelRun)`은 기존 session의 논리 크기·completion rules를 읽는다. terrain의 실제 충돌/표시 지연 mask·표시 brick·재질/일방통행·trap은 복사 없이 빌린다. actor는 제거된 입자 보유 기록까지 전체 상태/frame/방향과 HD 발 위치를 복사한다.
- HUD는 실제 quota·Nuke 분모·인원/직업 수량·시간·대기 중 pause/rate·선택/완료·다음 레벨/재도전 가능 여부를 제공한다. 읽기 전용 호출은 세션·뷰의 카메라 clamp 값·입력·사운드를 변경하지 않는다. session에는 논리 크기/완료 규칙의 읽기 전용 getter2개만 추가했다.

## TDD·동일성

공개 API 부재로 새 테스트가 실패한 뒤 구현했다. 분수 크기의 실제 버튼 시작점을 테스트하면서 역변환 반올림이 y=960 경계를 앞쪽으로 보내는 실패를 확인했다. 출력 좌표에서 각 경계를 직접 비교하도록 고쳐16개 모두 통과했다. `usize::MAX` 맵의 끝점 테스트는 해당 맵 크기의 실제 비례 거리를 반영하도록 수정했고 별도 로그에 보존했다.

경계/resize·125/150/200% 상당 좌표·여백·모든 버튼/미니맵·잘못된 값·발/반전·i32 음수 극값·mask 길이/메타데이터 불일치·대기 입력·실제 폭발의 표시 mask 지연·제거된 입자·완료/재시작을 확인했다. 같은 Builder/Nuke 입력180틱에서 HD snapshot을 매번3회 읽은 경우와 읽지 않은 경우의 논리 상태/terrain·brick·원본 RGB/사운드가 일치했다.

`make check-python`의253개·기존 COMP18/70/120레벨/240성공실패 증거·라이선스/표시 마감 감사 PASS. source index는438파일로 갱신하고 #140 이전 pin을 별도 보존했다. 기존380행 성능 관측과 NOT READY·#113의0/1/0 미충족 기록은 변경하지 않았다. 현재 검사와 과거 전수 플레이/성능 관측의 범위를 구분한다.

정확한 워크스페이스·WASM·실제 CI/PR·병합 결과는 아래 완료 기록과 GitHub #140에 연결한다. 원본 renderer·시뮬레이션 알고리즘·PCM·플랫폼 입력 경로는 수정하지 않았다.

로컬 최종 검사: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked --offline -- -D warnings`, `cargo test --workspace --locked --offline`의Rust637개/doc29개와 `cargo build --workspace --lib --target wasm32-unknown-unknown --locked --offline` 모두 PASS. 새16개 통합 테스트는 이637개에 포함한다. WASM은 라이브러리 컴파일 검사이며 새 HD JS API의 실제 브라우저 실행 검사는 아니다.

self-review에서 기존 경로의 변경이 lib export·Session getter2개에 한정되고 코어·자원 파서·기존 renderer/PCM·앱 입력 코드가 불변인지 대조했다. 변경 Markdown10개의 상대 파일 링크246개·git diff --check PASS. 정확한 CI/병합은 GitHub #140 최종 기록을 따른다.

## 후속

이 단계는 읽기 전용 표시 자료와 입력 대상 계산의 구현이다. 새 JS wire/export·실제 Native/Web HD 렌더링/입력·PNG/atlas 검증·GPU 합성과 HD 자산은 #137의 다음 절편이다. 이 변경으로 새 HD 플레이 screenshot이나 성능 합격을 선언하지 않는다. 기존 v0.1.0/Pages·#1 CLOSED와 #113 OPEN/저사양 장비의 긴60fps 반복 중단을 유지한다.
