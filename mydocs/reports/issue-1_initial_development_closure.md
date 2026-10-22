# #1 초기 구현·플레이 배포 종료

이슈: https://github.com/edwardkim/ohmylemmings/issues/1

2026-10-22 메인테이너가 “1번 이슈도 클로즈 처리를 해야겠습니다.”라고 지시했다. 초기 구현과 첫 플레이 배포의 상위 추적을 종료하고 남은 성능·구간 메모리는 기존 [#113](https://github.com/edwardkim/ohmylemmings/issues/113)에서 독립적으로 계속한다.

| 범위 | 종료 시점의 상태·근거 |
|---|---|
| G00~G10 | 원본/자원·시뮬레이션·직업·SDL/Web 플레이·사운드 구현과 단계별 검증 완료. [전체 계획](../plans/issue-1_implementation_plan.md) |
| G11 기능·호환성 | 120개 초기 재생·성공/실패 입력·COMP 재감사와 Native 표시 보정 증거 유지. [#104](issue-104_comp_reaudit.md), [#107](issue-107_render_cadence.md) |
| G11 남은 성능·메모리 | 더 좋은 호스트의 최종 검증을 #113에 이관. 기존 마감 초과0/1/0과 미충족 판정 보존 |
| G12 첫 플레이 배포 | [4플랫폼 v0.1.0](https://github.com/edwardkim/ohmylemmings/releases/tag/v0.1.0)·대응 소스/해시와 [Pages 플레이](https://edwardkim.github.io/ohmylemmings/) 제공. [#115](issue-115_release_preparation.md), [#124](issue-124_release_policy.md), [#126](issue-126_github_pages.md) |
| 라이선스·사용자 인수 | 자체 코드 MIT/제3자 고지 [#129](issue-129_project_license.md), 메인테이너 Windows 11·웹브라우저 성공 판정과 Windows 10 지원 제외 [#130](issue-130_maintainer_acceptance.md) |

Windows 지원 대상은 Windows 11 x64이다. macOS의 ad-hoc/미공증은 안내된 현재 배포 조건으로 유지한다. 메인테이너 인수 보고에 없는 개별 검사·환경/버전·패키지 해시를 추가로 통과 처리하지 않는다.

#1은 메인테이너 지시에 따라 초기 개발의 상위 추적을 완료 처리한다. G12는 현재 지원 범위의 첫 플레이 배포 완료로 정리하며, G11 전체 성능 합격 체크는 미완료로 남겨 #113을 연결한다. #1 종료 후에도 #113은 OPEN을 유지한다.

README·전체 계획/진행 기록·문서 색인·[작업일지](../orders/20261022.md)와 관련 최신 상태를 동기화했다. 과거 계획/검증과 일지는 각 시점의 기록으로 보존했다. 변경 Markdown13개의 상대 파일 링크454개·git diff --check PASS. self-review에서 종료 범위와 근거, #113 OPEN 유지·G11 성능 미완료, Windows 10 지원 제외가 일치하는지 확인했다. PR 실제 병합 후 #1을 종료하고 정확한 병합 결과는 GitHub #1의 최종 기록에 연결한다. 이번 변경은 문서/추적 정리이며 전체 플랫폼 빌드·게임/성능 실행은 반복하지 않는다.
