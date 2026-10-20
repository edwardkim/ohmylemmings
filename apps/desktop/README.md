# Desktop preview

Linux SDL2 어댑터로 원본 **Fun 01 — Just dig!**의 지형·입구·출구 이미지와 생성·걷기·낙하를 표시합니다. Digger·Basher·Miner·Builder·Blocker·Bomber·Climber·Floater 할당과 Nuke를 지원합니다. 출구·위험물·승패·결과 화면·재시작과 SDL 음향을 지원합니다. `--difficulty fun|tricky|taxing|mayhem --level 1..30`으로 특수4개를 포함한 레벨을 선택합니다. 생략하면 Fun1입니다. 전수 완주 인증은 후속입니다.

## 준비와 실행

Ubuntu/Debian에서 SDL2 개발 라이브러리와 고정 Rust 도구 체인을 준비하고 저장소 루트에서 실행합니다.

```sh
sudo apt-get update
sudo apt-get install --no-install-recommends -y libsdl2-dev
cargo fetch --locked
python3 tools/reference_assets.py fetch
python3 tools/reference_assets.py verify
cargo run --release --locked --offline -p lemmings-desktop -- --assets assets/local/dos-original
```

최초 fetch 이후 Cargo 명령은 오프라인으로 실행합니다. Release 빌드를 권장합니다. 원본 없이 SDL2를 확인하려면 마지막 명령의 앱 인자를 `--synthetic`으로 바꿉니다.

## 조작

| 키 | 동작 |
|---|---|
| F / C / E / K / U / M / B / D | Floater / Climber / Bomber / Blocker / Builder / Miner / Basher / Digger 선택 |
| N | 생성 중지 후 기존 레밍의 순차 Nuke |
| 왼쪽 클릭 | 선택한 직업을 가까운 레밍에 할당 |
| Space | 일시정지/재개 |
| Up / Down | 생성 속도 +1 / -1, 레벨 초기값~99 |
| Left / Right | 카메라 16픽셀 이동 |
| Home / End | 월드 양끝 |
| Esc / 창 닫기 | 종료 |
| 결과 화면 Enter / 왼쪽 클릭 | 실패 시 재시도, 성공 시 다음 레벨 |
| 결과 화면 R | 현재 레벨 다시 시작 |
| 결과 화면 오른쪽 클릭 | 종료 |

`--help`에서 실행 옵션을 확인합니다. `--steps 340`은 제어 호출 340회 후 종료하며, Pause 중에도 제어 호출은 증가합니다. `--fast`는 `--steps`를 지정한 검사에서만 벽시계 대기를 생략합니다.

## 실행 간격

원본 기본값 `--pacing dos`는 고정 DOSBox/VGA 메뉴 PC2 기준의 `dos-vga-5` 간격(약 14.017호출/초)입니다. 기존 17Hz 실행은 `--pacing reference`로 선택하며 합성 실행의 기본값입니다. 게임 시계는 두 경우 모두 17호출마다 1초입니다. 같은 호출 번호/입력에서는 같은 상태와 그림을 만듭니다. `run.json`에 실제 pacing·분자/분모를 기록합니다. 다른 DOS 판본/하드웨어·-x 경로까지 같은 속도라고 가정하지 않습니다.

