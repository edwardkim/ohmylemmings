# G07-08 Floater 계약과 재현

이슈: [#42](https://github.com/edwardkim/ohmylemmings/issues/42), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 고정 기준과 영구 능력

Lemmix `40e9bc34451f0e9127fd53b290a3877e700d143d`의 [Game.pas](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas) 기존46함수에 AssignFloater/HandleFloating 원문을 더한48함수와 FloatParametersTable 원문을 FPC3.2.2로 실행합니다. 기존 두 inline 변수 문법 변환 외에 본문을 바꾸지 않습니다. 기존 HandleFalling/Transition/UpdateExplosionTimer의 Floater 분기도 실제 실행합니다. Umbrella는 양방향8프레임·16×16·FootX8/FootY16·Loop입니다.

살아 있고 수량이 남으며 이미 Floater가 아니고 Blocking/Splatting/Exploding이 아니면 수량1개를 소비하고 `is_floater`를 켭니다. 현재 행동·프레임·좌표·카운트다운은 바꾸지 않습니다. Climber와 독립적인 영구 능력이며 Climbing/Hoisting/OhNo에도 할당할 수 있습니다. 다른 직업·착지·Bomber 전환 후에도 능력이 남습니다. Floater 할당은 Climber의 Shrugging 버그를 새로 일으키지 않고 이미 남아 있는 Walking Once 상태도 보존합니다.

## 펼침과 이동 순서

- Falling은 **이전 틱의 Fallen이16을 넘었는지** 먼저 검사합니다. 참이면 해당 틱은 이동/착지 검사 없이 Floating0으로 전환하고 Fallen과 표 커서를0으로 초기화합니다. Fallen15에서3픽셀을 더 내려18이 된 뒤 다음 틱에 펼칩니다. 발에 고체가 있거나 Fallen이60을 넘어도 이 펼침 분기가 먼저입니다.
- Floating은 일반 애니메이션 증가를 건너뜁니다. 표의 현재 커서로 frame/dy를 선택하고 커서를 먼저 증가시킵니다.16이 되면8로 돌아갑니다. 전환 직후 frame0은 이 표와 별개로 한 틱 표시됩니다.
- 처음0..7의 dy는 `[3,3,3,3,-1,0,1,1]`, frame은 `[1,2,3,5,5,5,5,5]`입니다. 이후8..15는 dy2와 frame `[5,6,7,7,6,5,4,4]`를 반복합니다. 펼침 직후1픽셀 상승과 정지 구간을 일정한 하강으로 바꾸지 않습니다.
- dy가 양수이면 매 픽셀마다 현재 발을 먼저 검사합니다. 빈 칸일 때만 Y를1 늘리며 minY도0부터 증가합니다. 고체를 만나면 즉시 Walking0으로 전환하고 남은 이동·하부 검사를 건너뜁니다. dy0/-1은 지형을 검사하지 않습니다.
- 하부163 검사는 표의 전체 이동이 끝난 후입니다. 예를 들어 Y162에서dy3은165에서 제거되며, 먼저 고체에 착지했다면 Walking으로 돌아갑니다. 원문의 이 순서를 별도 보정하지 않습니다. 산술 오류는 복사 상태에서 검사해 원래 개체/배치를 보존합니다.
- 표 커서는 Floating을 떠나도 남으며 다시 진입할 때0으로 초기화합니다. 복원 시0..15를 요구하고, Floating 또는0이 아닌 커서는 Floater 능력이 필요합니다. Bomber 타이머가 Floating 중 만료되면 OhNo를 건너뛰고 Exploding으로 전환합니다.

기존 `advance`와 직업/Bomber 조정기를 그대로 사용합니다. 논리 표는 no_std 코어에 있고 이미지/팔레트·SDL/파일 읽기는 데스크톱에 있습니다. 리소스 계층 의존성을 시뮬레이션에 추가하지 않습니다.

## SDL·재생과 실행

F/C/E/K/U/M/B/D는 Floater/Climber/Bomber/Blocker/Builder/Miner/Basher/Digger 선택입니다. 성공한 할당만 수량을 줄이며 클릭은 안정된 ID로 기록합니다. 일시정지 중 할당은 가능하지만 이동·표 커서는 재개 후 진행합니다. 캡처의 `floater.jsonl`은 수량과 `[id,is_floater,float_index]`를 기록합니다. 기존 직업·지형/객체 맵·Climber 상태와 함께 대조합니다. 실제 UI 로그의 Pause/Resume·생성 속도 입력도 읽어 기준 실행에 재생합니다.

```sh
source assets/local/dev-tools/env.sh
cargo run --release --locked --offline -p lemmings-desktop -- --assets assets/local/dos-original --level 2
python3 tools/lemmix/build_floater_reference.py assets/local/my-floater-reference --compiler /usr/lib/fpc/3.2.2/ppcx64 --units /usr/lib/fpc/3.2.2/units/x86_64-linux/rtl
python3 tools/check_floater.py --binary target/release/lemmings-desktop --reference assets/local/my-floater-reference/reference
SDL_VIDEODRIVER=x11 python3 tools/verify_floater.py assets/local/my-floater-native --reference assets/local/my-floater-reference/reference --xdotool /usr/bin/xdotool
make check
```

재생 예는 `54 0 AssignFloater 0`입니다. `--synthetic-floater`는 자체 생성 높은 발판/낙차·하부 지형·벽을 사용하며 두 번째 낙하 중 Bomber 만료도 검사합니다. CI는 실제 원문 실행에서 기록한 golden과 어댑터 해시를 원본 없이 확인합니다. 갱신에는 실제 `--reference`가 필요합니다. 기준 호스트의 그래픽·음향/기록기 스텁은 전체 Lemmix 실행 인증이 아닙니다.

## 원본 표와 DOS 관측

고정 EXE의 파일 offset49736부터224바이트는 `(frame,signed-dy)` 순서의112개 항목입니다. 처음8개 이후 같은8개 하강 항목을13회 저장합니다. 이를 원문 표와 전수 확인했습니다. DOS는 actor+0x0e의 바이트 커서를 선형 증가시키고 Lemmix는8..15의 논리 커서로 반복하므로 두 저장 값을 그대로 같은 상태라고 간주하지 않습니다. 데이터는 ignored 경로에 보존하고 해시만 기록합니다.

Fun02 “Only floaters can survive this”는10명/10%/RR50/5분·Floater10/다른 직업0입니다. 네이티브 camera512와 DOS scroll528의16픽셀 테두리를 대응시킵니다. F2 후보 `IJJLDLCCCL`은 [직접 역분석한 DOS 코드 알고리즘](https://www.lemmingsforums.net/index.php?topic=7364.0)에서 rating0/level1/percentage100/salt0으로 계산하고 원본 preview/진입 상태로 확인합니다. 일반 사용자 데이터에 이 후보를 강제로 쓰지 않습니다.

F4 선택값3을 관측한 뒤 첫 개체의 Falling에 Space로 할당합니다. 고정 네이티브 입력54는 DOS counter55의(568,41)에 대응하며 테두리를 뺀 위치는(552,25)입니다. 포인터는 이후 HUD로 옮깁니다. EXE368b는 actor+0x25(첫 개체DS:ad)에255를 쓰고368f는 DS:6d 수량을 줄입니다. Climber의 DS:ac와 독립된 바이트입니다.22e6의 Floating 코드는0x400입니다. 관측은 이 능력·수량10→9·Floating·표 이동·안전한 Walking 착지를 읽기 전용으로 확인합니다.

```sh
python3 tools/observe_floater_dos.py --dosbox /usr/bin/dosbox --xdotool /usr/bin/xdotool --output assets/local/my-floater-dos --seed assets/local/issue-34-dos-proof-1/bootstrap/drive/RUSSELL.DAT --seed-sha256 04462851c5edc273303b6fd19b8ee3c47ef722b9c1803dfd5adbd0229041e313
python3 tools/observe_floater_dos.py --dosbox /usr/bin/dosbox --xdotool /usr/bin/xdotool --output assets/local/my-floater-dos-repeat --seed assets/local/issue-34-dos-proof-1/bootstrap/drive/RUSSELL.DAT --seed-sha256 04462851c5edc273303b6fd19b8ee3c47ef722b9c1803dfd5adbd0229041e313
python3 tools/compare_floater_dos.py --recording assets/local/my-floater-dos/run assets/local/my-floater-dos-repeat/run --native assets/local/my-floater-native/F02-single --output assets/local/my-floater-comparison
```

DOSBox0.74-3·VGA/PC2·fixed10000cycles·같은 seed/정규화 설정/mapper/컨트롤러와 서로 다른 녹화 경로를 요구합니다. #34의 실제256바이트 seed offset0만9→0으로 바꾸고 원본50개/EXE·에뮬레이터·게임 메모리는 수정하지 않습니다. `/proc` 자식 메모리 읽기만 수행합니다. 전체320×160·정확한2배 픽셀/스캔라인·RGB565 코드 정규화만 사용하며 커서 마스크·위치 이동·색 허용 오차는 없습니다.

원본 네이티브 검증은300호출 경계까지로 제한하며 출구 진입·구출 결과를 인증하지 않습니다. 비동기 숫자 폴링과 화면 앵커는 전체 원자적 DOS 상태/입력 틱/속도·전체 HUD/음향을 인증하지 않습니다. 8개 개별 직업 구현과 G07 전체 완료를 구분합니다. Nuke·전체 상호작용 마무리·객체/구출/승패는 후속입니다. [보고서](../reports/issue-42_floater.md)와 [manifest](../../assets/manifests/floater.json)에 최종 실행·제외 범위와 출처/해시를 기록합니다.
