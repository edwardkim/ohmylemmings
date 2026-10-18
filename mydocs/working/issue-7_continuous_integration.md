# G01-02 CI·개발 검증 진행 기록

작업 이슈: [#7](https://github.com/edwardkim/ohmylemmings/issues/7), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 시작 조건과 범위

완료했습니다. 이슈 등록과 번호·URL·열린 상태를 확인한 뒤 작업했고 #6의 라이브러리·합성 입력 테스트를 로컬/CI의 같은 명령에 연결했습니다. 실제 Actions 세 작업의 성공을 확인해 G01 전체를 완료했습니다.

- `make check`: Rust·Python·WASM 검사 전체 실행.
- GitHub Actions: push·PR·수동 실행, Ubuntu 24.04의 독립 작업 3개.
- Rust 1.93.1과 Cargo.lock을 사용하고 공식 checkout/setup-python Action을 검증한 커밋 SHA로 고정합니다.
- 원본 파일·target이 없는 체크아웃과 비어 있는 Cargo 홈에서 검증합니다.

## 로컬 검증 결과

- actionlint 1.7.12 정적 검사를 통과했습니다. 초기 env 컨텍스트 오류를 실행 단계의 환경 설정으로 수정했습니다.
- 로컬 `make check`와 원본·target·캐시가 없는 Git tree 내보내기의 같은 명령을 통과했습니다.
- Rust 단위 테스트 5개·문서 예제 2개, Python 테스트 8개, 빌드·포맷·Clippy·WASM 컴파일을 확인했습니다.
- 문서 36개의 로컬 링크 175개·앵커 28개, 이슈 문서명 24개와 공백 검사를 통과했습니다.
- [설계](../plans/issue-7_continuous_integration.md), [보고서](../reports/issue-7_continuous_integration.md), [작업일지](../orders/20261018.md#issue-7)를 연결합니다.

## 다음 행동

G02-01과 G04-01을 각각 범위·테스트가 있는 별도 이슈로 등록·확인한 뒤 진행합니다.

## 실제 CI·게시 확인

- 구현 커밋 `37b988a`의 [push CI 실행](https://github.com/edwardkim/ohmylemmings/actions/runs/39105594092)에서 Rust native·Python tooling·WASM libraries가 모두 성공했습니다.
- 실행 로그에서 Rust 단위 테스트 5개·문서 예제 2개와 Python 테스트 8개, clean-input 검사 성공을 대조했습니다. 실제 버전은 Rust 1.93.1·Python 3.12.15였습니다.
- 구현 커밋의 원격 파일 53개가 로컬 blob 해시와 일치했습니다. 상위 #1은 G00·G01만 완료 목표로 체크하고 열린 상태로 유지했습니다.
- PR·수동 이벤트는 workflow 선언과 정적 검사를 확인했으며 실제 실행 증거는 push 이벤트입니다.
