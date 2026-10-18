# G05-03 개체 이동·제거

작업 이슈: [GitHub #25](https://github.com/edwardkim/ohmylemmings/issues/25). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 근거와 범위

[#5 비교 계약](issue-5_compatibility_baseline.md), [#20 지형 조회](issue-20_terrain_mask.md), [#24 생성 요청](issue-24_startup_schedule.md)을 연결합니다. 고정 Lemmix의 [Transition/TurnAround](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L1927), [Walking/Jumping](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L2826), [Falling/Splatting](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L3210), [경계/제거](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L3379)를 정적으로 분석했습니다. [Styles.Base 메타데이터](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Styles.Base.pas#L401)의 네 행동 프레임/FootY와 DOSORIG의 FallerStartsWith3을 사용합니다. 실제 Pascal/DOS 실행 인증은 아닙니다.

TS의 [걷기](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/game-play/actions/action-walk-system.ts)에서는 높은 턱을 처음부터 전체 높이만큼 올리지만 Lemmix는 첫 점프를 2픽셀로 시작합니다. [낙하](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/game-play/actions/action-fall-system.ts)의 일반 양수 좌표 3픽셀 탐색은 참고하되 ClipY·경계·초기 누적값까지 같은 것으로 판정하지 않습니다.

## API와 안전성

[motion.rs](../../crates/lemmings-core/src/simulation/motion.rs)는 리소스 enum과 분리된 기본 Actor를 제공합니다. 코어는 no_std·무할당·I/O 없음·의존성 0개, 기존 리소스→코어 의존 하나를 유지합니다.

| API | 계약 |
|---|---|
| `ActorState` | ID·생성 iteration·발 x/y·수평 방향·행동·frame/end·fallen·제거 이유. 제거 후 마지막 상태 보존 |
| `MotionAction / MotionDirection` | Walking/Jumping/Falling/Splatting, Left/Right/Stopped. Stopped는 Splatting에만 사용 |
| `MotionRules` | FallerStartsWith3 옵션. 기본 true, Falling에 진입할 때 누적값 3 또는 0으로 초기화 |
| `Actor::from_spawn` | 원래 순서의 입구 0~4개 중 요청 index 선택, 입구+(24,14), 오른쪽 Falling·frame 0. 선택 오류/좌표 덧셈 오버플로 거부 |
| `Actor::from_state` | frame 범위·방향/행동·fallen·제거 상태 일관성 검증. i32 좌표와 생성 ID/iteration 보존 |
| `Actor::advance` | 불변 TerrainMask에서 1진행 틱. 산술 오류 시 전체 Actor 보존, 이미 제거된 개체는 무효 변경 |
| `MotionOutcome` | 객체 검사 가능 여부·새 제거 이유·새 Splatting 전환. 객체 자체는 처리하지 않음 |
| `step_actors / ActorTick` | 호출자 소유 슬라이스의 엄격한 ID 순서 확인, 전체 다음 상태 사전 검사 후 갱신/통지. 오류는 배치 전체·지형·콜백을 보존 |

배치는 제거된 기록도 ID 순서에 포함하고 갱신/콜백은 건너뜁니다. 활성 개체 수·새 제거 수·객체 검사 수·새 충돌 사망 전환 수를 반환합니다. 사전 검사와 갱신의 두 패스로 무할당 오류 보존을 구현하며 비용은 O(개체 수), 추가 메모리는 O(1)입니다. 지형과 actor 저장소를 콜백에서 동시에 가변 대여할 수 없어 사전 검사 조건이 유지됩니다. 콜백 panic의 외부 효과는 되돌리지 않습니다. 향후 직업/객체 상호작용은 단일 개체 API와 별도 순서 제어를 사용합니다.

발 좌표 연산은 checked i32, ClipY의 비교용 탐색 좌표는 i64를 사용합니다. 표현할 수 없는 이동/낙하 카운터는 명시적 오류이며 원문 Integer의 오버플로와 같다고 판정하지 않습니다. Core는 외부 자원·플랫폼 시계·렌더링에 의존하지 않습니다.

## 이동·프레임 계약

- 일반 프레임 증가 후 행동을 처리합니다. Walking 8/Falling 4는 Loop, Jumping 1/Splatting 16은 Once입니다. 전환은 frame/end/fallen을 초기화하고 단순 방향 전환은 프레임을 유지합니다. 이 값은 [#16 리소스 해석](issue-16_lemming_sprites.md)의 프레임/발 기준과 대응합니다.
- 걷기는 먼저 수평 1픽셀을 이동합니다. 발 위치의 지형과 위쪽 7회 탐색에서 높이 0~2는 직접 올라가고 3~6은 2픽셀 상승한 Jumping, 7은 방향 전환입니다. Jumping은 이후 틱마다 최대 2픽셀 상승하고 빈 공간에서 Walking으로 돌아갑니다.
- 아래 1~3픽셀에서 지형을 만나면 걷고, 4픽셀 빈틈은 그만큼 이동 후 Falling으로 전환합니다. 초기 fallen은 3/0이며 낙하 틱은 최대 3픽셀 이동합니다. 온전한 3픽셀 이동만 fallen에 3을 더합니다. 부분 착지 이동은 더하지 않고 기존 fallen>60을 검사합니다.
- 60 이하는 Walking, 61 이상은 Splatting입니다. 새 Splatting은 수평 속도 0·frame 0·fallen 0이며 객체 검사를 허용합니다. 원문의 직접 낙하 출구 검사 순서를 위한 결과만 보존하며 실제 구출은 구현하지 않습니다. 다음 16갱신째 제거하고 그 전에는 제거 수에 포함하지 않습니다.
- Walker의 x=0..1647 검사는 전진 뒤 수행합니다. 밖으로 한 칸 이동한 좌표를 유지하면서 방향을 바꿉니다. 표시 지형 폭으로 자르지 않으며 Falling에는 같은 수평 벽 처리를 적용하지 않습니다.
- 지형 조회는 `max(y,minY)` ClipY 후 실제 TerrainMask 크기를 사용합니다. 위쪽/아래쪽 탐색의 minY도 원문 순서를 따릅니다. 네 행동의 FootY=10과 head 최소 -5에서 발 y<5이면 y=3으로 보정·방향 전환하고 Jumping은 Walking으로 전환합니다.
- 하단 163 검사는 행동별 시점에 따릅니다. 빈 지형의 y=163에서 Faller는 y=164에서 제거되고 Walker는 빈틈 처리 뒤 y=167에서 제거됩니다. 지형 영상 1584×160과 개체 경계를 같은 크기로 간주하지 않습니다.

## 독립 모델과 전수 검증

새 합성 19개 모두 Red, 기존 코어 79개/리소스 181개 통과 후 Green 코어 98개를 확인했습니다. 탄생/복원·모든 잘못된 프레임·방향/제거 일관성·턱/빈틈·프레임·60/61·음수 ClipY·상하좌우 경계·극단/산술 오류·배치 오류 보존·지연 제거를 검사합니다.

[독립 Python 모델](../../crates/lemmings-core/tests/fixtures/motion_reference.py)은 좌표 집합과 첫 지형/빈 공간 탐색으로 기대 결과를 계산합니다. Rust 출력에서 기대값을 만들지 않습니다. 3×3 지형의 모든 512패턴 × 두 Faller 규칙 × 위치 15개 × 유효 행동/방향/프레임/누적값 21조합의 **322560전이**를 검사합니다. Walking frame 0/7, Jumping 0, Falling 0/3와 fallen 0/60/61, Splatting 0/14/15를 포함합니다.

각 입력과 전체 이동 상태·결과를 직렬화하여 지형/규칙당 315전이의 1024개 FNV-1a 서명을 [테스트 golden](../../crates/lemmings-core/src/simulation/motion/golden.rs)과 대조합니다. 전체 서명은 **`a04dc778204e5a15`**입니다. ID/생성 iteration 보존은 각 사례에서 별도 검사합니다. Golden은 Rust 구현 전에 계산했으며 Python 명령이 모든 그룹을 다시 계산·확인합니다.

통합 fixture는 96×160의 평지·높은 착지·빈 맵·턱·벽·상단 지형 6개 × 무입력/RR 변경/Pause 3종 × PauseGlitch 2종의 36사례입니다. 상한 4·입구 2개·RR50에서 시작해 실제 StartupScheduler 요청을 생성하고 같은 틱의 Actor 배치 갱신에 연결합니다. 첫 생성 54의 스냅샷은 발 (24,17)·fallen 6·frame 1입니다.

총 20초의 한 프레임·정수 60Hz·불규칙 나노초의 **108회 재생·36720제어 호출**에서 전체 시작/개체 이벤트와 매 호출 상태가 같습니다. 한 렌더 패턴의 36사례는 17040개 Actor 갱신이며 독립 Python 통합 서명 **`4821a442b91d9b8b`**와 일치합니다. 테스트의 상태 바이트 형식은 양쪽 모델 코드에 명시하며 전체 trace-v1 직렬화나 실제 플랫폼/DOS 실행으로 표시하지 않습니다.

## 재현 명령

루트의 `make check`는 원본·Node·네트워크 없이 단위/문서 테스트·포맷·Clippy·네이티브/WASM 빌드를 실행합니다. `make check-python`은 도구 unittest 8개에 이어 독립 모델/golden도 검증하므로 같은 GitHub Actions Python 작업에서 실행됩니다. 전수·통합 테스트는 다음과 같습니다.

```sh
cargo test --locked --offline -p lemmings-core simulation::motion::tests::exhaustive_motion_matches_independent_per_terrain_and_rule_goldens -- --exact --nocapture
cargo test --locked --offline -p lemmings-core simulation::motion::tests::startup_and_actor_replays_match_python_and_three_render_patterns -- --exact --nocapture
```

독립 기대값을 다시 계산하고 저장된 golden 전체를 확인합니다. 이 기본 명령은 파일을 수정하지 않습니다.

```sh
python3 crates/lemmings-core/tests/fixtures/motion_reference.py
```

로컬 레퍼런스를 준비한 환경에서는 원문 식별도 확인합니다. 새 원본 파일/패키지를 설치하지 않습니다.

```sh
python3 tools/reference_assets.py verify --manifest assets/manifests/reference_sources.json
python3 - <<'PY'
from pathlib import Path
import hashlib, json
m=json.loads(Path('assets/manifests/reference_sources.json').read_text())
names={'lemmix__src__Game.pas','lemmix__src__Dos.Consts.pas',
       'lemmings-ts__src__game__game-play__actions__action-walk-system.ts',
       'lemmings-ts__src__game__game-play__actions__action-fall-system.ts'}
matched=0
def check(data,entry):
    assert len(data)==entry['size']
    assert hashlib.sha256(data).hexdigest()==entry['sha256']
    assert hashlib.sha1(b'blob '+str(len(data)).encode()+b'\0'+data).hexdigest()==entry['git_blob_sha1']
for entry in m['files']:
    if entry['name'] not in names: continue
    check((Path(m['local_directory'])/entry['name']).read_bytes(),entry)
    matched+=1
assert matched==4
sprites=json.loads(Path('assets/manifests/lemming_sprites.json').read_text())
entry=next(v for v in sprites['additional_references'] if v['path']=='src/Styles.Base.pas')
check(Path('assets/local/issue-16-sources/Styles.Base.pas').read_bytes(),entry)
print('PASS: four motion sources and Styles.Base size/SHA-256/Git blobs')
PY
```

## 완료 범위와 다음 단계

로컬 make check는 Rust 단위 279개·문서 테스트 27개(기존 compile-fail 2개 포함)·Python 8개·포맷·Clippy·빌드·WASM 컴파일을 통과했습니다. 세 sh 블록을 그대로 재현했고 CI와 같은 make check-python의 독립 golden 검사도 통과했습니다. 게시·실제 CI·원격 tree를 확인합니다.

직업/영구 능력·객체/출구·Blocker/강철·지형 변형·성패/전체 시계 종료·실제 음원/화면·리소스 연결·전체 trace/실제 DOS 실행은 후속입니다. G05 전체·COMP-06은 완료 처리하지 않습니다. 다음 후보는 별도 이슈의 **G05-04 상태 추적·재생·기준 실행 대조**입니다.

- [결과 보고서](../reports/issue-25_actor_motion.md), [진행 기록](../working/issue-25_actor_motion.md), [작업일지](../orders/20261018.md#issue-25)
