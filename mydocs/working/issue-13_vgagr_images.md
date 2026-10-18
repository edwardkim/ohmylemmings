# G03-04 VGAGR 비트플레인·마스크 진행 기록

작업 이슈: [GitHub #13](https://github.com/edwardkim/ohmylemmings/issues/13). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 범위와 순서

이슈 번호·URL·OPEN 상태를 먼저 확인했습니다. 고정 소스·원본 대조 → 합성 TDD → 무할당 이미지 API → 원본 전체 프레임의 독립 비교 → 문서·검사·게시·실제 CI 순서로 진행합니다.

GROUND 메타데이터와 VGAGR의 지형/객체 구간을 결합해 팔레트 인덱스·불투명 마스크를 읽습니다. VGASPEC·전체 레벨 지형 합성·시뮬레이션·렌더러는 후속 범위입니다.

## 현재 상태

로컬 구현·원본 전체 픽셀 대조를 마쳤으며 문서 검사·게시·CI를 확인합니다. 착수 시 번호·URL·OPEN 상태를 확인한 뒤 문서·실험·구현을 진행했습니다.

## 진행과 근거

- 원본 50개·고정 소스 46개를 재검증하고 추가 소스 2개를 고정 커밋·크기·해시로 식별했습니다. Pascal contents 응답의 바이트 불일치는 raw 다운로드 후 Git blob SHA-1 대조로 해결했습니다.
- 새 합성 테스트 14개 실패·기존 리소스 79개 통과의 Red 이후 93개 리소스 테스트가 통과했습니다.
- `VgagrFile`·`VgagrImages`·`ObjectFrames`·`PlanarImage`와 독립 색상/불투명 픽셀 API를 구현했습니다.
- `make check`: Rust 단위 95개·문서 예제 8개·Python 8개, 포맷·Clippy·빌드·WASM 성공.
- 원본 지형 273개·객체 540프레임·미리보기 54개, 총 771664픽셀이 Rust·독립 Python·고정 TS 전체 바이트에서 일치했습니다.
- 해제 구간 10개의 크기·해시도 Python·TS·기존 G02-02 결과와 같습니다. 새 매니페스트에 구간·픽셀 연결 해시를 고정했습니다.
- 문서 재현 코드 그대로 실행해 형상·순서·픽셀 해시를 확인했습니다. CLI 인자·입력 누락·GROUND 길이·추가 구간·후미 손상·너비 오류도 확인했습니다.

## 산출물과 다음 행동

[기술 분석·재현 명령](../tech_insights/issue-13_vgagr_images.md), [결과 보고서](../reports/issue-13_vgagr_images.md), [작업일지](../orders/20261018.md#issue-13).

문서·게시·실제 CI·원격 tree를 확인하고 상위 #1을 갱신한 뒤 #13을 종료합니다. G03 전체는 진행 중이며 다음은 별도 이슈의 G03-05 VGASPEC 특수 배경 해석입니다.
