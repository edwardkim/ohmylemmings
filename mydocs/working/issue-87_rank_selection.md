# G11-02 네 난이도 레벨 선택·전환

이슈: [#87](https://github.com/edwardkim/ohmylemmings/issues/87) · 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1) · [검증 계획](../plans/issue-85_compatibility_validation.md).

기준 main `ae8ab7cddbb1aa02c6fe93b426b5ba6d9a2f2b30`. 이슈 번호·URL 확인 후 `issue-87-rank-selection`에서 착수했다.

## 범위

기존 `LevelId`를 공통 로더·SDL·Web 선택/재시작/Continue·입력 기록에 연결한다. Fun 기본 CLI/API와 이전 재생 입력을 유지한다. 난이도 끝의 원작 규칙을 고정 소스로 확인하고, 무효 ID·1/30 경계·음악 cursor/세대·자원 교체 실패 보존을 실패하는 테스트부터 검증한다.

특수4장면은 G11-03에서 연결한다. 이번 네 난이도 진입 증거는 전체120개 완주 인증이 아니다. 원본/캡처/개별 로그는 ignored `assets/local/issue-87-review/`에 둔다. HTTP8080·CDP19222 loopback과 무관한 미추적 파일을 유지한다.

구현과 TDD를 완료했다. [계약](../tech_insights/issue-87_rank_selection.md)과 [보고서](../reports/issue-87_rank_selection.md)에 범위·원작 관측 한계를 구분했다. 로컬 Rust603/문서29/Python164·format/clippy·WASM, 실제 브라우저35사례/12,564상태/7,225Canvas·제품 DOM, SDL 원본8진입/648상태/80readback과 창작 경계3개를 통과했다. 원본50파일은 불변이다.

다음: 자체 검토와 한글 PR의 게시/병합 CI를 확인하고 #87/#1을 동기화한다. 전체 SDL 합성·웹 완주/오디오 회귀는 게시 CI에서 확인한다. HTTP 미리보기를 새 UI로 갱신했다. 다음 구현은 G11-03 특수4장면 연결이다.
