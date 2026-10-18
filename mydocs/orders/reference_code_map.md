# G00-02 레퍼런스 버전·파일·함수 고정

## 목표와 입력

작업 이슈: [GitHub #3](https://github.com/edwardkim/ohmylemmings/issues/3). 등록과 열린 상태를 조회해 확인했습니다.

상위 [이슈 #1](https://github.com/edwardkim/ohmylemmings/issues/1)의 G00-02 작업입니다. [전체 구현 계획](../plans/implementation_plan.md)과 [DOS 기준판 분석](../tech_insights/dos_original_baseline.md)을 바탕으로 후속 구현에서 읽을 소스 위치와 역할을 고정합니다.

## 범위

- G00-01에서 사용한 Lemmix·Lemmings.ts 커밋을 유지합니다.
- DAT 파싱, 레벨·그래픽, 틱·물리, 직업 상태, 렌더링·사운드의 진입점을 확인합니다.
- 파일별 크기·해시와 심볼 위치를 기록하고 소스 재확보·검증 방법을 제공합니다.
- 두 구현의 역할과 차이, 추가 검증이 필요한 가정을 구분합니다.

## 완료 기준

- [x] 프로젝트별 저장소·커밋·참고 역할 고정
- [x] 읽을 파일·함수와 G01~G10 작업 연결
- [x] 소스 매니페스트 및 로컬 캐시 확보·해시 검증
- [x] 코드 위치와 문서 링크 검증, 차이·한계 기록
- [x] 진행·결과 기록과 GitHub 이슈 게시

원작 실행 동작과 Rust 포팅의 호환성 판정 기준은 G00-03에서 정의합니다.

결과: [코드 지도](../tech_insights/reference_code_map.md), [진행 기록](../working/reference_code_map.md), [보고서](../reports/reference_code_map.md).
