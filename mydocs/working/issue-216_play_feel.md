# 플레이 피드백 개선 진행

담당 [#216](https://github.com/edwardkim/ohmylemmings/issues/216), [계획](../plans/issue-216_play_feel.md).

2026-10-24: 이슈 번호/URL 확인 후 `issue-216-play-feel` 별도 worktree를 #215 후보f9f0f65에서 만들었다. #215 PR [#217](https://github.com/edwardkim/ohmylemmings/pull/217)의 CI/병합과 분리한다. #215 병합 후 기준을 동기화한다.

## 초기 확인

- Fun02 기존 실제100틱 캡처의 최초 상태는 digger.remaining=0, floater.remaining=10이다. 적용 불가는 정상 자원 규칙이며 이를 알아보기 어렵게 만드는 UI를 개선해야 한다.
- `crates/lemmings-core/src/simulation/motion/digger.rs`는16프레임 주기의0/8위상에9셀 폭 행을 제거한다. 새 작업 시작 시 추가 초기 행 처리도 있다. 기존 규칙을 유지하면서 표시/소리의 동기화를 먼저 검토한다.
- core `SoundCue`에는 Splat/AssignmentBlocked/SkillButtonSelect는 있으나 전용 Digging cue는 없다. 자원 매핑은 Splat→8, AssignmentBlocked→18이며 수량0 경로에 같은 실패음이 전달되는지는 후속 확인이다.
- `hd/scene.rs`의 primary hit 대상 표시가 발밑30×3HDpx Fill 선이다. 실제 선택 대상 ID는 유지하며 원형 표현으로 바꿀 수 있다.
- Splatting은 HD에서 별도 스프라이트 종류로 매핑된다. 실제 프레임/발 anchor/충돌 시점의 재현 검증은 아직 수행하지 않았다. 원인을 확정하지 않는다.

## 구현 및 검증 후보

#215 완료 main f60827e를 병합한 뒤 공유 game 계층에 구현했다.

- 수량0 기술은 키보드와 툴바 모두 선택을 유지한다. HD 패널에 EMPTY를 표시한다. 실제 Fun02 Digger0/Floater10을 확인했다.
- HD 발밑 선을 캐시된 원형 조준점으로 교체했다. 대상 해석 실패/수량0에는 빨간 X를 더한다. 청록색은 대상 표시이며 모든 적용 조건의 성공 보장은 아니다.
- Digger 굴착 위상에 몸의 짧은 압축과 분진을 추가했다. 지형 변경을 확인한 경계에 DigStroke를 전달하며 굴착 폭/속도/충돌 규칙은 유지한다.
- 100ms 절차적 노이즈 굴착음을 코드로 생성한다. 외부 음원은 사용하지 않으며 프로젝트 MIT 코드에 포함된다. 기존 효과음/마스터 볼륨을 따르고 일시정지·정지·리셋 때 잔음을 제거한다.
- 치명적 추락의 Splatting에서 넓고 납작한 충격 자세와 분진을 표시한다. 안전 착지/사망 시점/기존 Splat음은 보존한다.

검증 자료는 ignored `assets/local/issue-216-review/`에 보관한다. 원본 유래 화면/PCM은 공개 저장소에 추가하지 않는다.

- Python 333개 통과, workspace 단위 테스트 통과, 전체 target clippy 통과, 최신 Native release/WASM 빌드 성공.
- Native 입력22개 및 신규 수량0/굴착 상태·일시정지/PCM 청크 독립성·effects mute 테스트 통과.
- 원본 사운드98사례·97효과 분기·170음악 조합 및 기존6 PCM/229665프레임 유지. 신규 합성 trace에 DigStroke가 추가되어 해당 trace hash만 갱신했다.
- 사운드 경계 검사에서 발견한 post-tick control_step 오류를 pre-tick 값으로 수정하고 회귀 assertion을 추가했다. `game-sound-updated.log`가 수정 후 통과 기록이다.
- 최신 실제 Web `web-feedback-latest/feedback-check.json`: Fun02 수량0 키/툴바 차단, Native와241상태 일치, Splatting 캡처, 굴착4회/PCM peak4156, 효과음 음소거·일시정지·모드 간 상태 일치, console 오류0.
- `tools/web/play_feedback.cjs`로 재현한다. 전용 빈 QA 브라우저와 Native Fun02 JSONL을 인자로 사용하며 사용자 CDP19222는 조작하지 않는다.
- 실제 Web Fun01 툴바8개·일시정지·미니맵·재시작·전체화면·배정 및664틱/665상태/10구출 Won 통과 (`web-product-final/product-check.json`).
- compatibility index PASS-SCOPED와 기존 G11 NOT READY를 보존했다. 새 성능 인수 또는 전체120레벨 플레이 결과가 아니다.

미리보기: http://localhost:8080/preview-216/ (기존 원격 접속 주소에서는 같은 경로 사용). 1레벨 굴착 타격감/소리와 2레벨 추락 표현을 사용자가 확인할 수 있다. 메인테이너가 “모두 통과 판정입니다.”로 실제 청취·시각 인수를 완료했다. PR#220 병합과 CI40090002104 7개 성공을 확인했고 이슈를 완료로 닫았다. [#219 v0.3.2 배포 보고서](../reports/issue-219_release_v032.md)에 공개 검증을 기록한다. 공식 v0.3.1/Pages는 변경하지 않았다.
