# G05-02 시작 시계·입구·생성 스케줄

작업 이슈: [GitHub #24](https://github.com/edwardkim/ohmylemmings/issues/24). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 근거와 범위

[#5의 시간/입력 계약](issue-5_compatibility_baseline.md#입력상태-기록-프로토콜)과 [#23 골격](issue-23_fixed_ticks.md)을 이어 구현합니다. 고정 Lemmix의 [초기화·입구 순서](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L1477), [갱신·시계](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L3408), [PauseGlitch](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L3930), [생성](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L4001) 원문을 기준으로 삼습니다. Pascal 엔진이나 DOS 실행에서 계측한 결과가 아닙니다.

고정 TS의 [생성 코드](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/game-play/lemming-manager.ts#L138)는 `104-RR` 간격과 첫 입구만 사용해 이번 계약과 다릅니다. Lemmix의 초기화/시계/정지/카운트다운을 정적으로 확인하고 독립 정수식으로 기대 결과를 계산했습니다. 기존 소스 46개, 관련 원문 2개의 크기·SHA-256·git blob을 [고정 매니페스트](../../assets/manifests/reference_sources.json)와 대조했습니다.

## API와 순서

[StartupScheduler](../../crates/lemmings-core/src/simulation/startup.rs)는 기존 TickStepper·InputSchedule을 재사용합니다. `no_std`·무할당·I/O 없음·코어 의존성 0개와 리소스→코어 의존 하나를 유지합니다.

| API | 계약 |
|---|---|
| `StartupConfig / StartupConfigError` | 분 단위 시간 u16·총 생성 수 u32·초기 RR 0~99·입구 0~4·PauseGlitch 옵션. RR 다음 입구 수 순으로 검증 |
| `StartupScheduler::state / step_state` | 다음 경계 직전의 시계/RR/입구/생성 상태와 별도의 기존 제어/진행/정지 카운터. 호출자가 함께 기록 |
| `StartupCommand::SetReleaseRate(u8)` | 초기 RR~99만 수락. 현재 카운트다운은 유지하고 다음 재설정에서 새 RR 사용 |
| `StartupEvent::Input` | 모든 경계 입력의 control_step·sequence·accepted를 순서대로 통지. 범위 밖 RR은 false, 후속 입력/진행은 계속 처리 |
| `StartupEvent` | 입력 전부 → 시계 갱신 → LetsGo/EntranceSound/EntrancesOpened → 생성 요청 → Tick. 실제 소리·화면·actor 갱신은 호출자 책임 |
| `SpawnRequest` | 0기반 안정된 actor_id·원래 입구 목록의 entrance_index·생성 iteration. 좌표/actor를 할당하지 않음 |

초기 잔여 시간은 `분×60`, clock_frame/glitch/released는 0, 입구는 닫힘, 카운트다운은 20입니다. 정지하지 않은 진행마다 clock_frame을 증가시키고 17에서 0으로 돌리며 잔여 초를 1 줄입니다. 시간 0은 포화하며 자동 종료하지 않습니다. 전체 시간 초과·성공/실패는 상위 게임 단계가 판단합니다.

진행 틱 15는 LetsGo, 34는 입구음, 35는 입구 개방입니다. 개방한 같은 틱부터 20을 감소시켜 첫 생성은 54입니다. 0에 도달하면 먼저 `floor((99-RR)/2)+4`로 재설정하고, 입구가 존재하며 생성 상한 미만일 때 요청을 냅니다. 입구 0개·상한 도달 뒤에도 재설정은 유지합니다. 총수 0은 생성하지 않습니다. 입구 순서는 1개 AAAA·2개 ABBA·3개 ABCB·4개 ABCD이며 ID는 실제 요청마다 1씩 증가합니다. u32 최대 총수에서 마지막 ID는 u32::MAX-1입니다.

초기 RR보다 낮거나 99보다 높은 명령은 거부합니다. Lemmix UI의 증가/감소 클램핑과 구분한 명시적 경계 명령 정책이며 키/버튼 어댑터는 후속입니다. RR을 경계 53에서 99로 바꾸면 진행 54의 재설정에 반영하지만 경계 54에서 바꾸면 이미 54에 설정한 카운트다운을 유지합니다.

PauseGlitch가 켜진 상태에서 정지로 전환할 때 이전 iteration이 34 미만이고 glitch가 0이면 `34-iteration`을 저장합니다. 정지 중에는 진행/시계/glitch/생성을 바꾸지 않고 입력과 제어 카운터만 처리합니다. 재개 뒤 저장한 횟수만큼 glitch를 줄이면서 시계 증가를 건너뜁니다. 입구/생성은 논리 iteration에 따릅니다. 반복 Pause는 다시 무장하지 않습니다. 경계 0의 실시간 Pause는 34를 저장하지만 Lemmix 리플레이 헤더 읽기는 0~33으로 제한하므로 리플레이 파일 동작까지 같다고 판정하지 않습니다.

오류/입력 순서는 기존 stepper가 보장합니다. control_step 한계는 콜백 전 거부하여 두 상태를 보존합니다. 콜백 panic의 외부 효과는 되돌리지 않습니다. 시작 전체 상태 복원·강제 진행·Nuke·실제 음악/효과음·객체 프레임·actor·리소스 연결은 후속 범위입니다. 한 경계 비용은 O(log N + 해당 입력 수), 추가 메모리는 O(1)입니다.

## 경계 fixture와 전수 모델

새 17개 중 16개 Red·초기 상태 1개 및 기존 코어 62개/리소스 181개 통과에서 Green 코어 79개를 확인했습니다. 모든 범위 밖 설정·최대 시간/총수·17프레임 시계·0 포화·입구 없는 경우·생성 상한·RR 명령 순서/거부·초기/반복/임계 Pause·카운터 한계 무통지를 검사합니다.

| fixture | 기대 결과 |
|---|---|
| F00 / RR 50 | LetsGo 15·입구음 34·개방 35·첫 생성 54·그다음 82 |
| F02 / 경계 54에서 RR 98 또는 99 | 생성 틱 54·82·86·90. 기존 28 카운트다운 유지, 다음부터 간격 4 |
| 경계 53에서 RR 99 | 생성 54·58. 같은 경계의 낮은 RR/255 거부는 이전 수락 값을 보존 |
| F03 / 경계 16 Pause·34 Resume | 정지 18제어 호출. 논리 첫 생성은 54, 제어 호출 완료 수는 72 |
| F03 / PauseGlitch 비교 | 재개 때 18번 시계 억제. 진행 54에서 켜짐은 잔여 298초/프레임 2, 꺼짐은 297초/프레임 3 |

초기 RR 0~99 × 입구 0~4 × glitch 꺼짐/켜짐 × 무입력/RR99/RR98/초기 정지의 **4000개** 설정을 340제어 호출씩 검사했습니다. 시간은 5분·상한은 10입니다. 독립 모델은 iteration을 절대 제어 호출 수로 계산하고 시계는 누적 진행에서 억제 구간을 빼며 생성은 절대 마감 틱의 정수 나눗셈으로 계산합니다. 카운트다운을 감소시키는 두 번째 구현을 기대값으로 사용하지 않습니다.

각 설정을 총 20초인 한 프레임·정수 나노초 60Hz·0/1ns/불규칙 긴 프레임의 세 패턴으로 재생했습니다. **12000회·4080000번 제어 호출**의 전체 이벤트와 매 호출 뒤 두 상태 스냅샷이 독립 모델과 같습니다. Rust 이전에 작성한 Python 모델의 FNV-1a 64비트 서명 **`81365042e5aa5fed`**도 일치합니다. 실제 플랫폼 렌더링·전체 게임 trace·DOS 호환성 인증을 뜻하지 않습니다.

## 재현 명령

원본·Node·네트워크 없이 전체 검사는 `make check`로 실행합니다. 전수 대조는 다음 명령으로 서명까지 확인합니다.

```sh
cargo test --locked --offline -p lemmings-core simulation::startup::tests::exhaustive_startup_replays_match_closed_form_model_and_python_stream -- --exact --nocapture
```

독립 모델은 설정 순서 RR→입구→glitch→시나리오로 스트림을 연결합니다. 헤더 `[7, RR, 입구 수, glitch, 시나리오]` 뒤 입력(0)·LetsGo(1)·입구음(2)·개방(3)·생성(4)·Tick(5)·스냅샷(6)의 태그를 씁니다. 정수 필드의 너비·LE 순서와 닫힌식은 아래에 명시합니다. 전체 trace-v1 직렬화가 아닌 테스트 전용 형식입니다.

```sh
python3 - <<'PYMODEL'
import json,struct

def model(rr, entrances, glitch, scenario):
    stream=bytearray((7,rr,entrances,glitch,scenario))
    interval0=(99-rr)//2+4
    candidate={1:99,2:98}.get(scenario,rr)
    accepted=rr<=candidate<=99
    interval1=(99-(candidate if accepted else rr))//2+4
    next_initial=54+interval0
    previous_iteration=previous_released=0
    orders=((0,0,0,0),(0,0,0,0),(0,1,1,0),(0,1,2,1),(0,1,2,3))
    for control in range(1,341):
        boundary=control-1
        paused=scenario==3 and 16<=boundary<34
        iteration=min(control,16) if scenario==3 and control<=34 else control-(18 if scenario==3 else 0)
        if scenario in (1,2) and boundary==54:
            stream.extend(struct.pack('<BQIB',0,boundary,0,accepted))
        if scenario==3 and boundary in (16,34):
            stream.extend(struct.pack('<BQIB',0,boundary,0,1))
        rate=candidate if scenario in (1,2) and boundary>=54 and accepted else rr
        lost=min(max(iteration-16,0),18) if scenario==3 and glitch else 0
        clock_ticks=iteration-lost
        remaining_glitch=max(34-iteration,0) if scenario==3 and glitch and boundary>=16 else 0
        if iteration<35: countdown=20
        elif iteration<54: countdown=54-iteration
        elif iteration<next_initial: countdown=next_initial-iteration
        else: countdown=next_initial+((iteration-next_initial)//interval1+1)*interval1-iteration
        opportunities=0 if iteration<54 else 1+(0 if iteration<next_initial else 1+(iteration-next_initial)//interval1)
        released=min(10,opportunities) if entrances else 0
        if iteration!=previous_iteration:
            if iteration in (15,34,35): stream.append({15:1,34:2,35:3}[iteration])
            if released>previous_released:
                stream.extend(struct.pack('<BIBQ',4,previous_released,orders[entrances][previous_released%4],iteration))
            stream.extend(struct.pack('<BQ',5,iteration))
        stream.extend(struct.pack('<BQQBIBBBBBI',6,control,iteration,paused,
            max(300-clock_ticks//17,0),clock_ticks%17,remaining_glitch,rate,countdown,iteration>=35,released))
        previous_iteration,previous_released=iteration,released
    return stream

if __name__=='__main__':
    digest=0xcbf29ce484222325
    cases=0
    for rr in range(100):
        for entrances in range(5):
            for glitch in range(2):
                for scenario in range(4):
                    for byte in model(rr,entrances,glitch,scenario):
                        digest=((digest^byte)*0x100000001b3)&((1<<64)-1)
                    cases+=1
    assert cases==4000 and digest==0x81365042e5aa5fed
    print(json.dumps({'scenarios':cases,'replays':cases*3,'control_calls':cases*340*3,'fnv1a64':f'{digest:016x}'}))
PYMODEL
```

원문 사본이 준비된 환경에서는 다음으로 관련 고정 소스를 재확인합니다.

```sh
python3 tools/reference_assets.py verify --manifest assets/manifests/reference_sources.json
python3 - <<'PYREF'
from pathlib import Path
import hashlib, json
manifest=json.loads(Path('assets/manifests/reference_sources.json').read_text())
names={'lemmix__src__Game.pas', 'lemmings-ts__src__game__game-play__lemming-manager.ts'}
matched=0
for entry in manifest['files']:
    if entry['name'] not in names: continue
    data=(Path(manifest['local_directory'])/entry['name']).read_bytes()
    assert len(data)==entry['size']
    assert hashlib.sha256(data).hexdigest()==entry['sha256']
    assert hashlib.sha1(b'blob '+str(len(data)).encode()+b'\0'+data).hexdigest()==entry['git_blob_sha1']
    matched+=1
assert matched==2
print('PASS: startup sources sizes/SHA-256/Git blobs')
PYREF
```

## 완료 범위와 다음 단계

로컬 make check는 Rust 단위 260개·문서 테스트 25개(기존 compile-fail 2개 포함)·Python 8개·포맷·Clippy·빌드·WASM 컴파일을 통과했습니다. 세 sh 블록을 그대로 실행해 같은 결과를 재현했습니다. 게시·실제 CI·원격 tree를 확인합니다.

G05 전체·COMP-04~06은 진행 중/미검증으로 유지합니다. 다음 후보는 별도 이슈의 **G05-03 걷기·낙하·방향 전환·제거**이며 이후 상태 추적·기준 실행 대조, G06 리소스/플랫폼 통합으로 이어집니다.

- [결과 보고서](../reports/issue-24_startup_schedule.md), [진행 기록](../working/issue-24_startup_schedule.md), [작업일지](../orders/20261018.md#issue-24)
