# G05-03 개체 이동·제거 진행 기록

작업 이슈: [GitHub #25](https://github.com/edwardkim/ohmylemmings/issues/25). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 범위와 순서

번호·URL·OPEN 상태를 확인했습니다. [#5 비교 기준](../tech_insights/issue-5_compatibility_baseline.md), [#20 지형 조회](../tech_insights/issue-20_terrain_mask.md), [#24 생성 요청](../tech_insights/issue-24_startup_schedule.md), [#16 프레임/발 메타데이터](../tech_insights/issue-16_lemming_sprites.md)를 읽었습니다. 계약 → 독립 모델·합성 Red/Green → 전수/통합 재생 → 문서·검사·게시·실제 CI 순서입니다.

직업·객체/출구·강철·지형 변경·성패·리소스/플랫폼 연결·전체 trace/DOS 실행은 후속 범위입니다. G05 전체는 진행 중입니다.

## 현재 상태

기본 이동/제거·ID 순 배치·오류 보존의 구현과 전수/통합 재생·공통 검사를 통과했습니다. G05 전체는 상태 추적/기준 실행 대조가 남아 진행 중입니다.

## 수행과 근거

- [계약·독립 모델·재현 명령](../tech_insights/issue-25_actor_motion.md), [결과 보고서](../reports/issue-25_actor_motion.md).
- 새 19개 모두 Red·기존 코어 79개/리소스 181개 통과에서 Green 코어 98개를 확인했습니다.
- 322560전이의 1024그룹 golden·전체 서명 a04dc778204e5a15, 36사례/108재생/36720제어 호출의 전체 이벤트·상태와 서명 4821a442b91d9b8b가 독립 모델과 같습니다.
- make check: Rust 단위 279개·문서 테스트 27개(기존 compile-fail 2개 포함)·Python 8개·포맷·Clippy·빌드·WASM 성공.
- 세 문서 sh 블록을 그대로 실행했고 소스 46개·관련 원문 4개/Styles.Base의 크기/SHA-256/Git blob을 확인했습니다. 독립 모델/golden을 make check-python과 기존 CI에 연결했습니다.

- Markdown 90개·로컬 링크 629개·앵커 108개·이슈 문서명 78개·#25 일지와 변경 파일 16개의 범위·언어/공백을 확인했습니다. 코어 의존 0개·리소스→코어 로컬 의존 하나와 make check-python의 unittest 8개/독립 모델 golden 통과를 확인했습니다.

## 다음 행동

문서 재현을 확인했습니다. 추적 규칙·의존 경계를 확인했습니다. main 게시·실제 CI·원격 tree·상위 #1 갱신·#25 종료를 확인합니다. 다음 후보는 별도 이슈의 G05-04 상태 추적·재생·기준 실행 대조입니다.
