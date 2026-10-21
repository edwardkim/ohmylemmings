# G11-05b Fun08~30 완주 검증

이슈: [#96](https://github.com/edwardkim/ohmylemmings/issues/96) · 상위 [#93](https://github.com/edwardkim/ohmylemmings/issues/93) · [결과·재현·한계](../reports/issue-96_fun_solutions.md).

기준 main `e51739a139f0fd31b644a0d2c44893647c43fc07`. 이슈 번호·URL 확인 후23개 전부의 성공/실패46입력을 확보했다. 마지막 Fun25는60/40, Fun29는77/48을 구출했다. 해법 탐색 시간을 과소평가했으며 이후 해법 확보와 호스트 검증 완료 수를 분리해 보고했다.

- 실제 SDL/WASM 각3회·성공/실패 뒤 실제 Restart와 고정 Lemmix 함수 대조를 병행한다. 실패한 긴 브라우저 실행은 보존하고 검증 후 heartbeat를 추가해 미완료 사례를 새 폴더에서 재실행한다.
- Python heartbeat TDD2개 RED→GREEN과 기존4개 통과. 전체 검사·self-review·정확한 게시 CI 후 실제 PR을 병합하고 main CI/상위 이슈를 갱신한다. 최종 판정과 해시는 #96에 남긴다.
- 원본/실행 원시는 ignored assets/local/issue-96-review, 공개 입력은 assets/manifests/solution_replay_fun.json이다. 제품 Rust 코드는 변경하지 않는다.
- 최종 판정 전 완료 수를 늘리지 않는다. 통과 후 기존7개와 합쳐 Fun30/120이며 나머지90개·COMP 전체·성능/G11/G12는 #93/#1 후속이다.
