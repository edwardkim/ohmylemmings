# G11-05a 튜토리얼7개 유효 해법 검증

이슈: [#94](https://github.com/edwardkim/ohmylemmings/issues/94) · 상위 [#93](https://github.com/edwardkim/ohmylemmings/issues/93) · [계약](../tech_insights/issue-94_solution_replay.md).

기준 main `6190c7b8be1d6a6e09e90d76b201a5eeda768eaa`. 이슈 번호·URL 확인 후 착수했다. Fun01~07은 8직업을 모두 소개하는 첫 절편이다. 원본 수량/재고/목표와 실제 입력을 유지했다.

- 성공7개·Nuke 실패7개의 저장 입력을 확보했다. 탐색용 프로세스의 결과는 호스트 반복 검증과 구분한다.
- 입력 순서/범위·명령/arity·목표 강제 변경·미완료/이유 없는 결과·재시작 필드/맵 변경을 거부하는 TDD4개, 기존 초기 재생 단위4개를 통과했다. 전체 Python176개 통과.
- 실제 SDL/WASM의 원본 없는 성공/실패·재시작 smoke 통과. 기존 초기 재생 smoke도 회귀한다.
- 원본14사례 각3회 SDL/실제 WASM과 성공/실패 후 실제 재시작 실행 중이다. self-review와 정확한 게시/병합 CI를 포함한 최종 판정은 #94에 기록한다.

G11-05 전체120개와 COMP 전체 마감은 #93에 남긴다. 이 절편의 미검증/한계는 [결과 기록](../reports/issue-94_solution_replay.md)을 따른다.
