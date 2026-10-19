# Desktop preview

Linux SDL2 어댑터로 원본 **Fun 01 — Just dig!**의 지형·입구·출구 이미지와 생성·걷기·낙하를 표시합니다. Digger·Basher·Miner·Builder·Blocker·Bomber·Climber·Floater 할당을 지원합니다. 출구 판정·승패·음향은 후속 구현입니다.

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
| 왼쪽 클릭 | 선택한 직업을 가까운 레밍에 할당 |
| Space | 일시정지/재개 |
| Up / Down | 생성 속도 +1 / -1, 레벨 초기값~99 |
| Left / Right | 카메라 16픽셀 이동 |
| Home / End | 월드 양끝 |
| Esc / 창 닫기 | 종료 |

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

`make check-desktop`은 앱을 오프라인 빌드한 뒤 `target/release/lemmings-desktop`으로 실제 SDL dummy/software 창·입력 루프·화면 readback을 원본 없이 검사합니다. 전체 SDL 합성 검사는 release 실행 파일을 사용합니다. WASM 검사는 기존 두 라이브러리만 빌드합니다.

[구현 계약·참조·X11 재현](../../mydocs/tech_insights/issue-27_native_view.md), [검증 보고서](../../mydocs/reports/issue-27_native_view.md).

## Digger

왼쪽 클릭으로 레밍에게 Digger를 할당합니다. HUD의 DIG 수량은 성공 때만 줄어듭니다. 현재 Walking/Bashing/Mining 개체가 대상이며 출구/승패와 나머지 직업은 후속 구현입니다.

재생 입력 예: `60 0 AssignDigger 0`. 기존 Pause/Resume/SetReleaseRate와 같은 control_step/sequence 순서를 사용합니다. 일시정지 중 할당은 가능하고 굴착은 진행 때 갱신합니다.

캡처의 `digger.jsonl`에는 수량·초기 Digger 플래그·매 경계 삭제 인덱스/지형 서명이 있습니다. `terrain-initial.bin`에서 삭제 이력을 재생하면 `terrain-final.bin`과 같아야 합니다. [계약과 검증](../../mydocs/tech_insights/issue-31_digger.md)을 참고합니다.

## Basher와 레벨 선택

`--level 5`는 Fun05 “You need bashers this time”을 엽니다. 현재 Fun1..30의 표준 그래픽 레벨을 선택할 수 있습니다. B 키로 Basher를 선택하고 왼쪽 클릭하면 Walking/Digging/Mining 개체에 할당하며 성공 때만 BASH 수량을 줄입니다. D 키로 Digger를 다시 선택합니다. 재생은 `166 0 AssignBasher 0` 형식입니다.

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

원본 없이 `--synthetic-climber`로 좌/우 벽과 Hoisting을 확인합니다. `climber.jsonl`은 수량과 각 ID의 영구 능력/Walking Once 애니메이션 표지를 기록합니다. DOS의 Shrugging 중 할당 버그도 유지합니다. [계약·재현·관측 범위](../../mydocs/tech_insights/issue-40_climber.md)를 참고합니다. Floater는 아래 #42에서 구현했으며 Nuke·완주·음향은 후속입니다.

## Floater / #42

`--level 2`는 Fun02 “Only floaters can survive this”를 엽니다. F키로 선택한 뒤 레밍을 클릭하면 수량 하나를 소비하고 영구 능력을 켭니다. Climber와 함께 유지되며 높은 낙하에서 원본 Umbrella 스프라이트로 펼침·잠깐 상승/정지·하강·안전 착지를 표시합니다. 재생 예는 `54 0 AssignFloater 0`입니다.

원본 없이 `--synthetic-floater`로 발판/낙차와 Bomber 상호작용을 검사합니다. `floater.jsonl`은 수량과 `[id,is_floater,float_index]`를 기록합니다. 실제 UI의 Pause/Resume/속도 입력도 기준 함수에 재생합니다. [계약·실행·검증 범위](../../mydocs/tech_insights/issue-42_floater.md)를 참고합니다. 개별8직업을 지원하며 Nuke·전체 상호작용·구출/승패/음향은 후속입니다.
