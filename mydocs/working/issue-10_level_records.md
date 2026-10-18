# G03-01 DOS 기본 레벨 레코드 진행 기록

작업 이슈: [GitHub #10](https://github.com/edwardkim/ohmylemmings/issues/10). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 목표와 선행 확인

이슈 번호·URL·OPEN 상태를 확인한 뒤 착수했습니다. [G02 압축 해제](../tech_insights/issue-9_dat_decompression.md), [고정 코드 지도](../tech_insights/issue-3_reference_code_map.md), [호환성 기준](../tech_insights/issue-5_compatibility_baseline.md), [전체 계획](../plans/issue-1_implementation_plan.md)을 입력으로 사용합니다.

2048바이트 기본 레벨의 헤더·객체·지형·강철·32바이트 제목을 `no_std`·무할당으로 읽고 검증합니다. 원본 80개 기본 레코드를 대조하며 ODDTABLE 변형·120개 연결과 그래픽 리소스 결합은 후속 작업으로 분리합니다.

## 진행 상태

- [x] 이슈 선등록·확인, 관련 문서와 고정 두 구현 읽기
- [x] 좌표·슬롯·플래그 차이와 지원 정책 정의, 원본 통계 확인
- [x] 합성 테스트 먼저 작성, Red→Green 확인: 새 테스트 17개 실패 후 전부 통과
- [x] 원본 80개 기본 레코드 수치·제목·배치 대조
- [ ] 전체 로컬 검증·실제 CI·문서·원격 게시 확인

## 구현·검증 결과

`LevelRecord`·`LevelHeader`·배치 자료형·`LevelError`와 `inspect_levels` 호스트 예제를 추가했습니다. 정확한 2048바이트·헤더 범위·강철 예약 바이트를 검증하고 원래 슬롯과 원문 제목을 보존합니다. `make check`에서 Rust 단위 56개·문서 예제 5개·Python 8개와 빌드·포맷·Clippy·WASM을 통과했습니다.

80개 기본 레코드의 고정 해시를 확인하고 독립 Python과 헤더·제목·객체 454개·지형 11964개·강철 392개를 전부 대조했습니다. 변형 없는 80개 카탈로그 진입점의 초기 수치와 제목도 일치합니다. 강철·지형 Y의 레퍼런스 차이는 [기술 계약](../tech_insights/issue-10_level_records.md)에 명시했으며 [보고서](../reports/issue-10_level_records.md)에 검증 근거를 기록합니다.

## 다음 행동

재현 명령·문서와 실제 CI·원격 파일을 확인한 뒤 #10을 완료합니다. G03 전체는 진행 중이며 다음은 별도 이슈의 G03-02 ODDTABLE 변형·120개 카탈로그 연결입니다. 그래픽 해석·지형 재구성은 이후 G03 작업으로 남습니다.

작업일지: [20261018 / #10](../orders/20261018.md#issue-10).
