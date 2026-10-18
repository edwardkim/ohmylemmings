# G01-02 CI·개발 검증 진행 기록

작업 이슈: [#7](https://github.com/edwardkim/ohmylemmings/issues/7), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 시작 조건과 범위

이슈 등록과 번호·URL·열린 상태를 확인한 뒤 작업을 시작했습니다. #6의 `no_std` 라이브러리 2개와 합성 입력 테스트를 로컬/CI의 같은 검증 명령에 연결합니다.

- `make check`: Rust·Python·WASM 검사 전체 실행.
- GitHub Actions: push·PR·수동 실행, Ubuntu 24.04의 독립 작업 3개.
- Rust 1.93.1과 Cargo.lock을 사용하고 공식 checkout/setup-python Action을 검증한 커밋 SHA로 고정합니다.
- 원본 파일·target이 없는 체크아웃과 비어 있는 Cargo 홈에서 검증합니다.

## 로컬 검증 결과

- actionlint 1.7.12 정적 검사를 통과했습니다. 초기 env 컨텍스트 오류를 실행 단계의 환경 설정으로 수정했습니다.
- 로컬 `make check`와 원본·target·캐시가 없는 Git tree 내보내기의 같은 명령을 통과했습니다.
- Rust 단위 테스트 5개·문서 예제 2개, Python 테스트 8개, 빌드·포맷·Clippy·WASM 컴파일을 확인했습니다.
- [설계](../plans/issue-7_continuous_integration.md), [보고서](../reports/issue-7_continuous_integration.md), [작업일지](../orders/20261018.md#issue-7)를 연결합니다.

## 다음 행동

게시 후 실제 Actions 작업 3개의 결과·실행 URL을 확인합니다. 문서·원격 상태 검증 후 #7과 상위 #1에 G01 완료 여부를 반영합니다.
