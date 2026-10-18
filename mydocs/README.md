# Hyper-Waterfall 문서 안내

프로젝트 기획은 [rusty_lemmings.md](plans/rusty_lemmings.md), 기여 지침은 [AGENTS.md](../AGENTS.md)를 기준으로 합니다.

단계별 구현 목표와 작업 순서는 [전체 구현 계획](plans/implementation_plan.md)을 참고합니다.

G00-01의 [DOS 기준판 식별·원본 확보](tech_insights/dos_original_baseline.md)를 완료했습니다. 로컬 데이터 준비는 [리소스 안내](../assets/README.md)를 참고합니다.

[G00-02 레퍼런스 코드 지도](tech_insights/reference_code_map.md)의 고정 소스와 선언 위치를 검증했고 결과 게시를 진행 중입니다.

## 폴더 구조

- [plans/](plans/README.md): 마일스톤 및 아키텍처 계획
- [orders/](orders/README.md): AI 작업 지시 템플릿 및 모듈별 요구사항
- [working/](working/README.md): 진행 중 작업 상태와 이어서 작업할 컨텍스트
- [reports/](reports/README.md): 작업 결과와 검증 보고서
- [feedbacks/](feedbacks/README.md): 사용자 및 리뷰어의 피드백과 반영 결과
- [tech_insights/](tech_insights/README.md): 레퍼런스 분석과 확인된 기술 지식
- [troubleshootings/](troubleshootings/README.md): 문제 원인과 해결 이력

## 작업 흐름

1. GitHub 이슈를 먼저 등록하고 생성된 번호·URL을 조회해 확인합니다.
2. 이슈를 연결한 `orders/` 작업 지시와 `working/` 진행 기록을 작성하고 관련 계획·피드백·기술 분석 문서를 읽습니다.
3. 작은 모듈부터 TDD로 구현하고 검증하며, 진행 상황과 다음 작업을 `working/`에 기록합니다.
4. 완료된 작업의 변경 내용과 검증 결과를 `reports/`에 정리합니다.
5. 발견한 지식, 피드백 반영 결과, 문제 해결 이력을 기록하고 계획의 진행 상태를 갱신한 뒤 이슈를 완료 처리합니다.

문서에서는 예정된 작업과 구현·검증이 완료된 내용을 구분합니다.
