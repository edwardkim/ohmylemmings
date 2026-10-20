# G09-02 자산 바이트·공통 장면 분리 검증

이슈: [#64](https://github.com/edwardkim/ohmylemmings/issues/64) · [구현 계약](../tech_insights/issue-64_shared_game.md) · [진행 기록](../working/issue-64_shared_game.md)

기준 main `eb442214d7edaca99bf49948037414a112fe4790`의 Assets·CPU render/View·panel·pointer 표현·LevelRun을 `lemmings-game`으로 추출했습니다. 이름→바이트 공급을 공개하고 파일/경로·SDL 입력/표시는 desktop에 남겼습니다. 시뮬레이션과 core/resources의 no_std 경계는 그대로입니다.

## 공개 계약과 회귀

미구현 공개 API 실패를 먼저 기록한 뒤 기존 순수 테스트24개를 공유 크레이트로 옮겼습니다. 새 공개 API 테스트7개는 자체 DAT/GROUND/VGAGR/MAIN 합성 바이트로 정상 장면·스킬·패널 조립, 필수 자산 누락/손상, ODDTABLE 적용, 잘못된 번호/특수 그래픽, 선택 EXE 프로필/파티클, 공급자 오류 전파, 실패한 재시작의 상태/화면 보존과 새 바이트 사용을 검사합니다. 파일 읽기와 SDL 통합 테스트27개는 desktop에 남겼습니다. 전체 Rust513개·문서28개·Python101개와 Clippy를 통과했습니다.

독립 target 디렉터리에서 `lemmings-game --lib --target wasm32-unknown-unknown --locked --offline`를 빌드했습니다. 정상 의존 그래프는 game→core/resources/session뿐이고 플랫폼 호출이 공유 src에 없습니다. render/panel/lifecycle 본문과 pointer 브리지, 자산 조립/장면 본문의 보존을 별도로 대조했습니다. session/core/resources src 변경은 없습니다.

## 추출 전후 네이티브 실행

main 실행 파일을 보존하고 합성 성공·실패·혼합·파티클 및 원본 Fun01 성공·실패6실행1912상태/413캡처를 새로 저장했습니다. 추출 후 렌더 간격1/7의12실행3824상태/826캡처에서 모든 정규화 상태·순서 있는 지형/객체 이력, 초기/최종10개씩의 bin과 모든 논리320×200 RGB가 같습니다. 실제 SDL dummy/software 캡처·재생 경로를 사용했습니다. 변경된 화면 동작은 없습니다.

## 메모리 공급과 파일 공급

원본 바이트를 한 번 메모리에 올린 BTreeMap 공급과 실제 desktop 파일 어댑터를 별도 호스트에서 비교했습니다. Fun1..30 중 표준 그래픽29개를 동일하게 조립했고 특수 그래픽 Fun22는 두 경로가 같은 오류로 거절했습니다. 각 레벨 초기 상태와 Fun01/02/03/08/30의380틱 입력 재생을 합쳐1929개 상태·지형/브릭·전체 논리 RGB 프레임이 같습니다. 그중144개 RGB와 추적/최종 지형을 보관했습니다. 이는 공급 경로 동등성 확인이며29레벨 완주나120레벨 DOS 호환성 판정은 아닙니다.

## 검토·증거·게시

자체 검토에서 새 외부 의존 없이 플랫폼 경계·오류/부재 구분·선택 EXE·새 세션 교체 순서·원본 없는 fixture·기존 입력 경로를 확인했습니다. [해시 명세](../../assets/manifests/shared_game.json)에 소스와 실행 증거의 출처/크기/SHA-256을 고정합니다. 원본 파생 바이트·추적·캡처·실행 파일은 ignored `assets/local/issue-64-*` 아래에만 보관합니다.

최종 `make check` 종료 결과, 검토 커밋/원격 트리, push·PR·main의 정확한 커밋 CI와 실제 전체 로그·병합/종료는 #64에 누적합니다. G09 전체는 진행 중이며 다음은 G09-03 JS/WASM 바인딩과 합성 브라우저 진입입니다. 후속 브라우저 연결은 CDP19222를 사용합니다.
