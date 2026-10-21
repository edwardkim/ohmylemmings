# G11-04 전체120레벨 초기 재생 진행

이슈: [#91](https://github.com/edwardkim/ohmylemmings/issues/91) · 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1) · [G11 계획](../plans/issue-85_compatibility_validation.md).

기준 main `dfaa1fafb55769f64c585c49105186f58c7630e4`. 이슈 번호·URL 확인 후 착수했다.

## 실행 계약

- 네 난이도 각30개, 원본120개를 SDL/actual WASM에서 각각340 control steps까지 무입력 재생한다. 플랫폼별3회, 렌더 주기1/7/13과 시간 전달 분할을 바꾼다. 결과가 먼저 끝나면 남은 경계에서 상태가 고정되는지 확인한다.
- 모든 경계의 전체 정규화 상태·지형/다리/객체 맵을 대조하고 렌더가 발생한 경계의 전체 RGB와 SDL/Canvas readback을 대조한다. 초기 수치/재고·팔레트·강철/일방향/지형/객체·패널/카메라를 고정 독립 증거와 대조한다.
- 원본/파생물은 ignored assets/local, 공개는 창작 fixture·도구·메타데이터·해시다. 최초 불일치를 기록하고 초기 재생 통과를 완주·전수 DOS·성능 인증으로 합산하지 않는다.

## 진행

- [x] #91 등록·번호/URL 확인, main 동기화·기존 진단 경로 검토
- [x] 실패하는 도구 테스트와 구현
- [ ] 원본120개×양 플랫폼×3회·정적 기대값 대조
- [ ] 자체 검토·문서·한글 PR·게시/병합 CI·상위 동기화/마감

로컬 Rust611/문서29/Python172·WASM 및 창작5개×양 플랫폼3회 검증을 통과했다. 원본120개 실행과 독립 초기 대조는 진행 중이다. [계약](../tech_insights/issue-91_initial_replay.md)·[결과 기록](../reports/issue-91_initial_replay.md). 검토 후 초안 PR의 CI를 병행하고 전수 통과 전에는 병합하지 않는다. 최종 전수 수치/레벨별 해시·CI와 마감은 GitHub #91에 기록한다.
