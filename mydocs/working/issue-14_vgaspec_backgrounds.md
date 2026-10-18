# G03-05 VGASPEC 특수 배경 진행 기록

작업 이슈: [GitHub #14](https://github.com/edwardkim/ohmylemmings/issues/14). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 범위와 순서

번호·URL·OPEN 상태를 먼저 확인했습니다. 고정 소스·원본 대조 → 합성 TDD → 무할당 VGASPEC API → 원본 4개의 전체 픽셀·팔레트 독립 비교 → 문서·전체 검사·게시·실제 CI 순서로 진행합니다.

특수 배경 이미지와 팔레트를 해석합니다. 전체 레벨 지형 합성·충돌·렌더러·DOS 실행 호환성은 후속 범위입니다.

## 현재 상태

G03-05 구현·원본 전체 대조·검증·게시를 완료했습니다. 착수 시 이슈 등록·번호·URL·OPEN 상태를 확인한 뒤 문서·실험·구현을 진행했습니다.

## 진행과 근거

- 원본 50개·고정 소스 46개와 앞 단계 추가 소스 2개의 크기·해시를 재검증했습니다.
- 새 합성 테스트 14개 실패·기존 리소스 93개 통과의 Red 이후 리소스 107개 테스트가 통과했습니다.
- `VgaSpecFile`·`VgaSpecRecord`·`VgaSpecImage`와 정확한 스트립/RLE·출력 경계 검증을 도입했습니다. 기존 RGB·픽셀 타입을 재사용합니다.
- `make check`: Rust 단위 109개·문서 예제 9개·Python 8개, 포맷·Clippy·빌드·WASM 성공.
- 원본 4개·614400픽셀·230400플레인 바이트와 전체 내부 데이터·팔레트가 Rust·독립 Python·고정 TS와 일치했습니다. 내부 바이트도 기존 G02-02 결과를 유지합니다.
- TS Frame/ColorPalette 캡처 어댑터로 인덱스·팔레트·스트립 배치 호출을 비교했으며 실제 RGB/알파 렌더링은 검증하지 않았습니다.
- 고정 해시·팔레트 매니페스트를 작성하고, 문서 재현 명령을 그대로 실행해 내부/플레인/픽셀 해시·독립 RLE와 픽셀을 확인했습니다.
- CLI 인자·파일 누락·추가 DAT 구간·손상 후미·짧은 헤더·팔레트·종료 표식·내부 후미 오류를 확인했습니다.

## 산출물과 다음 행동

[기술 분석·재현 명령](../tech_insights/issue-14_vgaspec_backgrounds.md), [결과 보고서](../reports/issue-14_vgaspec_backgrounds.md), [작업일지](../orders/20261018.md#issue-14).

구현 커밋 `722f9d8`과 [실제 CI 세 작업](https://github.com/edwardkim/ohmylemmings/actions/runs/39116139981)의 성공을 확인했습니다. push SHA·로그의 Rust 단위 109개·문서 예제 9개·Python 8개·clean-input 검사를 대조했습니다. 원격 파일 91개의 모드·blob 해시가 로컬과 일치하며 작업 트리는 깨끗했습니다.

최종 문서 게시·CI를 확인하고 상위 #1을 갱신한 뒤 #14를 종료합니다. G03 전체는 진행 중이며 다음은 별도 이슈의 G03-06 레벨 지형 재구성입니다.
