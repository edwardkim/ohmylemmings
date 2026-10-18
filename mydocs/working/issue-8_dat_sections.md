# G02-01 DAT 구간 읽기 진행 기록

작업 이슈: [GitHub #8](https://github.com/edwardkim/ohmylemmings/issues/8). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 목표와 선행 확인

이슈 번호·URL·OPEN 상태를 확인한 뒤 시작했습니다. [전체 구현 계획](../plans/issue-1_implementation_plan.md), [코드 지도](../tech_insights/issue-3_reference_code_map.md), [호환성 기준](../tech_insights/issue-5_compatibility_baseline.md), [CI 명령](../plans/issue-7_continuous_integration.md)을 입력으로 사용합니다.

외부 의존성·힙 할당 없이 DOS DAT의 10바이트 헤더와 압축 본문을 빌려 읽습니다. 압축 해제는 G02-02로 남깁니다.

## 확인한 형식과 구현 결정

- 고정 Lemmix의 `TCompressionHeaderRec`과 Lemmings.ts의 `FileContainer.read`를 대조했습니다. 크기는 두 개의 32비트 정수가 아니라, 예약 워드 두 개와 해제/구간 크기인 네 개의 big-endian 16비트 워드입니다.
- 로컬 원본 packed DAT 21개의 102개 구간은 예약 워드가 모두 0이며 본문 XOR가 체크섬과 일치합니다. 비트 수는 0~7이 관찰됐고 0도 정상입니다. 두 디코더가 지원하는 8까지 허용합니다.
- 빈 입력은 구간 0개입니다. 각 구간은 비어 있지 않은 본문·해제 크기를 요구합니다. 예약 워드가 0이 아닌 변형은 명시적으로 거부합니다.
- 구간 iterator는 전체 입력을 끝까지 검사해야 파일 전체가 검증됩니다. 오류를 한 번 반환한 뒤 종료하고, 중간 오류 이후를 다시 탐색하지 않습니다.

## 진행 상태

- [x] 이슈 등록·확인, 소스·원본 구조 대조
- [x] API·오류와 합성 테스트를 먼저 작성하고 Red 확인: 새 테스트 13개 실패, 빈 입력 1개 통과
- [x] 최소 구현 후 정상·손상·경계 테스트 Green: 새 테스트 14개 통과
- [x] Rust 파서로 원본 21개 파일·102개 구간 대조, 전체 로컬 검사
- [ ] 실제 CI 3개 작업 성공 확인
- [ ] 결과 보고·계획·일지 동기화, 게시 확인·이슈 완료

## 검증과 다음 행동

`make check`에서 전체 Rust 단위 19개·문서 예제 3개, Python 8개와 빌드·포맷·Clippy·WASM을 통과했습니다. 원본 50개·소스 46개의 해시를 다시 확인했습니다. 호스트 예제의 정상 원본·인자 누락·손상 구간 오류도 확인했습니다.

형식과 엄격한 지원 정책은 [기술 문서](../tech_insights/issue-8_dat_sections.md), 명령·검증 근거는 [결과 보고서](../reports/issue-8_dat_sections.md)에 기록합니다. 게시 후 실제 CI·원격을 확인하고 #8을 완료합니다. 다음은 G02-02 압축 해제이며 G02 전체는 진행 중입니다.

작업일지: [20261018 / #8](../orders/20261018.md#issue-8).
