# 플레이 피드백 개선 진행

담당 [#216](https://github.com/edwardkim/ohmylemmings/issues/216), [계획](../plans/issue-216_play_feel.md).

2026-10-24: 이슈 번호/URL 확인 후 `issue-216-play-feel` 별도 worktree를 #215 후보f9f0f65에서 만들었다. #215 PR [#217](https://github.com/edwardkim/ohmylemmings/pull/217)의 CI/병합과 분리한다. #215 병합 후 기준을 동기화한다.

## 초기 확인

- Fun02 기존 실제100틱 캡처의 최초 상태는 digger.remaining=0, floater.remaining=10이다. 적용 불가는 정상 자원 규칙이며 이를 알아보기 어렵게 만드는 UI를 개선해야 한다.
- `crates/lemmings-core/src/simulation/motion/digger.rs`는16프레임 주기의0/8위상에9셀 폭 행을 제거한다. 새 작업 시작 시 추가 초기 행 처리도 있다. 기존 규칙을 유지하면서 표시/소리의 동기화를 먼저 검토한다.
- core `SoundCue`에는 Splat/AssignmentBlocked/SkillButtonSelect는 있으나 전용 Digging cue는 없다. 자원 매핑은 Splat→8, AssignmentBlocked→18이며 수량0 경로에 같은 실패음이 전달되는지는 후속 확인이다.
- `hd/scene.rs`의 primary hit 대상 표시가 발밑30×3HDpx Fill 선이다. 실제 선택 대상 ID는 유지하며 원형 표현으로 바꿀 수 있다.
- Splatting은 HD에서 별도 스프라이트 종류로 매핑된다. 실제 프레임/발 anchor/충돌 시점의 재현 검증은 아직 수행하지 않았다. 원인을 확정하지 않는다.

아직 #216 런타임 구현/새 효과음/추락사 수정 완료는 아니다. 다음은 수량0/적용 불가/마우스·키보드 선택의 회귀 테스트와 실제 조준 대상 표시 구현이다.
