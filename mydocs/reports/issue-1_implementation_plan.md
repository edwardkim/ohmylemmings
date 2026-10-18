# 전체 구현 계획 수립 결과

## 대상과 범위

- [작업일지](../orders/20261018.md#issue-1)
- [전체 구현 계획](../plans/issue-1_implementation_plan.md)
- [진행 기록](../working/issue-1_implementation_plan.md)

이번 작업의 산출물은 구현 계획과 추적 체계입니다. 구현 목표 G00~G12의 현재 상태는 대기입니다.

## 작성한 내용

- 13개 목표와 선행 조건, 작은 작업, 완료 기준
- 독립적으로 구현할 수 있는 리소스·코어 경로와 첫 통합 목표
- G00에서 시작하는 초기 작업 순서와 후속 이슈 작성 규칙
- `working/` 진행 기록, `reports/` 검증 결과, 상위 이슈 체크리스트의 연결

## 검증 상태

- 로컬 Markdown 링크 36개와 공백 검사를 통과했습니다.
- G00~G12의 13개 목표 ID와 선행 목표를 검사했고 누락·순환 의존성이 없음을 확인했습니다.
- [GitHub 이슈 #1](https://github.com/edwardkim/ohmylemmings/issues/1)을 등록하고 전체 구현 목표를 미완료 체크리스트로 남겼습니다.
- `git diff --cached --check`를 통과했습니다.
- 계획 게시 커밋 `8bd673c`를 `main`에 푸시했습니다. `git ls-remote origin refs/heads/main`과 로컬 HEAD가 일치했습니다.
- `gh api repos/edwardkim/ohmylemmings/git/trees/main?recursive=1` 결과의 원격 파일 17개를 `git ls-tree -r HEAD`의 blob 해시와 비교해 모두 일치함을 확인했습니다.
- `gh api repos/edwardkim/ohmylemmings/issues/1`에서 이슈가 열린 상태이며 G00~G12 체크리스트 13개가 모두 미완료임을 확인했습니다.
- Rust 빌드·테스트는 `Cargo.toml` 도입 전이므로 실행 대상이 없습니다.

## 후속 작업

G00-01에서 대상 DOS 기준판과 리소스를 식별합니다. 기준판·레퍼런스·검증 기준을 확정하면 G01 개발 기반을 구축합니다.

## G05 완료 갱신 / #26

[#26 보고서](issue-26_state_replay.md)의 실제 고정 Lemmix 함수 하네스·합성 42개/원본 다섯 사례·세 렌더 주기 상태 대조와 [게시 CI](https://github.com/edwardkim/ohmylemmings/actions/runs/39149874933)를 확인했습니다. G05-01~04의 고정 틱·입력·생성/기본 이동·상태 기록·반복 재생으로 G05 전체 완료 기준을 충족했습니다. 전체 trace-v1·DOS·직업/객체·승패/오디오 인증은 후속 목표에 남습니다. 다음 후보는 G06-01 네이티브 화면입니다.

## G06-01 완료 갱신 / #27

[#27 네이티브 화면·입력](issue-27_native_view.md)의 원본 X11 6실행/1786상태·63개 픽셀 대조 화면과 실제 키 입력 재생, [게시 CI](https://github.com/edwardkim/ohmylemmings/actions/runs/39152276025)의 세 작업을 확인했습니다. G06-01을 완료했으며 G06 전체는 #5의 DOS 실행·동일 장면 관측 증거가 남아 진행 중입니다. 다음 후보는 별도 이슈의 G06-02입니다.

## G06 전체 완료 갱신 / #30

[#30 보정 결과](issue-30_dos_differences.md)의 원본 세 영상/일곱 전체 장면 0픽셀 차이·타이머 경로 분리·pacing 보정·실제 X11/키/상태 회귀와 [게시 CI](https://github.com/edwardkim/ohmylemmings/actions/runs/39158078764)의 세 작업을 확인했습니다. G06-01~03을 마쳐 G06 전체를 완료했습니다. 제한된 DOS 카운터 관측은 전체 상태 인증이 아니며 GUI/5000의 1영상 프레임 차이는 보존합니다. 다음은 이슈를 먼저 등록하는 G07-01 Digger입니다.

## G07 첫 직업 완료

[#31 Digger](issue-31_digger.md)의 할당/수량·지형 삭제/강철·기준 함수 실행·원본 네이티브/DOS 비교·실제 CI를 완료했습니다. G07 전체는 진행 중이며 다음 후보는 G07-02 Basher입니다. 먼저 새 이슈에 범위/테스트/완료 기준을 등록하고 번호/URL을 확인합니다.
