# G09-02 자산 바이트 공급·공통 장면과 패널

이슈: [#64](https://github.com/edwardkim/ohmylemmings/issues/64) · 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1) · 선행 [#62](issue-62_shared_session.md)

기준 main은 `eb442214d7edaca99bf49948037414a112fe4790`입니다. `lemmings-game`에 기존 SDL 없는 Assets·CPU render/View·panel·pointer 표현·LevelRun을 옮깁니다. desktop은 파일 읽기·경로 오류·SDL 이벤트/표시·시스템 시간·캡처/재생을 담당합니다. game→resources/session/core, resources→core, session→core 경계이며 core/resources의 no_std와 session 구현은 바뀌지 않습니다.

## 바이트 공급 계약

- `AssetSource::read(&mut self, name: &str) -> Result<Option<Vec<u8>>, Box<dyn Error>>`는 대소문자를 유지한 논리 이름을 사용합니다. 같은 서명의 FnMut도 구현체입니다. 동기 조립이며 웹은 비동기로 확보한 바이트를 먼저 보유한 뒤 공급할 수 있습니다.
- `Assets::from_source(source, number)`는 기존 Fun1..30 표준 그래픽 지원 범위에서 카탈로그→LEVEL→필요한 ODDTABLE→GROUND/VGAGR→MAIN→선택 EXE 순서로 필요한 이름만 요청합니다. 유효하지 않은 번호는 읽기 전에 거절합니다. 특수 그래픽은 이전과 같이 미지원입니다.
- `Ok(None)`은 부재입니다. 필수 자산이면 공개 `MissingAsset { name }` 오류가 됩니다. desktop은 실제 root와 결합한 NotFound 오류를 만듭니다. 공급자의 다른 오류는 원래 형식을 보존해 전파하며, 읽기·경로·URL·캐시는 호스트의 책임입니다.
- 선택적 `VGALEMMI.EXE`의 부재만 허용합니다. 읽기 실패와 지원하지 않는 길이/MZ 프로필은 오류이며 파티클 표의 기존 오프셋·파싱은 유지합니다. 원본 자산 진위 확인은 기존 acquisition/verify 도구의 책임입니다.

## 표현·전환 계약

`View::frame`은 동일한 320×200 RGB를 합성하며 Session을 변경하지 않습니다. 카메라·선택·패널 누름/포인터·결과 오류·장면 캐시는 View가 소유합니다. panel은 논리 좌표 버튼·미니맵·누름 반복과 화면 합성을, pointer는 core 선택에 대한 좌표/캡션 변환을 담당합니다. 실제 키·SDL 좌표·포커스 이벤트는 desktop에서 변환합니다.

`LevelRun::new`는 자산에서 공유 Session을 구성합니다. `prepare`는 새 자산·세션·View를 모두 만든 후 교체 대상으로 돌려주며 실패하면 기존 결과·세션·화면이 유지됩니다. 재시작은 새 바이트를 다시 조립할 수 있습니다. 기존 desktop 모듈은 공유 타입 재수출 경로로 남겨 입력·재생의 타입과 경계를 보존합니다.

## 확인한 근거와 후속

render/panel/lifecycle의 본문은 모듈 설명 외에 바이트 단위로 같고 pointer의 SDL 없는 본문도 같습니다. 자산 조립 본문은 공급 인자·호스트 중립 오류 문구 외에 같으며 지형/객체/합성 fixture 본문도 같습니다. 플랫폼 호출이 공유 src에 없고 session/core/resources는 바뀌지 않았습니다. 세부 검증은 [보고서](../reports/issue-64_shared_game.md)와 [해시 명세](../../assets/manifests/shared_game.json)를 따릅니다.

다음 G09-03은 JS/WASM 바인딩과 실제 합성 브라우저 진입입니다. Canvas·입력·원본 공급과 동일 레벨 브라우저 완주는 G09-04/05에서 이어집니다. 사용자가 제공한 Chromium CDP 포트19222를 후속 브라우저 검증에 사용합니다. G09 전체는 진행 중입니다.
