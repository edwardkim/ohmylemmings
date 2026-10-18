# G05-01 고정 틱·입력 순서 골격

작업 이슈: [GitHub #23](https://github.com/edwardkim/ohmylemmings/issues/23). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 근거와 책임 경계

[#5의 시간·입력 계약](issue-5_compatibility_baseline.md#입력상태-기록-프로토콜)을 구현하는 첫 골격입니다. 고정 Lemmix의 [InternalUpdate](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L3408)는 iteration 0 리플레이·정지 판단·iteration 증가·생성/개체/객체 순서를 구분하고, [IncrementIteration](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L3498)은 게임 시계 17프레임을 1초로 처리합니다. 기준 계약이 이 17Hz를 채택합니다. 원작 실시간 주기를 측정한 결과가 아닙니다.

고정 TS의 [GameTimer](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/game-play/game-timer.ts#L4)는 60ms setInterval·부동소수점 변환과 before-tick 통지를 사용합니다. 이번 17Hz 정수 누적기와 같지 않습니다. Pascal/TS 실행을 대조한 것으로 표시하지 않고 고정 원문과 매니페스트의 크기·SHA-256·git blob을 확인했습니다. 기존 소스 46개도 검증했습니다.

## API와 불변식

[simulation 모듈](../../crates/lemmings-core/src/simulation.rs)은 clock·input·stepper를 분리합니다. 라이브러리 코드는 `no_std`·무할당·I/O 없음·외부 의존성 0개를 유지합니다. 리소스→코어 의존 하나와 기존 지형 API를 유지합니다.

| API | 계약 |
|---|---|
| `FixedTickClock::advance(Duration)` | 어댑터가 전달한 경과 시간에서 처리할 제어 호출 수 `u64` 반환. 누적 위상만 저장하며 벽시계·렌더링·논리 상태와 분리 |
| `phase_numerator()` | 다음 호출까지의 잔여 비율. 10억 분의 1틱 단위로 0~999999999 |
| `TimedInput<C>`·`InputCommand<C>` | `(control_step: u64, sequence: u32)`와 Pause/Resume/Custom payload. 경계 0도 처리 |
| `InputSchedule::new / at` | 키의 엄격한 오름차순 검증 후 해당 경계 슬라이스 반환. 중복·역행의 첫 인덱스 오류. 정렬/복사/소비하지 않으며 payload에 Copy/Clone 요구 없음 |
| `TickStepper::step` | 현재 경계 입력 전부를 순서대로 `StepEvent::Input` 통지 후, 최종 정지 상태가 false면 증가한 iteration의 Tick을 한 번 통지 |
| `state / from_state` | 다음 경계 직전 `StepState { control_step, iteration, paused }`. 초기 0/0/false, iteration ≤ control_step 검증 |
| `StepOutcome` | 처리한 경계 번호·실제 진행 iteration(Some)/정지(None)·입력 수. 저장된 state의 control_step은 그 다음 경계 |

시간 누적은 `scaled = elapsed.as_nanos() × 17 + phase`, `due = scaled / 10^9`, `phase = scaled % 10^9`입니다. Duration 최대값의 곱도 u128 안에 들어갑니다. due가 u64를 넘으면 TickOverflow로 위상을 보존합니다. 58823529ns는 아직 0틱, 1ns 추가 시 1틱·위상 10이며 1초는 정확히 17틱입니다. 총 나노초가 같으면 프레임 분할과 관계없이 틱 합계와 위상이 같습니다. 어댑터는 반환된 모든 호출을 처리해야 하며 이 API는 과부하 틱을 제한/폐기하지 않습니다.

입력의 sequence는 간격이 있어도 되고 다음 경계에서 다시 시작할 수 있습니다. 같은 경계의 마지막 Pause/Resume이 그 경계의 Tick 유무를 정하며 같은 명령 반복도 통지합니다. 정지 중에도 17Hz 제어 호출과 입력 전달을 계속해 Resume에 도달합니다. Custom의 유효성·성공/거부 결과와 앱 상태 갱신은 호출자가 담당합니다. 한 경계 처리 비용은 O(log N + 해당 입력 수), 추가 메모리는 O(1)입니다.

제어 호출이 성공할 때마다 control_step을 1 증가시키고 실제 Tick일 때만 iteration을 증가시킵니다. control_step 한계는 콜백 전 검사하므로 StepOverflow에서 통지·상태 변경이 없습니다. iteration ≤ control_step과 생성/복원 검증이 iteration 오버플로를 방지합니다. 입력 조회는 step+1을 계산하지 않아 u64::MAX도 조회할 수 있지만 해당 경계의 진행은 다음 스냅샷을 표현할 수 없어 거부합니다. 복원 시 앱 상태·해당 일정은 호출자가 함께 복원합니다. 콜백 panic의 외부 효과는 되돌리지 않으며 시간 누적기와 경계 처리기의 오류 보존은 각각의 객체에 적용됩니다.

## 합성 검증과 독립 모델

새 테스트 22개 중 20개 Red·초기/빈 입력 2개 통과, 기존 코어 40개·리소스 181개 통과를 확인한 후 Green 코어 62개를 확인했습니다. 나노초 임계·장시간/60ms 누적·u64/Duration 극단 값·오류 후 복구·불변 payload·키 순서·경계 0·정지/재개·입력 전부→Tick·이전/미래 경계·복원 불변식·한계 직전/한계의 무통지를 검사합니다.

0~1000ms의 모든 두 프레임 분할 **501501건**을 정수 닫힌식과 대조했습니다. 별도로 Pause/Resume/+1/×2 네 명령의 길이 0~6인 **5461개** 목록을 독립 코드 모델과 대조했습니다. 입력은 경계마다 두 개씩 배치하며 앱 값은 1에서 시작해 Custom의 +1/×2, Tick의 새 iteration 더하기를 적용합니다. 같은 경계의 비가환 입력 순서 변경이 다른 결과를 만드는 별도 fixture도 확인했습니다.

각 목록을 ①1초 한 프레임 ②정수 나노초로 분할한 60프레임 ③0/1ns·틱 경계·불규칙 긴 프레임의 세 패턴으로 재생했습니다. **16383회 재생·278511번 제어 호출**에서 전체 입력/Tick 통지 스트림·최종 카운터·정지 상태·앱 결과가 같습니다. 불규칙 패턴도 합계 정확히 1초이며 실제 모니터/브라우저 실행을 뜻하지 않습니다.

길이→코드 사전순으로 입력을 나열합니다. 입력 통지는 `[0, step, sequence, code]`, Tick은 `[1, iteration]`, 최종 상태는 `[2, control_step, iteration, paused]` 다음 앱 값의 8바이트 LE입니다. fixture의 앞쪽 수치는 모두 한 바이트 이내입니다. 한 패턴의 모든 목록 스트림을 FNV-1a 64비트로 연결한 서명은 **`5338d6ae252490ba`**입니다. Rust 구현 전 작성한 독립 Python 모델과 같습니다. 전체 게임 trace/호환성 해시가 아닙니다.

## 재현 명령

원본·Node·네트워크 없이 전체 검사는 루트의 `make check`로 실행합니다. 분할·재생 검사는 다음과 같습니다.

```sh
cargo test --locked --offline -p lemmings-core simulation::clock::tests::all_two_frame_millisecond_partitions_match_exact_total_time -- --exact
cargo test --locked --offline -p lemmings-core simulation::stepper::tests::exhaustive_replays_match_integer_model_across_three_render_patterns -- --exact --nocapture
```

독립 모델 서명은 Python 표준 라이브러리만 사용합니다.

```sh
python3 - <<'PY'
import itertools
signature=0xcbf29ce484222325
scenarios=0
for length in range(7):
    for codes in itertools.product(range(4),repeat=length):
        paused=False; iteration=0; value=1; stream=[]
        for step in range(17):
            for index,code in enumerate(codes):
                if index//2!=step: continue
                stream.extend((0,step,index%2,code))
                if code==0: paused=True
                elif code==1: paused=False
                elif code==2: value+=1
                else: value*=2
            if not paused:
                iteration+=1; value+=iteration; stream.extend((1,iteration))
        stream.extend((2,17,iteration,int(paused)))
        stream.extend(value.to_bytes(8,'little'))
        for byte in stream:
            signature=((signature^byte)*0x100000001b3)&((1<<64)-1)
        scenarios+=1
assert scenarios==5461 and signature==0x5338d6ae252490ba
print(f'{scenarios} scenarios / fnv1a64={signature:016x}')
PY
```

고정 원문의 식별 값은 [레퍼런스 매니페스트](../../assets/manifests/reference_sources.json)에 있습니다. 로컬 사본이 준비된 환경에서 크기·SHA-256·Git blob을 재확인합니다. 외부 엔진 실행은 필요 없습니다.

```sh
python3 tools/reference_assets.py verify --manifest assets/manifests/reference_sources.json
python3 - <<'PYREF'
from pathlib import Path
import hashlib, json
manifest=json.loads(Path('assets/manifests/reference_sources.json').read_text())
names={'lemmix__src__Game.pas', 'lemmings-ts__src__game__game-play__game-timer.ts'}
matched=0
for entry in manifest['files']:
    if entry['name'] not in names: continue
    data=(Path(manifest['local_directory'])/entry['name']).read_bytes()
    assert len(data)==entry['size']
    assert hashlib.sha256(data).hexdigest()==entry['sha256']
    blob=hashlib.sha1(b'blob '+str(len(data)).encode()+b'\0'+data).hexdigest()
    assert blob==entry['git_blob_sha1']
    matched+=1
assert matched==2
print('PASS: both pinned timing source sizes/SHA-256/Git blobs')
PYREF
```

## 완료 범위와 다음 단계

로컬 `make check`는 Rust 단위 243개·문서 테스트 23개(기존 compile-fail 2개 포함)·Python 8개·포맷·Clippy·빌드·WASM 컴파일을 통과했습니다. 위 세 sh 블록을 그대로 실행해 같은 결과를 재현했습니다. 구현 게시·실제 CI·원격 tree까지 확인했습니다. 최종 문서 게시·CI·#23 종료는 GitHub 이슈 기록에서 추적합니다.

Pause/Resume은 골격의 Tick 통지만 제어합니다. 원작 PauseGlitch 시계/입구 처리·종료/RR·생성·actor/직업·전체 trace 직렬화·실제 DOS/WASM 실행은 후속 범위입니다. G05 전체·COMP-04~06은 진행 중/미검증으로 유지합니다. 다음 후보는 별도 이슈의 **G05-02 시작 시계·입구·생성 스케줄**입니다.

- [결과 보고서](../reports/issue-23_fixed_ticks.md), [진행 기록](../working/issue-23_fixed_ticks.md), [작업일지](../orders/20261018.md#issue-23)
