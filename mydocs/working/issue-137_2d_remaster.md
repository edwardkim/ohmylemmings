# #137 2D HD 리마스터 진행

이슈: https://github.com/edwardkim/ohmylemmings/issues/137

메인테이너의 리마스터 등록/처리 지시를 받아 #137 OPEN·번호/URL을 확인했다. 초기 개발 #1은 CLOSED, 성능/메모리 #113은 독립 OPEN으로 유지한다. 기준 main001671c에서 별도 worktree/branch로 시작했다.

첫 절편 #138 OPEN·번호/URL 확인 후 실제 Fun 01을 bounded100틱으로 캡처하고 지형 배치·캐릭터 크기를 확인했다. 원본 캡처·trace는 기존 ignored assets/local에 보존했다. 이 실행은 성능 측정이나 새로운 전체 해법 검증이 아니다.

built-in image_gen으로 부드러운 손그림풍2D의1672×941 시안을 제작하고 프로젝트 assets/remaster에 복사했다. 프롬프트·출처/해시·원본 대응 목록과 [HD 자산/표시 계약](../plans/issue-138_hd_art_contract.md)을 작성했다. 시안은 디자인 후보이며 분리된 런타임 자산·실제 HD 게임 모드는 아직 없다. 사용자 스타일 응답은 후속 제작에 우선 반영한다.

원본28애니메이션337프레임·5세트273지형/54객체·540프레임/미리보기54개·특수4장면·UI19묶음200개를 제작 범위로 기록했다. 모든 런타임 HD 항목은 not_started로 유지했다. 검증된 원본 데이터나 규칙을 그림으로 대체하지 않는다.

HD-01은 PR #139/merge5a543b3로 병합하고 #138을 완료했다. 메인테이너 다음 진행 승인으로 [HD-02 / #140](issue-140_hd_presentation.md)을 등록·확인하고 typed HD snapshot·좌표/입력 계약과 합성16개 테스트를 구현했다. 정확한 로컬/CI·병합은 [#140 결과](../reports/issue-140_hd_presentation.md)를 따른다. 실제 HD 자산/표시·한 레벨/전체 확장·지원 플랫폼/패키지는 [상위 계획](../plans/issue-137_2d_remaster.md)의 후속이다.

HD-02 #140은 PR #141 실제 병합·동일 tree와 최종 CI7개 SUCCESS로 완료했다. Web180초 대기 실패는 같은 head의 해당 job 재실행으로 통과했고 Native15분 취소/설치 지연·실패 이력은 #140 보고서에 보존했다. 다음은 HD-03 분리 캐릭터/애니메이션·지형·시설/UI 자산 제작/로딩·검증이다. 실제 HD 플레이 화면·전체 자산은 미구현이므로 #137은 OPEN을 유지한다.

사용자 다음 단계 승인으로 #144 HD-03a를 등록·확인하고 걷기 좌우16프레임·암석 분리 PNG와 RGBA 팩 로더/검증을 구현한다. 실제 HD 게임과 다른 동작/시설/UI는 후속으로 유지한다. [진행](issue-144_hd_assets.md), [결과](../reports/issue-144_hd_assets.md).

HD-03a #144의 걷기16프레임·암석 후보와 공통 팩/검증은 PR #145 실제 병합·동일 tree/CI7개 SUCCESS로 완료했다. 통합된 HD 애니메이션은 여전히0개이며, 다음은 입출구·굴착/계단/폭발·UI 제작 후 실제 Fun01 HD 통합이다. #137은 OPEN으로 유지한다.

HD-03b [#147](https://github.com/edwardkim/ohmylemmings/issues/147)의 직업65프레임·첫 테마 입출구11프레임·13조작 아이콘과 typed 팩 확장을 PR #149/merge `653762ff7383d87a5963fc2bc39c43d25983fdd2`로 실제 병합했다. 최종 head `3ff29f8df90f8bcad4c31eff84d9c11829f7bd25`/CI39866059128의7개 검사 SUCCESS·전체 tree 일치, 로컬 실제7atlas106자산/81캐릭터 프레임·브라우저 미리보기·기존 팩 호환성을 확인했다. [결과](../reports/issue-147_hd_actions.md)를 참고한다. 다음은 HD-04 실제 Fun01 Native/Web 합성·원본 모드 선택/입력 통합이며, 동작 크기/위상·출구 doorway/원본 trigger와 누락 상태 처리도 이 단계에서 검토한다. 전체 HD 게임/GPU·나머지21애니메이션256프레임/다른 테마·시설/효과·전체 UI·배포는 미완료, #137은 OPEN이다. #113 성능 이월/기존 NOT READY·#142 Chrome156 진단·#1 CLOSED/공개 v0.1.0/Pages를 유지한다.

HD-04 [#151](../reports/issue-151_hd_integration.md)의 실제 Native/Web HD 합성·원본 전환/전체화면·입력과 Fun01 동일성을 PR #152/merge `3147601e06b12b67289fc5c105ed19163bf09203`로 실제 병합했다. 최종 head `efadab1f241061f102d2a96cc0cb190fa1f4c920`/CI39876930568 최종 attempt2의7개 SUCCESS와 전체 tree 일치. 실제7애니메이션81프레임 통합/부분 runtime_ready=true이며 full_remaster_ready=false다. 다음 HD-05/06·#137 OPEN, #113 이월/NOT READY·#142 진단과 공개 v0.1.0/Pages 유지.