[#30 보정·관측 범위](../../mydocs/tech_insights/issue-30_dos_differences.md)를 참고합니다.

## 입력 기록과 화면 캡처

`--capture assets/local/my-native-run`은 새 디렉터리에 초기/매 호출 상태 `trace.jsonl`, 재생할 `inputs.txt`, 실행 정보 `run.json`과 주요 경계/입력 직후의 RGB·SDL readback PPM을 기록합니다. 원본 파생 캡처는 `assets/local/` 아래에 두며 기존 결과를 덮어쓰지 않습니다.

`--replay INPUTS_FILE`은 `CONTROL_STEP SEQUENCE Pause|Resume|SetReleaseRate [RATE]` 형식의 정렬된 입력을 재생합니다. 재생 중에는 카메라와 종료 키를 사용할 수 있습니다. Space·RR은 기록된 입력을 따릅니다.

## 검증

```sh
make check
```

`make check-desktop`은 앱을 오프라인 빌드한 뒤 `target/release/lemmings-desktop`으로 실제 SDL dummy/software 창·입력 루프·화면 readback을 원본 없이 검사합니다. 전체 SDL 합성 검사는 release 실행 파일을 사용합니다. WASM 검사는 core/resources/session/game/web 라이브러리를 빌드합니다. 전체 make check의 새 브라우저 검사 준비는 [웹 안내](../web/README.md)를 따릅니다.

[구현 계약·참조·X11 재현](../../mydocs/tech_insights/issue-27_native_view.md), [검증 보고서](../../mydocs/reports/issue-27_native_view.md).

## Digger

왼쪽 클릭으로 레밍에게 Digger를 할당합니다. 기술 패널의 Digger 수량은 성공 때만 줄어듭니다. 현재 Walking/Bashing/Mining 개체가 대상입니다. 출구/구출은 #48에서 구현했으며 승패는 후속입니다.

재생 입력 예: `60 0 AssignDigger 0`. 기존 Pause/Resume/SetReleaseRate와 같은 control_step/sequence 순서를 사용합니다. 일시정지 중 할당은 가능하고 굴착은 진행 때 갱신합니다.

캡처의 `digger.jsonl`에는 수량·초기 Digger 플래그·매 경계 삭제 인덱스/지형 서명이 있습니다. `terrain-initial.bin`에서 삭제 이력을 재생하면 `terrain-final.bin`과 같아야 합니다. [계약과 검증](../../mydocs/tech_insights/issue-31_digger.md)을 참고합니다.

## Basher와 레벨 선택

`--level 5`는 Fun05 “You need bashers this time”을 엽니다. `--difficulty tricky --level 1`처럼 네 난이도의 일반 레벨을 선택할 수 있습니다. B 키로 Basher를 선택하고 왼쪽 클릭하면 Walking/Digging/Mining 개체에 할당하며 성공 때만 BASH 수량을 줄입니다. D 키로 Digger를 다시 선택합니다. 재생은 `166 0 AssignBasher 0` 형식입니다.

`--synthetic-basher`는 원본 없는 양방향 굴착 검사 장면입니다. 캡처의 `basher.jsonl`과 공통 지형 삭제 기록, `config.json`·`objects.bin`·`bash-masks.bin`으로 기준 실행을 재현합니다. [Basher 계약·검증](../../mydocs/tech_insights/issue-32_basher.md)을 참고합니다.

## Miner

`--level 4`는 Fun04 “Now use miners and climbers”를 엽니다. M 키로 Miner를 고르고 왼쪽 클릭하면 Walking/Digging/Bashing 개체에 할당하며 성공 때만 MINE 수량을 줄입니다. B/D로 다른 직업을 선택합니다. 재생 예는 `80 0 AssignMiner 0`입니다.

캡처의 `miner.jsonl`과 공통 지형 삭제 기록, `mine-masks.bin`·기존 객체/마스크/config 입력으로 기준 실행을 재현합니다. Miner의 DOS 일방통행 버그와 원본/CI 범위는 [계약·검증](../../mydocs/tech_insights/issue-33_miner.md)을 참고합니다. Climber는 아래 #40에서 구현했으며 완주 판정은 후속입니다.

## Builder / #34

`--level 7`은 Fun07 “Builders will help you here”를 엽니다. U키로 Builder를 선택하고 클릭하면 Walking/Digging/Bashing/Mining/Shrugging 개체에 할당합니다. 수량은 성공 때만 소비하며 중복 Building 할당은 거부합니다. 12개 벽돌을 소진하면 Shrugging을 거쳐 Walking으로 돌아갑니다. 재생 예는 `80 0 AssignBuilder 0`입니다.

원본 없이 `--synthetic-builder`로 양방향 계단·충돌 장면을 검사할 수 있습니다. `builder.jsonl`의 수량/남은 벽돌/`filled`, 기존 `digger.jsonl`의 `cleared`로 초기/최종 전체 지형을 재생 검증합니다. `brick-final.bin`은 벽돌 재질입니다. [계약·기준 함수·DOS 대조](../../mydocs/tech_insights/issue-34_builder.md)를 참고합니다. G07 전체·완주/승패는 후속 범위입니다.

## Blocker / #36

`--level 3`은 Fun03 “Tailor-made for blockers”를 엽니다. K키로 Blocker를 선택하고 클릭하면 Walking/Shrugging/Building/Bashing/Mining/Digging에 할당합니다. 성공 때만 BLOCK 수량을 줄이고, 방향 전환 영역과 겹치면 거부합니다. 발판이 없어지면 기존 객체 맵을 복원하고 Walking으로 돌아갑니다. 재생 예는 `80 0 AssignBlocker 0`입니다.

원본 없이 `--synthetic-blocker`를 사용할 수 있습니다. `blocker.jsonl`의 저장9칸/객체 캐시/영역 순변화와 `object-map-initial.bin`/`object-map-final.bin`을 검증합니다. [계약·기준 함수·DOS 대조](../../mydocs/tech_insights/issue-36_blocker.md)를 참고합니다. Bomber에 의한 영역 해제는 #38에서 검증했으며 완주 판정은 후속입니다.

## Bomber / #38

`--level 6`은 Fun06 “A task for blockers and bombers”를 엽니다. E키로 Bomber를 선택하고 살아 있는 레밍을 클릭하면 수량을 하나 소비하고79틱 카운트다운을 시작합니다. 기존 행동을 유지하다 OhNo/폭발로 전환하고 지형을 삭제합니다. Blocker의 방향 전환 영역도 복원합니다. 재생 예는 `80 0 AssignBlocker 0`과 `100 0 AssignBomber 0`입니다.

원본 없이 `--synthetic-bomber`로 합성 장면을 실행합니다. `bomber.jsonl`은 수량/카운트다운/입자 카운터를 기록하며 `explosion-mask.bin`과 기존 캡처 파일로 재현합니다. 원본50개를 확보한 환경은 기존 `VGALEMMI.EXE`의 입자 표를 읽습니다. EXE가 없으면 입자 그림을 생략하고, 제공한 EXE의 크기/MZ 프로필이 다르면 오류를 반환합니다. 원본 확보 시 해시 검증을 수행하세요. [계약·기준 함수·DOS 대조](../../mydocs/tech_insights/issue-38_bomber.md)를 참고합니다. Floater·Nuke·완주 판정은 후속입니다.

## Climber / #40

`--level 4`는 Fun04 “Now use miners and climbers”를 엽니다. C키로 Climber를 선택하고 레밍을 클릭하면 성공할 때만 수량을 하나 소비하고 영구 능력을 켭니다. 기존 행동을 유지하며 Walking 중 높은 벽에 닿으면 Climbing, 턱을 만나면 Hoisting으로 전환합니다. 머리 돌출 지형에는 반전·낙하합니다. 이 Fun04 경로는 Hoisting 이전에 반전하는 원본 동작을 재현합니다. 재생 예는 `80 0 AssignClimber 0`입니다.

원본 없이 `--synthetic-climber`로 좌/우 벽과 Hoisting을 확인합니다. `climber.jsonl`은 수량과 각 ID의 영구 능력/Walking Once 애니메이션 표지를 기록합니다. DOS의 Shrugging 중 할당 버그도 유지합니다. [계약·재현·관측 범위](../../mydocs/tech_insights/issue-40_climber.md)를 참고합니다. Floater는 아래 #42에서 구현했으며 Nuke는 아래 #44에서 구현했으며 완주·음향은 후속입니다.

## Floater / #42

`--level 2`는 Fun02 “Only floaters can survive this”를 엽니다. F키로 선택한 뒤 레밍을 클릭하면 수량 하나를 소비하고 영구 능력을 켭니다. Climber와 함께 유지되며 높은 낙하에서 원본 Umbrella 스프라이트로 펼침·잠깐 상승/정지·하강·안전 착지를 표시합니다. 재생 예는 `54 0 AssignFloater 0`입니다.

원본 없이 `--synthetic-floater`로 발판/낙차와 Bomber 상호작용을 검사합니다. `floater.jsonl`은 수량과 `[id,is_floater,float_index]`를 기록합니다. 실제 UI의 Pause/Resume/속도 입력도 기준 함수에 재생합니다. [계약·실행·검증 범위](../../mydocs/tech_insights/issue-42_floater.md)를 참고합니다. 개별8직업을 지원하며 전체 상호작용·구출/승패/음향은 후속입니다.

## Nuke / #44

N키를 한 번 누르면 새 레밍 생성을 멈추고 기존 개체에 순서대로79틱 타이머를 붙입니다. Bomber 수량을 쓰지 않고 기존 타이머를 유지하며 Blocker 필드 해제·폭발·입자는 기존 동작을 재사용합니다. Pause 중 N키·키 반복·재생 중 실시간 입력은 무시합니다. 재생 형식은 `120 0 Nuke`이며 ID가 없습니다.

`--synthetic-nuke`는 자체 생성 Builder 경기장을 사용합니다. `nuke.jsonl`은 실행 상태/진행/다음 순서/최대 인원을 기록합니다. 기본 DOS NukeGlitch에 따라 HUD의 최대 인원이 이미 생성된 수로 바뀌며 구출률/승패는 #52의 완주 세션에서 원래 목표/현재 분모로 계산합니다. [계약·DOS 순서 차이·재현](../../mydocs/tech_insights/issue-44_nuke.md)을 참고합니다.

현재 하단은 텍스트 HUD입니다. 원본 기술 선택 버튼·미니맵·구출률 등의 패널 통합은 G08 범위이며 원본 게임 영역320×160의 일치 검증과 구분합니다.

## 출구·구출 / #48

출구 트리거에 들어간 레밍은8프레임 애니메이션 후 제거되고 SAVED 수에 한 번 집계됩니다. 공중 Falling은 진입하지 않으며 착지와 Floating의 순서는 원본 계약을 따릅니다. `exit.jsonl`은 매 제어 경계의 `saved`와 진행 중인 `exiting` ID를 기록합니다.

원본 없이 `--synthetic-exit` 또는 `--synthetic-exit-drop`으로 검사합니다. 원본 Fun01은 Digger 한 번으로10마리 구출 경로를 재생할 수 있습니다: 입력 파일에 `60 0 AssignDigger 0`을 넣고 `--level 1 --replay 파일 --steps 700`으로 실행합니다. [계약·재현·범위](../../mydocs/tech_insights/issue-48_exit_rescue.md)를 참고합니다. 물·불·함정은 #50에서 연결했으며 승패/결과 화면·재시작·원본 패널/미니맵은 후속 절편입니다.

## 위험물·함정 / #50

원본 Fun09(`--level 9`)의 불, Fun17(`--level 17`)의 작동형 함정, Fun29(`--level 29`)의 물에서 별도 할당 없이 사망 경로를 확인할 수 있습니다. 물/불은 사망 애니메이션 후 제거되며 함정은 한 개체를 제거한 뒤 재사용을 기다립니다. Space는 개체와 함정을 함께 일시정지합니다.

`--synthetic-water`, `--synthetic-fire`, `--synthetic-trap`은 원본 없는 검사 장면입니다. `--synthetic-trap-slow|--synthetic-trap-continuous`는 경합/연속 프레임, 물/불의 `-drop`과 `-late` 접미사는 공중 진입/타이머 경계를 검사합니다. `hazards.jsonl`·`traps.bin`과 기존 입력/지형/객체 추적으로 재현합니다. [계약·명령·관측 범위](../../mydocs/tech_insights/issue-50_hazards.md)를 따릅니다. 승패/재시작·원본 하단 패널·포인터 우선순위는 후속입니다.


## 시간·구출률·승패 / #52

원본 레벨은 시간 만료 또는 전체 개체/폭발 표시 소진으로 종료합니다. 필요한 구출률만 충족한 상태에서는 계속 진행됩니다. HUD에 현재/목표 백분율과 `WON`/`LOST`를 표시하며 종료 후에는 카메라와 Esc를 사용할 수 있습니다. 결과 화면·재시작은 후속입니다.

```sh
cargo run --offline --release -p lemmings-desktop -- --synthetic-outcome win
cargo run --offline --release -p lemmings-desktop -- --synthetic-outcome loss
cargo run --offline --release -p lemmings-desktop -- --synthetic-outcome clock --steps 1200 --fast
python3 tools/check_completion.py
```

합성 종류는 `win|loss|clock|particles|zero|mixed`입니다. 기존 개체 전용 합성 fixture는 지정한 전체 관측 구간을 유지합니다. 완주 세션의 유한 `--steps` 실행은 결과가 나오면 최종 상태/화면을 기록하고 끝나며 이후 예약 입력은 적용하지 않습니다. `completion.jsonl`은 단계·종료 이유·원래/현재 총원·필요/구출 수·정수 백분율을 기록합니다. [계약·Nuke 분모·DOS 경계 차이·재현](../../mydocs/tech_insights/issue-52_level_outcome.md)을 참고합니다.

## 결과 화면·재시작 / #54

재시작은 원래 자산에서 지형·개체·수량·시계·Nuke·카메라를 다시 구성합니다. 성공 시 다음 번호로 이동하며 Fun30→Tricky1, Tricky30→Taxing1, Taxing30→Mayhem1로 이어집니다. Mayhem30 성공 뒤에는 R/종료를 선택합니다. 다음 파일이 없거나 손상되었으면 결과 화면에서 오류를 표시합니다.

```sh
cargo run --offline --release -p lemmings-desktop -- --synthetic-outcome win
# 동일 시도 재시작 후 두 번째 합성 loss 장면으로 이동:
cargo run --offline --release -p lemmings-desktop -- --synthetic-outcome win --steps 1200 --fast --result-actions restart,continue
python3 tools/check_results.py
```

`--result-actions`는 결과가 표시된 뒤 순서대로 SDL 키를 주입하는 유한 실행용 옵션입니다. `--steps`는 각 시도의 상한입니다. 첫 `--replay`의 남은 명령은 전환 시 폐기하며 이후는 수동 입력을 받습니다. 캡처는 최초 루트와 `attempt-0001/` 등 시도별 폴더, 루트 `transitions.jsonl`로 구분합니다. 자체 결과 UI이며 원본 결과 배경/글꼴·메뉴/접근 코드는 후속입니다. [계약·재현](../../mydocs/tech_insights/issue-54_results_restart.md).

## 원본 기술 패널·미니맵 (#56)

하단40픽셀은 MAIN.DAT 구간2의 VGA 패널/숫자/폰트입니다.8직업 버튼을 클릭하거나 기존 단축키로 선택하고 레밍을 클릭합니다. 초기 선택은 기존 Digger입니다. 감소/증가 버튼은 누르는 동안 control_step마다 속도를 조정하고, Pause는 토글, Nuke는 더블클릭입니다. 기존 N 키는1회 입력입니다. 미니맵의 클릭/왼쪽 드래그는 카메라를 이동합니다.

OUT/IN/TIME·직업 수량과 현재 지형/개체·카메라가 표시됩니다. 재생 중 실시간 게임 명령은 받지 않으며 카메라는 조작할 수 있습니다. 새 레벨에서 선택/누름/드래그가 초기화됩니다. [좌표·리소스·검증 계약](../../mydocs/tech_insights/issue-56_skill_panel.md)과 [결과](../../mydocs/reports/issue-56_skill_panel.md)를 참고합니다.

`make check-desktop`은 원본 없는 패널·지형 변경·재생 검사를 포함합니다. 실제 X11 검사는 독립 DISPLAY에서 `python3 tools/verify_panel.py --xdotool /usr/bin/xdotool --evidence assets/local/my-panel-x11`로 실행합니다.

## 포인터 선택·우클릭 (#58)

포인터 아래 작업 중인 레밍을 우선 선택하며, 같은 분류에서는 나중에 생성된 개체가 선택됩니다. 우클릭을 누르면 일반 개체만 선택합니다. HUD 왼쪽은 선택 대상의 행동/영구 능력과 겹침 수를 표시합니다. 시스템 커서는 게임 영역에서 십자, 선택 대상이 있으면 손 모양입니다.

DOS RightClickGlitch도 적용합니다. 일반 개체를 가리킨 뒤 우클릭을 누른 채 작업 중인 개체 위에서 왼쪽을 누르면 이전 대상에 기술을 할당합니다. 기억한 대상이 함정으로 제거된 경우에도 원래 행동이 허용하면 Blocker 수량과 필드가 적용됩니다. FocusLost/재시작은 선택 기억을 초기화합니다.

기존 안정 ID 재생에 `105 0 AssignPointer Builder 0 1 false`(우선/일반 ID)와 `105 0 AssignPointer Blocker 0 - true`(원격)를 추가했습니다. `pointer.jsonl`은 원시 논리 좌표/우클릭·선택 ID·문구를 기록합니다. [상세 계약·검증·원문 충돌 차이](../../mydocs/tech_insights/issue-58_pointer_selection.md)를 참고합니다. `make check-desktop`의 `check_pointer.py`는 원본/FPC 없이 선택 golden과8개 SDL 재생을 검사합니다.

## 공통 게임 경계

[#64](../../mydocs/tech_insights/issue-64_shared_game.md)에서 자산 해석·CPU 화면·패널·포인터 표현·레벨 전환을 [lemmings-game](../../crates/lemmings-game/README.md)으로 옮겼습니다. 데스크톱은 파일 읽기·경로 오류·SDL 입력/표시·캡처/재생을 담당합니다. 기존 CLI와 조작은 같습니다.

## 사운드 / #80

기본 음량40%, S로 켜기/음소거·장치 오류 후 재시도, PageUp/PageDown으로10%씩 조절합니다. `--mute`, `--volume 0..100`을 지원하며 `--fast`는 장치를 사용하지 않습니다. 원본 폴더의 ADLIB.DAT가 필요하고 누락/손상·장치 오류가 있어도 게임은 계속됩니다. 연습 모드는 새로 만든 음원입니다. 창 숨김/최소화·결과는 큐를 비우고, 재개 시 현재 음악을 처음부터 시작합니다. 게임 Pause 중 음악은 계속됩니다. 창 제목에서 상태를 확인합니다. [계약/검증](../../mydocs/tech_insights/issue-80_audio_output.md) · [LGPL 소스/재링크](../../third_party/README.md).

난이도/전환은 [#87 계약](../../mydocs/tech_insights/issue-87_rank_selection.md)을 따릅니다. 캡처 config와 transitions에는 difficulty/target_difficulty를 기록합니다. 기존 정수 경계/순번/명령 재생 파일은 유지하고 재생 시 같은 `--difficulty`를 지정합니다.

초기 전수 재생의 선택적 `--capture-stream`·`--elapsed-replay FILE` 계약은 [#91](../../mydocs/tech_insights/issue-91_initial_replay.md)을 참고합니다. `--fast --steps --capture`에서 전체 상태와 검증된 SDL RGB 스트림을 기록하며 기본 실행/파일 캡처는 유지합니다.
