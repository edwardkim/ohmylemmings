# 계획 문서

- [#140 HD 표시 snapshot·좌표/입력 구현](issue-140_hd_presentation.md).

- [#137 2D HD 리마스터](issue-137_2d_remaster.md), [#138 Fun 01 HD 아트/표시 계약](issue-138_hd_art_contract.md).

- [#85 G11 전체 호환성 점검·성능 측정 계획](issue-85_compatibility_validation.md)

마일스톤과 시스템 아키텍처를 관리합니다. 전체 기획서는 이 폴더의 [프로젝트 기획서 / #1](issue-1_rusty_lemmings.md)에 있습니다.

분할정복 기반 목표와 의존관계·완료 기준은 [전체 구현 계획](issue-1_implementation_plan.md)에서 관리합니다.

현재 Cargo 구성과 코어·리소스·플랫폼 경계는 [G01-01 Rust 개발 골격](issue-6_rust_workspace.md)을 참고합니다.

로컬과 CI가 사용하는 공통 명령·환경·완료 판정은 [G01-02 CI·개발 검증](issue-7_continuous_integration.md)을 따릅니다.

- 단계별 목표, 구현 범위, 모듈 간 의존성, 완료 기준을 기록합니다.
- [#115 배포 대상·검증 계약](issue-115_release_preparation.md): Linux/Windows/Web 산출물과 깨끗한 빌드·라이선스/원본 제외·성능 이월 조건.
- 설계 결정은 선택 이유와 관련 기술 분석 문서를 함께 남깁니다.
- 진행 상태는 실제 구현과 검증 결과에 맞춰 갱신합니다.

파일명은 `issue-<번호>_<주제>.md`로 작성하고 상단에 담당 이슈를 연결합니다. 장기 계획은 상위 추적 이슈를 사용합니다. 예: `issue-1_implementation_plan.md`.

- [#118 Windows 전체 게임 첫 실행](issue-118_windows_full_game.md): 원본 그래픽·120레벨·음향 동봉과 더블클릭 실행·실제 Windows 검증.

- [#120 macOS 전체 게임 앱](issue-120_macos_full_game.md): Apple Silicon/Intel universal2·원본 동봉·실제 Mac 검증과 서명/공증 조건.

- [#122 Linux·Web 전체 게임 다운로드](issue-122_linux_web_full_game.md)

- [#124 버전·태그 정책과 Releases 배포](issue-124_release_policy.md)

- [#126 GitHub Pages 플레이 배포](issue-126_github_pages.md)

- [#129 자체 코드 라이선스 적용안](issue-129_project_license.md): 자체 코드 정책 결정과 기존 제3자/원본 범위 보존.

- [#144 HD 자산·팩 계약](issue-144_hd_assets.md): HD-03a 걷기16프레임·암석과 공통 로더/검증.

- [#147 HD-03b 직업·시설·패널 자산](issue-147_hd_actions.md).

- [#151 HD-04 실제 Fun01 Native/Web 통합](issue-151_hd_integration.md).

- [#154 HD-05a 캐릭터 전체 프레임 계획](issue-154_hd_actors.md)

- [#157 HD-05b 일반 지형 재질 계획](issue-157_hd_terrain.md)

- [#160 HD-05c 전체 테마 입출구](issue-160_hd_portals.md)

- [#163 HD 자산 팩 정리 계획](issue-163_hd_pack.md)

- [#165 BGM 수정·청취 인수 완료](issue-165_bgm.md)

- [#169 테마별 HD 로딩](issue-169_hd_loading.md)

- [#172 특수4장면 HD](issue-172_hd_special.md)

- [#175 물·불 HD](issue-175_hd_hazards.md)

- [#178 기계 함정 HD](issue-178_hd_traps.md):발동/재사용·원본 판정 보존과 Native/Web 검증.

- [#181 깃발·일방통행 표시 HD](issue-181_hd_decor.md):방향 제한/마스크와 원본 규칙 보존.

- [#184 HD 글꼴·전체 게임 UI](issue-184_hd_ui.md)

- [#187 HD 패키징·최종 인수](issue-187_hd_release.md)

- [#191 리마스터 v0.2.0 버전 적용](issue-191_remaster_version.md)

- [#194 v0.2.0 릴리즈·Pages](issue-194_release_v020.md)

- [#196 웹 점진적 자산 다운로드](issue-196_web_progressive.md)

- [#200 사용자 K-pop BGM 교체 검토 계획](issue-200_custom_bgm_review.md)

- [#202 로컬 MP3 BGM 구현 계획](issue-202_local_mp3.md)

- [#205 v0.3.0 릴리즈·Pages 계획](issue-205_release_v030.md)
