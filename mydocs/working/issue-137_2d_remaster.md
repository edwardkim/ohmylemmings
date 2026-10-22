# #137 2D HD 리마스터 진행

이슈: https://github.com/edwardkim/ohmylemmings/issues/137

메인테이너의 리마스터 등록/처리 지시를 받아 #137 OPEN·번호/URL을 확인했다. 초기 개발 #1은 CLOSED, 성능/메모리 #113은 독립 OPEN으로 유지한다. 기준 main001671c에서 별도 worktree/branch로 시작했다.

첫 절편 #138 OPEN·번호/URL 확인 후 실제 Fun 01을 bounded100틱으로 캡처하고 지형 배치·캐릭터 크기를 확인했다. 원본 캡처·trace는 기존 ignored assets/local에 보존했다. 이 실행은 성능 측정이나 새로운 전체 해법 검증이 아니다.

built-in image_gen으로 부드러운 손그림풍2D의1672×941 시안을 제작하고 프로젝트 assets/remaster에 복사했다. 프롬프트·출처/해시·원본 대응 목록과 [HD 자산/표시 계약](../plans/issue-138_hd_art_contract.md)을 작성했다. 시안은 디자인 후보이며 분리된 런타임 자산·실제 HD 게임 모드는 아직 없다. 사용자 스타일 응답은 후속 제작에 우선 반영한다.

원본28애니메이션337프레임·5세트273지형/54객체·540프레임/미리보기54개·특수4장면·UI19묶음200개를 제작 범위로 기록했다. 모든 런타임 HD 항목은 not_started로 유지했다. 검증된 원본 데이터나 규칙을 그림으로 대체하지 않는다.

HD-01은 PR #139/merge5a543b3로 병합하고 #138을 완료했다. 메인테이너 다음 진행 승인으로 [HD-02 / #140](issue-140_hd_presentation.md)을 등록·확인하고 typed HD snapshot·좌표/입력 계약과 합성16개 테스트를 구현했다. 정확한 로컬/CI·병합은 [#140 결과](../reports/issue-140_hd_presentation.md)를 따른다. 실제 HD 자산/표시·한 레벨/전체 확장·지원 플랫폼/패키지는 [상위 계획](../plans/issue-137_2d_remaster.md)의 후속이다.
