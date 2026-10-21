# G11-05e Mayhem01~30 완주 검증

이슈: [#102](https://github.com/edwardkim/ohmylemmings/issues/102) · 상위 [#93](https://github.com/edwardkim/ohmylemmings/issues/93) · 선행 [#100](../reports/issue-100_taxing_solutions.md).

사용자의 다음 절차 승인 후 이슈 번호·URL과 main `1a6419cf2d7a1527995ecc08b7ecdb8f2a50258f`의 원격 동기화를 확인했다. `issue-102-mayhem-solutions`에서 Mayhem30개의 원본 목표 성공/실패60입력을 확보한다. 자체 작성과 공개 리플레이 변환은 출처·해시로 구분한다.

[저장 해법 계약](../tech_insights/issue-94_solution_replay.md)에 따라 실제 SDL/WASM 각3회·전체 상태/맵/화면·실제 결과 뒤 Restart·고정 Lemmix56함수/표와 기존 종료 adapter를 대조한다. 해법 확보와 최종 검증 통과를 구분한다. 결함은 별도 이슈 최소 재현/TDD와 영향 회귀 후 진행한다.

원시는 ignored `assets/local/issue-102-review/`에 보존한다. 해법 확보 단계에서 성공 입력30/30개를 고정했다.22번은79/68명,25번은72/72명,28번은1/1명을 실제 구출했다. 출처와 재현은 [입력 보고서](../reports/issue-102_mayhem_solutions.md)에 기록했다.

초기 묶음01~24의 정식 SDL/WASM·고정 기준 대조를 통과한 뒤 마지막25~30을 별도 실행했다. 브라우저에는28번의 원본 WASM 성공 입력을 표시했으며, 표시용 영상과 정식 증거 실행을 구분했다.

최종60사례의 실행·자체 검토·정확한 push/PR/main CI·실제 병합·소스/미리보기 검증·상위 갱신·마감 판정은 [#102 최종 기록](https://github.com/edwardkim/ohmylemmings/issues/102)에 사례별 해시와 함께 남긴다. Mayhem30개 해법 묶음 이후에도 COMP 전체 재감사·성능/G11/G12는 #93/#1 후속이다.
