# G03-02 ODDTABLE·표시 카탈로그 진행 기록

작업 이슈: [GitHub #11](https://github.com/edwardkim/ohmylemmings/issues/11). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 목표와 선행 확인

이슈 번호·URL·OPEN 상태 확인 후 시작했습니다. [기본 레벨 파서](../tech_insights/issue-10_level_records.md), [호환성 기준](../tech_insights/issue-5_compatibility_baseline.md), [전체 계획](../plans/issue-1_implementation_plan.md), [기존 모듈 경계](../plans/issue-6_rust_workspace.md)를 입력으로 사용합니다.

120개 표시 레벨을 기본 DAT 구간에 연결하고 40개 변형의 수치·제목만 ODDTABLE에서 적용합니다. 기본 화면·그래픽·배치 바이트는 보존합니다. 그래픽 리소스 해석은 후속 G03 범위입니다.

## 설계 방향과 다음 행동

- `lemmings-resources → lemmings-core`의 로컬 의존 하나로 기존 검증된 `LevelId`를 재사용합니다. 코어는 파서·리소스·플랫폼을 참조하지 않고 외부 패키지는 추가하지 않습니다.
- ODD 인덱스는 표시 순번이 아닌 기본 레코드 번호 `file × 8 + part`입니다. Fun 8의 기준 인덱스는 6입니다.
- 고정 120개 순서표와 원본 ODD의 미사용 항목을 분석하고, 입력 길이·인덱스·필드 오류와 잘못된 파일 연결 정책을 정의합니다.

## 진행 상태

- [x] 이슈 등록·확인, 문서와 고정 소스 읽기
- [x] 순서표·ODD 통계와 API·오류·의존 방향 확정
- [x] 합성 테스트 먼저 작성, Red→Green 확인: 새 13개 실패 후 전부 통과
- [x] 원본 120개 최종 수치·제목·연결·배치 보존 대조
- [ ] 전체 로컬 검사·실제 CI·문서·원격 게시 확인

작업일지: [20261018 / #11](../orders/20261018.md#issue-11).

## 구현·검증 결과와 다음 행동

`OddTable`·`CatalogEntry`·`DosLevelFile`·`ResolvedLevel`과 공통 수치 검증을 구현했습니다. `make check`에서 Rust 단위 69개·문서 예제 6개·Python 8개와 빌드·포맷·Clippy·WASM을 통과했습니다. 원본 120개 연결·최종 수치·32바이트 제목이 독립 Python·기존 목록과 일치하고, 40개 변형을 포함한 전체 기본 바이트가 보존됩니다.

[기술 계약·재현 명령](../tech_insights/issue-11_level_catalog.md)과 [보고서](../reports/issue-11_level_catalog.md)를 동기화하고 실제 CI·문서·원격 게시를 확인합니다. G03 전체는 진행 중이며 다음은 G03-03 GROUND 메타데이터·팔레트 해석입니다.
