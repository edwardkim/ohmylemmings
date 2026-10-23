# #172 특수 4장면 HD

이슈: https://github.com/edwardkim/ohmylemmings/issues/172 · 상위 #137

#169의 선택 로딩 위에 특수4장면 그림과 typed 자산을 연결한다. 원본960×160의 x=304 배치, 지형 mask·굴착·철·입력·틱은 보존한다. 원본 자료를 확인하고 새 그림/프롬프트/출처를 기록한 뒤 parser/renderer/catalog 경계 TDD, Native/Web 실제 출력과 상태 동일성, 감사·CI·실제 병합으로 검증한다. BGM #165는 이후이며 전체 리마스터 완료로 확대하지 않는다.

최종 완료: PR #173 head `f96a5c78d9ee5afccb4324be59e6b2c11a31ba2e` / CI39928593396의7개 SUCCESS (첫 실행6개, 재실행 Rust native1개), 실제 merge `d8e6dacc554157ba3fcd91d9297d1c6b7242f1b0`와 검토 head 전체 tree 일치를 확인했다. [결과·재현·한계](../reports/issue-172_hd_special.md)를 따른다. 위 미완료/진행 중 기록은 당시 상태이며, 최종 검사·실제 병합을 완료했다.
