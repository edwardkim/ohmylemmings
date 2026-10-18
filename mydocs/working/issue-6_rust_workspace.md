# G01-01 Rust 개발 골격 진행 기록

작업 이슈: [#6](https://github.com/edwardkim/ohmylemmings/issues/6), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 시작 조건과 범위

완료했습니다. 이슈 등록·번호·URL·열린 상태를 확인한 뒤 작업했고 G00의 고정 입력과 [호환성 계약](../tech_insights/issue-5_compatibility_baseline.md)을 기준으로 Cargo workspace와 순수 코어·리소스 입력 경계를 도입했습니다.

- `crates/lemmings-core`: 원작 레벨 번호 검증과 식별 자료형.
- `crates/lemmings-resources`: 빌린 바이트 입력과 경계 오류 처리.
- 두 라이브러리는 `no_std`이며 외부 의존성·동적 할당·파일 접근이 없습니다.
- 플랫폼 I/O와 SDL2/웹 실행은 G06/G09의 어댑터가 담당합니다. CI는 G01-02에서 구성합니다.

## 검증 결과

- Rust 1.93.1을 고정하고 Cargo manifest·API 골격·행동 테스트 5개를 먼저 작성했습니다.
- TDD Red에서 5개 실패를 확인하고 최소 구현 후 단위 테스트 5개·문서 예제 2개를 통과했습니다.
- 오프라인 workspace/개별 빌드, 포맷·Clippy와 WASM 라이브러리 컴파일을 통과했습니다. Cargo metadata로 의존성 0개를 확인했습니다.
- 문서 33개의 로컬 링크 151개·앵커 24개, 이슈 번호 문서명 21개와 공백 검사를 통과했습니다.
- 설계와 API는 [개발 골격 문서](../plans/issue-6_rust_workspace.md), 상세 검증은 [보고서](../reports/issue-6_rust_workspace.md), 당일 작업은 [작업일지](../orders/20261018.md#issue-6)에 기록합니다.

## 다음 행동

G01-02 이슈를 먼저 등록·확인한 뒤 CI·개발 검증 구성을 진행합니다. G01 전체는 진행 상태입니다.

## 게시 확인

- 구현 커밋 `575c400`을 `main`에 게시했고 원격 파일 48개의 blob 해시가 로컬 HEAD와 일치했습니다.
- 빌드 산출물과 원본 바이너리는 Git 추적 대상에서 제외됨을 확인했습니다.
- 상위 #1에 G01-01/#6 완료와 다음 작업 G01-02를 반영했습니다. G00만 완료 목표로 체크했고 G01~G12는 미완료, 상위 이슈는 열린 상태입니다.
