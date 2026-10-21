# G11-05 유효 해법 완주·COMP 재감사

이슈: [#93](https://github.com/edwardkim/ohmylemmings/issues/93) · 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1) · [계획](../plans/issue-85_compatibility_validation.md).

기준 main `6190c7b8be1d6a6e09e90d76b201a5eeda768eaa`. #91의120개 초기340경계/3회/양 플랫폼 통과 이후 착수했다. 이 증거는 전수 해법 완주를 뜻하지 않는다.

## 진행 상태

- 2026-10-21: 이슈 번호·URL과 main 동기화를 확인했다. 기존 유효 완주 입력은 Fun01만 확인했다.
- Fun부터 난이도별30개 묶음으로 진행한다. 해법 출처·고정 입력·실제 목표 달성을 확인하고 실패·재시작·SDL/WASM 각3회 재생을 대조한다.
- 실제 재고/목표/원본 지형을 유지한다. 성공 주입·수량 변경은 해법으로 계산하지 않는다.
- 결함은 별도 이슈로 재현·TDD 수정 후 회귀한다. 미확보 해법/미검증 COMP와 DOS 관측 한계를 공개한다.

## 현재 판정

G11-05 실행 묶음120개는 pending이다. G11/G12는 진행 중이며 성능 검증은 후속이다. 원시 원본·파생 실행 자료는 ignored `assets/local/issue-93-review/`에 둔다.

## 첫 실행 절편

[#94 Fun01~07](issue-94_solution_replay.md)은8직업을 모두 소개하는 튜토리얼7개의 호스트 검증이다. 이후 Fun08~30, 나머지3난이도와 COMP 전체를 계속한다. #94 완료만으로 #93을 닫지 않는다.

## #102 병합 후 현재 집계 / #104 재감사

Fun30·Tricky30·Taxing30·Mayhem30 = **120/120개 유효 성공 입력과 실패 입력**의 실제 SDL/WASM 각3회·Restart 검증을 마쳤다. #94/#96/#98/#100/#102의 최종 증거와 정확한 CI는 각 이슈를 따른다. 위의 미완료 수치는 과거 착수 시점 기록이다. 현재 제품 기준은 main `64dc383`이며 [#104](https://github.com/edwardkim/ohmylemmings/issues/104)에서 COMP-01~18의 세부 요구·현재 소스·테스트·실제 기준 함수·호스트 증거를 재감사한다.

일부 이전 PC의 ignored 원시 자료가 현재 PC에 없어 실제 기준 함수로 경계를 보충한다. [새 보고서](../reports/issue-104_comp_reaudit.md)와 별도 감사 manifest에 해시와 제한을 고정한다. #85의 과거 계획 manifest는 유지한다. G11-05의 최종 마감은 #104의 self-review·실제 CI·병합 후 처리하며 성능은 G11-06 후속이다.
