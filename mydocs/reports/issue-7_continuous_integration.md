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
| 문서 링크·앵커·파일명 검사 | Markdown 36개, 로컬 링크 175개, 앵커 28개, 이슈 문서명 24개 통과 |
| `git diff --check`, `git diff --cached --check` | 통과 |

actionlint는 공식 v1.7.12 배포의 체크섬과 바이너리를 대조해 사용했습니다. Linux amd64 아카이브 SHA-256은 `8aca8db96f1b94770f1b0d72b6dddcb1ebb8123cb3712530b08cc387b349a3d8`입니다. 검증 도구 바이너리는 저장소에 포함하지 않습니다.

초기 정적 검사에서 job 수준 env의 `runner` 컨텍스트 사용을 거부했습니다. [컨텍스트 허용 범위](https://docs.github.com/en/actions/reference/workflows-and-actions/contexts#context-availability)에 맞춰 실행 단계의 `$RUNNER_TEMP`와 `GITHUB_ENV`로 변경한 뒤 통과했습니다.

## 실제 CI·게시 결과

구현 커밋 `37b988ac7232be8ca22e000d1b0c6a8d083580be`의 [실제 CI 실행](https://github.com/edwardkim/ohmylemmings/actions/runs/39105594092)은 push 이벤트로 실행됐으며 전체 결론은 success입니다.

| 작업 | 결과와 실행 근거 |
|---|---|
| [Rust native](https://github.com/edwardkim/ohmylemmings/actions/runs/39105594092/job/117610622896) | 성공: clean-input, Rust 1.93.1, 포맷·Clippy·빌드·단위 5개·문서 예제 2개 |
| [Python tooling](https://github.com/edwardkim/ohmylemmings/actions/runs/39105594092/job/117610622767) | 성공: clean-input, Python 3.12.15, 테스트 8개 |
| [WASM libraries](https://github.com/edwardkim/ohmylemmings/actions/runs/39105594092/job/117610622878) | 성공: clean-input, 고정 Rust와 WASM 타깃 설치, 라이브러리 2개 컴파일 |

작업 상태뿐 아니라 실행 로그의 테스트 개수·도구 체인과 대상 커밋을 대조했습니다. PR·수동 이벤트는 선언·정적 검사까지 확인했으며 이번 실제 실행 이벤트는 push입니다.

`git ls-tree -r HEAD`와 GitHub Git tree API를 대조해 구현 커밋의 파일 53개 blob 해시가 모두 일치했습니다. 상위 #1에 G01-02 완료와 G01 전체 완료를 반영했고 G00·G01만 체크한 열린 상태를 확인했습니다.

다음 작업은 각각 별도 이슈로 진행할 G02-01 DAT 구간 읽기와 G04-01 픽셀 지형 읽기·경계 검증입니다.
