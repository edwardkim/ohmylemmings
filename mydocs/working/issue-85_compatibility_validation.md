# G11-01 전체 호환성·성능 검증 준비

이슈: [#85](https://github.com/edwardkim/ohmylemmings/issues/85), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

기준 main: `7b64dad34d90fa27249b9d76a6339dd1faed3aa2`. 이슈 번호·URL을 확인한 뒤 `issue-85-compatibility-plan`에서 착수했다.

## 범위와 진행

- G11 첫 절편은 120레벨·8직업·21곡·18효과음·COMP-01~18 점검표, 기존 증거 분류, 차이 목록, 후속 절편과 성능 측정 기준이다.
- 소스에서 공통 로더·SDL·Web이 Fun 1–30에 한정되고 특수 배경을 거부하는 것을 확인했다. 자원 계층의 120레벨 정적 합성 증거와 호스트 실행 인증을 구분한다.
- 제품 코드·워크플로·의존성 변경 없이 문서와 점검 메타데이터를 작성하고 목록·해시·링크·변경 범위와 자체 검토를 수행한다. 제품 CI는 동일 코드의 기존 성공 실행을 재사용한다.
- 원본/PCM과 개별 확인 자료는 ignored `assets/local/issue-85-review/`에 둔다. HTTP 8080·CDP 19222 loopback 및 무관한 미추적 파일을 유지한다.

## 검증과 인계

[계획](../plans/issue-85_compatibility_validation.md), [120개 표](../tech_insights/issue-85_compatibility_matrix.md), [결과 보고](../reports/issue-85_compatibility_validation.md)를 작성했다. 목록·29/91 집계·미검증 상태·공개 증거12개/소스10개 해시·상대 링크·변경 범위 검사가 통과했다. 자체 검토에서 전체 인증으로 오인할 판정, 오래된 곡 순서 복제, 오디오를 CI 지연 원인으로 단정하는 오류가 없는지 확인했다. 미해결 검토 지적은 없다.

게시 후 #85와 상위 #1을 동기화하고 이 절편을 마감한다. 병합 SHA/PR은 #85에 기록한다. 다음 작업은 G11-02 난이도/레벨 ID 연결이다. G11 전체 완료와 성능 통과는 후속 검증 대상이다.
