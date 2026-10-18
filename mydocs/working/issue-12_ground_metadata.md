# G03-03 GROUND 메타데이터·팔레트 진행 기록

작업 이슈: [GitHub #12](https://github.com/edwardkim/ohmylemmings/issues/12). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 범위와 순서

1. 고정 Lemmix·Lemmings.ts와 원본 GROUND 5개를 대조해 필드·팔레트·빈 슬롯 정책을 정합니다.
2. 합성 입력으로 Red를 확인한 뒤 no_std·무할당 파서를 구현합니다.
3. 호스트 검사 예제, 독립 원본 대조, 전체 검사로 검증합니다.
4. 기술 분석·보고·일지·상위 계획을 갱신하고 게시·실제 CI를 확인합니다.

VGAGR 픽셀·마스크, VGASPEC과 지형 재구성은 후속 범위입니다.

## 현재 상태

이슈 #12의 번호·URL·OPEN 상태를 확인했습니다. 읽기 전용 사전 조사 후 이슈를 등록했으며, 구현·실험은 이 등록 다음에 시작합니다.

## 구현·검증

- 고정 소스와 원본·소스 매니페스트 50개·46개를 재검증했습니다.
- 새 합성 테스트 12개 실패·기존 67개 통과의 Red 이후 79개 리소스 테스트가 통과했습니다.
- `GroundRecord`, 슬롯 메타데이터, 팔레트 세 뱅크와 두 RGB 확대 방식을 도입했습니다.
- `make check`: Rust 단위 81개·문서 예제 7개·Python 8개, 포맷·Clippy·빌드·WASM 성공.
- 원본 5개 파일의 객체 80슬롯·지형 320슬롯, EGA 120개·VGA 120색과 계산값 전체가 독립 Python 해석과 일치합니다.

## 산출물과 다음 행동

[기술 분석·재현 명령](../tech_insights/issue-12_ground_metadata.md), [결과 보고서](../reports/issue-12_ground_metadata.md), [작업일지](../orders/20261018.md#issue-12).

문서 검사·게시·실제 CI·원격 tree 확인을 마친 뒤 #12를 종료합니다. G03 전체는 유지하며 다음 작업은 별도 이슈의 G03-04 VGAGR 비트플레인·마스크 해석입니다.
