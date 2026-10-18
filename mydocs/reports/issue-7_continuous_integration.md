# G01-02 CI·개발 검증 결과

작업 이슈: [#7](https://github.com/edwardkim/ohmylemmings/issues/7), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

- [설계와 공통 검증 명령](../plans/issue-7_continuous_integration.md)
- [작업일지](../orders/20261018.md#issue-7), [진행 기록](../working/issue-7_continuous_integration.md)

## 변경 내용

`make check`를 로컬/CI 공통 진입점으로 만들고 Rust native·Python tooling·WASM libraries 작업을 GitHub Actions에 연결했습니다. push·PR·수동 실행을 선언하고 원본 파일·기존 target·Rust 패키지 캐시 없는 상태를 확인합니다. 공식 Action 두 개는 확인한 릴리스의 커밋 SHA로 고정했습니다.

## 로컬 검증

| 명령·검사 | 결과 |
|---|---|
| actionlint 1.7.12로 ci.yml 검사 | 통과 |
| `make check` | Rust 단위 테스트 5개·문서 예제 2개, Python 테스트 8개, 빌드·포맷·Clippy·WASM 컴파일 통과 |
| staged Git tree `8ef0bfb`를 임시 폴더에 내보내고 빈 CARGO_HOME으로 `make check` | 같은 검사 모두 통과, 시작 시 원본·target·캐시 없음 확인 |
| Action 릴리스 태그→커밋 조회 | checkout v7.0.1, setup-python v7.0.0의 SHA와 workflow 일치 |

actionlint는 공식 v1.7.12 배포의 체크섬과 바이너리를 대조해 사용했습니다. Linux amd64 아카이브 SHA-256은 `8aca8db96f1b94770f1b0d72b6dddcb1ebb8123cb3712530b08cc387b349a3d8`입니다. 검증 도구 바이너리는 저장소에 포함하지 않습니다.

초기 정적 검사에서 job 수준 env의 `runner` 컨텍스트 사용을 거부했습니다. [컨텍스트 허용 범위](https://docs.github.com/en/actions/reference/workflows-and-actions/contexts#context-availability)에 맞춰 실행 단계의 `$RUNNER_TEMP`와 `GITHUB_ENV`로 변경한 뒤 통과했습니다.

## 원격 실행·게시 상태

게시 후 실제 실행 URL·커밋·작업별 결과와 원격 파일 대조를 기록합니다. 현재 로컬 검증만 완료했으며 G01 전체는 실제 CI 성공을 확인한 뒤 완료 처리합니다.
