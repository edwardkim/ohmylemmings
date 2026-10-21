# G11-05a Fun01~07 해법 검증 결과 기록

이슈: [#94](https://github.com/edwardkim/ohmylemmings/issues/94) · 상위 [#93](https://github.com/edwardkim/ohmylemmings/issues/93) · [계약/재현](../tech_insights/issue-94_solution_replay.md) · [입력/소스 목록](../../assets/manifests/solution_replay.json).

## 코드 검토 시점

기준 main `6190c7b`에서 실제 목표를 충족하는 Fun01~07 저장 입력과 실패 입력을 작성했다. 탐색 결과와 반복 호스트 판정을 분리한다. 원본14사례의 SDL/WASM 각3회·실제 성공/실패 후 재시작 대조를 실행 중이며 **최종 결과·개별/종합 해시·CI·병합/종료 상태는 GitHub #94의 최종 기록**을 따른다. 공개 목록은 이 시점의 소스/입력·계약을 고정한다.

입력 순서/인수/범위·목표 변조/미완료 결과·초기 상태/맵 누락 검사의 TDD4개와 기존 초기 단위4개가 통과했다. Python176개와 motion fixture도 통과했다. 원본 없는 실제 SDL/WASM 성공·실패·재시작 smoke를 통과했으며 기존 초기5개/15회 재생도 회귀한다. 제품 Rust 물리/렌더/UI/음향·의존성은 변경하지 않았다. 화면 변경이 없어 새 스크린샷을 추가하지 않는다.

## 첫 절편과 한계

첫7개는 Digger/Floater/Blocker/Miner·Climber/Basher/Bomber/Builder를 모두 포함한다. Nuke 사용 성공도 원본 `required` 이상을 실제 구출한 뒤 종료한다. 실패는 목표 미달과 종료 사유를 남긴다. 실제 양 플랫폼 일치는 전체 DOS 물리 인증과 다르며 새 입력의 원작 비교는 미검증이다. 이전 Fun01 원작 관측은 기존 #60/#70 범위만 재사용한다.

G11-05 나머지113개·COMP 전체 마감은 #93에서 계속한다. G11/G12나 성능 검증을 완료 처리하지 않는다. SDL 재시작은 이전 입력 폐기/초기 복원과 입력 없는 prefix, WASM 재시작은 같은 입력의 실제 재완주라는 범위를 명시한다. 결과 원시 파일은 ignored assets/local/issue-93-review에 둔다.
