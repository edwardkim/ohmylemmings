# G11-06 성능 측정

담당 [#106](https://github.com/edwardkim/ohmylemmings/issues/106), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1), 선행 [#104](https://github.com/edwardkim/ohmylemmings/issues/104).

기준 main `973fa7868fd436844205435857e89e205b2cf908`. [측정 계약](../plans/issue-85_compatibility_validation.md#성능-측정-계약), [결과](../reports/issue-106_performance.md), [원시 시간·환경/입력/해시](../../assets/manifests/performance_baseline.json).

## 진행

- #106 등록/검증 후 TDD 계측 도구와 원본 없는 smoke를 추가했다.
- 알고리즘 Native184/Web196행 각각3+30회, 실제 SDL/Web 표시·오디오 ON60초×3, 파일 읽기 신규 프로세스33개와 기존 비교 가능한 CI3회 분석을 완료했다.
- 상태/맵/RGB와 PCM 동일성을 확인했다. Web 숨김 중단 표본은 보존하고 해당1회만 보충했으며 별도 숨김·복귀 검사를 통과했다.
- SDL 약53.2fps는60Hz 목표 미달이다. [#107](https://github.com/edwardkim/ohmylemmings/issues/107)에 분리했고 이번에는 주기를 수정하지 않는다. Web 평균60fps에도 간격 지연은 보존한다.
- self-review·게시와 정확한 push/PR/main CI·명령별 시간·병합/소스 배포·마감은 #106에 고정한다. G11/G12는 미완료다.
