# G11-05c Tricky01~30 완주 검증

이슈: [#98](https://github.com/edwardkim/ohmylemmings/issues/98) · 상위 [#93](https://github.com/edwardkim/ohmylemmings/issues/93) · [결과·출처·재현](../reports/issue-98_tricky_solutions.md).

사용자 승인 후 main `1ac6ddc57cb32688d9a1c44aba83e5fe06391194`와 이슈 번호·URL을 확인했다. Tricky30개 성공/실패60입력을 확보했다. 성공20개는 자체 작성,10개는 출처를 밝힌 공개 리플레이의 기능 기록 변환이다. 해법 확보와 최종 검증 통과 수를 분리해 보고했다.

실제 SDL/WASM 각3회·결과 뒤 실제 Restart·고정 Lemmix56함수 대조를 진행한다. 검토 입력과 출처를 고정한 후 자체 검토·필수 CI·PR 실제 병합·main CI와 상위 이슈 갱신을 마무리한다. 최종 사례별 판정·해시·게시 기록은 #98에 남긴다. 제품·검증 도구 코드는 변경하지 않는다.

원시는 ignored assets/local/issue-98-review, 공개 입력은 assets/manifests/solution_replay_tricky.json이다. 최종 통과 후 Fun30개와 합쳐60/120이며 Taxing/Mayhem60개·COMP 전체·성능/G11/G12는 #93/#1 후속이다.
