# G02-02 DAT 압축 해제 진행 기록

작업 이슈: [GitHub #9](https://github.com/edwardkim/ohmylemmings/issues/9). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 목표와 입력

이슈 번호·URL·OPEN 상태를 확인한 뒤 착수했습니다. [G02-01 형식과 API](../tech_insights/issue-8_dat_sections.md), [고정 코드 지도](../tech_insights/issue-3_reference_code_map.md), [호환성 기준](../tech_insights/issue-5_compatibility_baseline.md), [전체 계획](../plans/issue-1_implementation_plan.md)을 입력으로 사용합니다.

`DatSection`의 압축 본문을 호출자가 제공한 버퍼에 해제합니다. 역방향 비트·6종 명령·겹침 복사와 잘린 입력·범위 오류를 구현하며 `no_std`·무할당·외부 의존성 0개를 유지합니다. 원본 없는 합성 테스트와 로컬 원본 대조를 구분합니다.

## 진행 상태

- [x] 이슈 선등록·확인, 관련 문서와 두 레퍼런스 읽기
- [x] 비트·명령·입력 소비와 버퍼 오류 계약 정의
- [x] 테스트 먼저 작성, Red→Green 확인: 새 20개 실패 후 전부 통과
- [x] 원본 81개 구간의 고정 크기·해시 대조 및 총 102개 구간의 세 구현 대조
- [x] 전체 로컬 검사·문서의 재현 명령 실행, 링크·파일명·공백 확인
- [x] 구현 커밋의 실제 CI 3개 작업 성공, 원격 파일 63개 blob 대조

## 구현·검증 내용

`DatSection::decompress_into`와 `DecompressError`, 호스트 예제 `unpack_dat`을 추가했습니다. 초기 비트 0~8을 지원하고 선언된 유효 비트를 모두 소비해야 성공합니다. 크기 오류는 쓰기 전에 거부하며 다른 오류의 출력은 버립니다. 현재 `make check`에서 Rust 단위 39개·문서 예제 4개·Python 8개와 빌드·포맷·Clippy·WASM을 통과했습니다.

원본 50개·소스 46개 해시를 재검증하고 기존 81개 해제 기대값을 유지한 채 대조했습니다. MAIN·그래픽을 포함한 21개 파일·102개 구간, 864757바이트의 Rust/Python/고정 TypeScript 크기·해시도 모두 일치합니다.

## 다음 행동

구현 커밋 `a9ce6b1`의 [실제 CI](https://github.com/edwardkim/ohmylemmings/actions/runs/39108031890)에서 세 작업 모두 성공했고 대상 SHA·실제 로그·원격 파일 63개를 확인했습니다. G02-01과 G02-02의 범위·완료 기준을 충족해 G02를 완료했습니다.

[기술 계약·재현 명령](../tech_insights/issue-9_dat_decompression.md), [결과 보고서](../reports/issue-9_dat_decompression.md)를 동기화했습니다. 최종 문서 게시·CI와 이슈 종료 기록은 GitHub #9에서 추적합니다. 다음은 새 이슈로 진행할 G03-01 레벨 레코드 해석이며 G03·G04는 준비 상태입니다.

작업일지: [20261018 / #9](../orders/20261018.md#issue-9).
