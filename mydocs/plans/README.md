# 계획 문서

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
