# #172 특수 4장면 HD

이슈: https://github.com/edwardkim/ohmylemmings/issues/172 · 상위 #137

#169의 선택 로딩 위에 특수4장면 그림과 typed 자산을 연결한다. 원본960×160의 x=304 배치, 지형 mask·굴착·철·입력·틱은 보존한다. 원본 자료를 확인하고 새 그림/프롬프트/출처를 기록한 뒤 parser/renderer/catalog 경계 TDD, Native/Web 실제 출력과 상태 동일성, 감사·CI·실제 병합으로 검증한다. BGM #165는 이후이며 전체 리마스터 완료로 확대하지 않는다.

사용자 시연 요청: 기존8080의 `/preview-172/`에 isolated ignored Web 작업본을 복사했다. Fun22·HD·시작 전 일시정지로 열리도록 해당 복사본만 설정했다. 원본 main 소스/기존 루트 Web은 유지한다. HTTP bundle/catalog/forest page 해시 검증 완료. 사용자 브라우저 연결은 CUA inventory에 없어 자동 탭 조작은 하지 못했다. 전체 구현 검증과 병합은 계속 미완료다.

사용자 CDP19222 연결 후 기존 빈 탭에서 `/preview-172/`를 열었다. 실제 UI의 Fun22/HD/1920canvas/일시정지/오류 없음과 화면을 확인했다. 계속하기 버튼으로 사용자 플레이 가능. 사용자 탭을 테스트용으로 전환하거나 종료하지 않는다. 현재 별도 worktree 변경은 미커밋이며, 4PNG/생성 프롬프트와 해시·확장 manifest·typed special key·world-aligned masked renderer·TDD를 구현했다. Rust/Clippy/Python 및 Native120레벨 검사 결과는 assets/local/issue-172-review에 보존한다. 이후 독립 QA 브라우저로 Web120장면 비교, 4장면 시각 검토, 소스/evidence pin 갱신, 문서/self-review/CI/실제 병합이 남았다. 사용자가 보는 preview 복사본은 임의로 바꾸지 않는다.

최종 완료: PR #173 head `f96a5c78d9ee5afccb4324be59e6b2c11a31ba2e` / CI39928593396의7개 SUCCESS (첫 실행6개, 재실행 Rust native1개), 실제 merge `d8e6dacc554157ba3fcd91d9297d1c6b7242f1b0`와 검토 head 전체 tree 일치를 확인했다. [결과·재현·한계](../reports/issue-172_hd_special.md)를 따른다. 위 미완료/진행 중 기록은 당시 상태이며, 최종 검사·실제 병합을 완료했다.
