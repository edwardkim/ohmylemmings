# G07-10 직업 전환·상호작용 진행 기록

이슈: [#46](https://github.com/edwardkim/ohmylemmings/issues/46), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 동기화와 착수

다른 개발 PC의 14커밋을 `git fetch origin`·fast-forward로 반영했습니다. 로컬 변경은 없었으며 main `2e3e478e6520a4603cce804346476723a40f2778`에서 PR #35~#45의 Builder·Blocker·Bomber·Climber·Floater·Nuke 병합과 #44 종료를 확인했습니다. main CI [39226742901](https://github.com/edwardkim/ohmylemmings/actions/runs/39226742901)은 성공입니다. 변경된 계획과 루트 지침을 읽고 #46 번호·URL·OPEN을 확인한 뒤 `issue-46-skill-interactions`에서 착수했습니다.

## 범위와 다음 행동

15개 지원 행동×8직업의 할당 전환, 수량0·영구 능력·기존 타이머·거부 시 보존을 고정 원문으로 확인합니다. Rust 합성 테스트와 실제 Pascal 실행으로 행렬을 대조하고, 다중 할당·변형 지형/Blocker·영구 능력/폭파/Nuke 혼합 재생을 렌더 주기1/7에서 검증합니다. 기존 원본 관측을 새 혼합 시나리오의 DOS 증거로 확대하지 않습니다.

실제 실패 테스트→필요한 수정→합성 golden/오프라인 CI→전체 검사→보고·자체 검토·PR/CI·병합/main CI 순서로 진행합니다. G07 전체 완료 여부는 검증 결과와 미지원 범위를 분리해 판단합니다. G08의 출구/위험물·구출/승패·원본 패널·음향은 별도입니다.

## 기준 실행 결과

복원 가능한2640할당과 혼합14사례28재생11788상태/1400캡처가 실제 원문 실행과 일치했습니다. 초기 세 시도의 Blocker 초기 필드/코드와 성공 입력 순서 오류는 fixture 문제로 수정했으며 최종4만 성공으로 계산합니다. 저장된 전 경계에 강화한 행동 도달 조건도 다시 적용했습니다. 엔진 수정은 필요하지 않았습니다.

Rust 기본 허용/수량0/제거 보존3테스트와 Python 누락·중복·순서·형식/해시·최초 차이4테스트를 추가했습니다. 기존50개 원문 함수/표와 추출 결과의 불변을 확인했습니다. [보고서](../reports/issue-46_skill_interactions.md)·[계약](../tech_insights/issue-46_skill_interactions.md)·[manifest](../../assets/manifests/skill_interactions.json)를 작성하고 기존 전체 검사/원격 CI를 확인합니다. G08의 COMP-14 포인터 선택/RightClickGlitch 및 객체 종료 행동은 이번 고정 ID 행렬과 구분합니다.

## 완료와 다음 단계

전체 `make check` 712.50초·Rust428/문서27/Python77·모델/모든 SDL·WASM을 통과했습니다. 원본50/고정 소스46개·문서 링크·21소스/401개별 산출물 해시를 확인했습니다. G07의8직업/Nuke/지형 및 전환/상호작용 회귀 기준을 충족했으며 다음은 G08-01 객체 트리거·출구/구출 판정입니다. 게시·정확한 HEAD CI·PR/병합·main CI·종료의 최종 근거는 [#46 완료 기록](https://github.com/edwardkim/ohmylemmings/issues/46)에서 추적합니다.
