# G07-07 Climber 계약과 재현

이슈: [#40](https://github.com/edwardkim/ohmylemmings/issues/40), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 고정 기준과 영구 능력

Lemmix `40e9bc34451f0e9127fd53b290a3877e700d143d`의 [Game.pas](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas) 기존43함수에 AssignClimber·HandleClimbing·HandleHoisting의 원문 본문을 더한46함수를 FPC3.2.2로 실행합니다. 기존 두 inline 변수 문법 변환 외에 원문을 바꾸지 않습니다. Climbing/Hoisting은8프레임·16×12·FootX8/FootY12이며 각각 Loop/Once입니다.

- 살아 있고 수량이 남으며 이미 Climber가 아니고 Blocking/Splatting/Exploding이 아니면 수량1개를 소비하고 영구 능력을 켭니다. Walking/Jumping/Falling·기존 굴착/건설·OhNo 중에도 할당할 수 있습니다. 행동·좌표·프레임·Bomber 타이머는 일반적으로 유지합니다. 다른 직업으로 전환해도 능력이 남습니다.
- `Dos.Consts.pas`의 DOSORIG 메커니즘에는 AssignClimberShruggerActionBug가 포함됩니다. Shrugging 중에는 action만 Walking으로 바뀌고 프레임·EndOfAnimation·Once 메타데이터가 남습니다. 코어는 `walking_once`로 이 상태를 보존합니다. 합법적인 다음 전환/방향 전환은 새 애니메이션 정보를 선택하므로 이 표지를 지웁니다. `MotionRules.assign_climber_shrugger_bug`는 기본 true이며 false에서는 일반 능력 할당만 수행합니다.
- Walking은 이동할 다음 열에서7픽셀 이상 벽을 만나면 Climber는 방향·높이를 유지한 채 Climbing0으로 진입하고, 능력이 없으면 반전합니다. 기존3~6픽셀 Jumping 규칙은 유지합니다.

## 등반과 경계 순서

행동 처리 전에 애니메이션 프레임을 진행합니다. Climbing0..3은 `(x,y-7-frame)`을 minY0으로 검사합니다. 비어 있으면 `y-frame+2`로 이동하고 Hoisting0으로 전환한 뒤 레벨 상단 경계를 확인합니다. Climbing4..7은 Y를 먼저1 줄이고 머리 상단이-5보다 작거나 `(x-direction,y-8)`의 minY-8 픽셀이 있으면 반전→Falling→새 방향으로 X2픽셀 순서입니다. Falling의 시작 낙하 카운터3 규칙을 유지합니다. Climbing의 객체 확인은 매 틱 허용합니다.

Hoisting은 진행된 frame1..4에서 Y를2씩 줄이고 상단 경계를 확인합니다. frame5/6/7은 객체 확인을 생략하며, Once 애니메이션 끝에서는 Walking0으로 전환하고 상단/객체 확인을 허용합니다. 전환 직후의 frame0과 다음 틱의 frame1을 같은 진행으로 세지 않습니다. 능력은 Hoisting 종료·반전/낙하·다른 직업·OhNo/폭발 후 기록에도 남습니다.

코어는 기존 불변 지형 `advance`와 모든 직업/Bomber 조정기를 사용합니다. 산술 오류는 개체를 복사한 상태에서 검사해 원래 기록을 보존합니다. Climbing/Hoisting의 능력 누락과 Walking 외 행동의 `walking_once`는 복원 시 오류입니다. 코어에 리소스·렌더링·I/O 의존성을 추가하지 않습니다.

## SDL·재생과 실행

C/E/K/U/M/B/D는 Climber/Bomber/Blocker/Builder/Miner/Basher/Digger 선택입니다. 클릭은 안정된 ID로 기록하며 `80 0 AssignClimber 0`으로 재현합니다. 일시정지 중 능력 할당은 가능하지만 이동/애니메이션은 재개 후 진행합니다. 원본 스프라이트의 별도 Climbing/Hoisting 방향·발 앵커를 사용하며 능력 표시를 위해 픽셀 색을 바꾸지 않습니다.

```sh
source assets/local/dev-tools/env.sh
cargo run --release --locked --offline -p lemmings-desktop -- --assets assets/local/dos-original --level 4
python3 tools/lemmix/build_climber_reference.py assets/local/my-climber-reference --compiler /usr/lib/fpc/3.2.2/ppcx64 --units /usr/lib/fpc/3.2.2/units/x86_64-linux/rtl
python3 tools/check_climber.py --binary target/release/lemmings-desktop --reference assets/local/my-climber-reference/reference
SDL_VIDEODRIVER=x11 python3 tools/verify_climber.py assets/local/my-climber-native --reference assets/local/my-climber-reference/reference --xdotool /usr/bin/xdotool
make check
```

`--synthetic-climber`는 자체 지형의 좌/우 벽을 사용하며 원본 없이 실행합니다. `climber.jsonl`은 수량과 `[id,is_climber,walking_once]`를 기록합니다. 실제 불리언·ID·행동 불변식을 검사하고 기존 여섯 직업의 상태·모든 경계 지형/객체 맵 변화를 함께 대조합니다. CI는 실제 실행에서 기록한 golden과 어댑터 해시를 검사하며 갱신에는 실제 `--reference`가 필요합니다. 호스트는 그래픽/음향/기록기 등을 스텁 처리하며 전체 Lemmix 실행 인증은 아닙니다.

## 실제 DOS 관측

원본 Fun04 “Now use miners and climbers”의10명/RR1/5분·Climber10/Miner1·scroll576을 레벨 헤더·preview·실제 숫자로 확인합니다. 기존 #33에서 검증한 F2 후보 `JLDLCINECR`로 진입합니다. 네이티브 camera560과 DOS의16픽셀 테두리를 대응시키며 입력80의(679,61)은 DOS counter81의Walking(695,77)에 해당합니다. F3 선택값2를 관측한 뒤 Space로 할당하고 포인터를 HUD로 옮깁니다.

EXE33cf는DS:6a의 수량을 감소시키고33d3는 actor+0x24(첫 개체DS:ac)에1을 씁니다. 실제 관측은 수량10→9·영구 능력 bit0·Climbing0x20·돌출 지형 반전 후 Falling0x4와 후속 Walking을 확인합니다. 이 Fun04 단일 능력 경로는 Hoisting에 이르기 전에 머리 충돌로 반전합니다. Hoisting의 프레임/높이 계약은 합성·실제 원문 실행으로 확인하며 이 원본 경로에서 발생했다고 주장하지 않습니다.

```sh
python3 tools/observe_climber_dos.py --dosbox /usr/bin/dosbox --xdotool /usr/bin/xdotool --output assets/local/my-climber-dos --seed assets/local/issue-34-dos-proof-1/bootstrap/drive/RUSSELL.DAT --seed-sha256 04462851c5edc273303b6fd19b8ee3c47ef722b9c1803dfd5adbd0229041e313
python3 tools/observe_climber_dos.py --dosbox /usr/bin/dosbox --xdotool /usr/bin/xdotool --output assets/local/my-climber-dos-repeat --seed assets/local/issue-34-dos-proof-1/bootstrap/drive/RUSSELL.DAT --seed-sha256 04462851c5edc273303b6fd19b8ee3c47ef722b9c1803dfd5adbd0229041e313
python3 tools/compare_climber_dos.py --recording assets/local/my-climber-dos/run assets/local/my-climber-dos-repeat/run --native assets/local/my-climber-native/F04-single --output assets/local/my-climber-comparison
```

DOSBox0.74-3·VGA/PC2·fixed10000cycles·같은 seed/정규화 설정/mapper/컨트롤러·서로 다른 녹화 경로를 요구합니다. #34의 실제256바이트 seed offset0만9→0으로 바꾸며 원본50개/EXE·에뮬레이터·게임 메모리는 수정하지 않습니다. `/proc` 자식 메모리 읽기만 수행합니다. 전체320×160·정확한2배 픽셀/스캔라인·RGB565 코드 정규화만 사용하며 커서 마스크·위치 이동·색 허용 오차는 없습니다.

비동기 숫자 폴링과 화면 앵커는 전체 원자적 DOS 상태/입력 틱/속도·전체 HUD·모든 객체/구출/승패·음향·G07 전체를 인증하지 않습니다. Floater/Nuke는 후속입니다. [보고서](../reports/issue-40_climber.md)와 [manifest](../../assets/manifests/climber.json)에 최종 실행·제외 목록과 출처/해시를 기록합니다.
