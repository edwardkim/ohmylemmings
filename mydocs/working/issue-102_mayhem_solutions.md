# G11-05e Mayhem01~30 완주 검증

이슈: [#102](https://github.com/edwardkim/ohmylemmings/issues/102) · 상위 [#93](https://github.com/edwardkim/ohmylemmings/issues/93) · 선행 [#100](../reports/issue-100_taxing_solutions.md).

사용자의 다음 절차 승인 후 이슈 번호·URL과 main `1a6419cf2d7a1527995ecc08b7ecdb8f2a50258f`의 원격 동기화를 확인했다. `issue-102-mayhem-solutions`에서 Mayhem30개의 원본 목표 성공/실패60입력을 확보한다. 자체 작성과 공개 리플레이 변환은 출처·해시로 구분한다.

[저장 해법 계약](../tech_insights/issue-94_solution_replay.md)에 따라 실제 SDL/WASM 각3회·전체 상태/맵/화면·실제 결과 뒤 Restart·고정 Lemmix56함수/표와 기존 종료 adapter를 대조한다. 해법 확보와 최종 검증 통과를 구분한다. 결함은 별도 이슈 최소 재현/TDD와 영향 회귀 후 진행한다.

원시는 ignored `assets/local/issue-102-review/`에 보존한다. 현재 성공 입력30/30개를 확보했다.28번은 원본1명을 실제 출구로 구출했다.25번은 원본80명 중72명을 구출하여 목표72명을 충족했다.22번은 원본80명 중79명을 구출하여 목표68명을 충족했다. 실제 SDL/WASM·고정 기준 대조는24/30개가 통과했다.

- 완료: 01~24 전체24개.
- 남은 정식 검증: 25~30 전체6개는 성공 입력을 확보하여 정식 검증 중이다.
- 현재 브라우저에는 원본 자산을 읽은 실제 WASM의28번 성공 입력 재생을 표시한다. 화면 재생과 정식 증거 실행은 구분하며, 표시용 재생은 통과 수를 추가하지 않는다.

최종 실행·CI 판정은 pending이다. self-review·정확한 push/PR/main CI·실제 병합·소스/미리보기 검증·상위 갱신 후 #102를 닫는다. Mayhem30개 완료 후에도 COMP 전체 재감사·성능/G11/G12는 #93/#1 후속이다.
