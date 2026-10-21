# G11-05d Taxing01~30 완주 검증

이슈: [#100](https://github.com/edwardkim/ohmylemmings/issues/100) · 상위 [#93](https://github.com/edwardkim/ohmylemmings/issues/93) · 선행 [#98](../reports/issue-98_tricky_solutions.md).

사용자 다음 절차 승인 후 이슈 번호·URL과 main `1d4f977f5c8a028f67164249cbcc1473cf6d807a`의 원격 동기화를 확인했다. `issue-100-taxing-solutions`에서 Taxing30개의 원본 목표 성공/실패60입력을 확보한다. 자체 작성과 공개 리플레이 변환은 출처·해시로 구분한다.

[저장 해법 계약](../tech_insights/issue-94_solution_replay.md)에 따라 실제 SDL/WASM 각3회·전체 상태/맵/화면·실제 결과 뒤 Restart·고정 Lemmix56함수/표와 기존 종료 adapter를 대조한다. 해법 확보 수와 최종 검증 통과 수를 분리한다. 결함은 별도 이슈 최소 재현/TDD와 영향 회귀 후 진행한다.

원시는 ignored `assets/local/issue-100-review/`에 보존한다. 성공30개/실패30개 입력을 확보했다. 성공7개 자체 작성과23개 공개 리플레이 변환을 구분했으며, 해법 확보와 최종 실행/CI 통과는 별도다. [입력·출처·재현](../reports/issue-100_taxing_solutions.md)에 고정했다. 최종 호스트/기준 검증을 진행한다. 최종 입력·출처·문서 자체 검토, 정확한 push/PR/main CI와 실제 병합·소스/미리보기 검증·상위 갱신 후 #100을 닫는다. 완료하면 Fun/Tricky/Taxing90/120이며 Mayhem30개·COMP 전체·성능/G11/G12는 #93/#1 후속이다.
