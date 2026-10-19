# G07-09 Nuke 계약과 재현

이슈: [#44](https://github.com/edwardkim/ohmylemmings/issues/44), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 일괄 폭파와 순서

고정 Lemmix `40e9bc34451f0e9127fd53b290a3877e700d143d`의 [Game.pas](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas) BtnNuke/CheckUpdateNuking 원문을 기존48함수와 FloatParametersTable 호스트에 더해50함수를 실행합니다. 본문은 변경하지 않으며 기존 두 inline 변수 문법 변환만 유지합니다. 새 호스트는 리스트 기본 인덱서·상태/JSON을 연결하고 RecordNuke/음향은 스텁으로 둡니다.

Nuke 입력은 즉시 새 생성을 막고 spawn_countdown을 동결합니다. 시작 시계·입구 열림 표지·기존 개체의 이동은 계속됩니다. 이 동결은 Lemmix/Rust의 논리 표현입니다. DOS DS:48 카운트다운은 Nuke 뒤에도1..28을 반복하지만 최대 인원을 이미 생성된 수로 줄여 출생을 막습니다. 다시 생성하지 않는 범위에서 결과는 같으며 이 비활성 원시 카운터가 Rust와 바이트 단위로 같다고 주장하지 않습니다. 실제 Tick의 전체 개체 이동·Bomber/객체 검사 후에 다음 개체 하나를 처리합니다. 수량을 쓰지 않으며 기존 타이머가0이고 Splatting/Exploding이 아닐 때79로 설정합니다. 이미 타이머가 있거나 제외 행동이어도 해당 순서는 소비합니다. 마지막 개체를 처리한 틱에는 assigning이 남고 다음 틱의 목록 끝 검사에서 꺼집니다. 중복 Nuke는 거부합니다.

일반 DOS 조작은 Pause 중 Nuke가 불가능합니다. 입력 어댑터는 이에 따라 거부하며 같은 경계에서도 Pause→Nuke와 Resume→Nuke를 순서대로 처리합니다. 원문 BtnNuke 자체에는 Pause 검사가 없으므로 호스트 입력 래퍼의 정책과 본문을 구분합니다. 일시정지 중에는 타이머·목록 커서를 진행하지 않습니다.

NukeGlitch 기본 규칙은 max_lemmings를 최초 레벨 인원에서 입력 당시 released로 바꿉니다. 불변 StartupConfig의 레벨 총수와 이 실행 상태를 분리합니다. `new_with_nuke_glitch(config,false)`는 최대 인원을 유지하면서도 생성을 중단합니다. 실제 구출률·승패/결과 계산은 아직 구현하지 않습니다.

## DOS와 Lemmix의 제거 검사 차이

고정 VGALEMMI.EXE의213a는 기존 타이머를 먼저 검사하고2140이 제거 바이트를 검사합니다. 따라서 타이머가0인 제거 개체는 즉시 건너뛰지만, 타이머가 남은 제거 개체는 **한 틱의 순서를 소비**합니다. Lemmix는 모든 제거 개체를 먼저 건너뛰므로 이 경계에서 다릅니다. Rust는 DOS 순서를 채택했으며 실패 테스트→수정으로 별도 검증합니다. 원문50함수 대조의10사례가 이 차이까지 같다고 확대하지 않습니다.

Nuke는 AssignBomber와 달리 OhNo에 타이머를 다시 설정할 수 있습니다. 이후 짧은 OhNo 애니메이션이 먼저 끝나면 Exploding·제거 상태에도 양수 타이머가 남습니다. 이 상태를 복원 가능한 계약으로 확장했습니다. Splatting은 여전히 타이머0을 요구하며 일반 AssignBomber의 OhNo 거부 규칙은 유지합니다. 기존 직업 검사기는 엄격한 모드를 유지하고 Nuke 검사기만 명시적으로 허용한 뒤 Nuke 상태를 함께 검증합니다.

## 코어와 SDL

StartupScheduler가 Nuke 상태와 입력/생성 중지를 소유합니다. `update_nuking`은 전체 이동 후 호출하며 동일 iteration의 중복 호출은 진행하지 않습니다. 제거된 슬롯을 지우지 않은 생성 순서 ID 목록을 요구하고, 길이/ID 오류를 모든 변경 전에 검사합니다. TickStepper 오버플로도 Nuke/시작 상태와 콜백을 보존합니다. 코어에 할당·렌더링·I/O 의존성을 추가하지 않습니다.

N키는 일회 Nuke 요청입니다. 키 반복·재생 중 실시간 N키·Pause 중 N키는 기록하지 않습니다. 재생 형식은 `120 0 Nuke`로 개체 ID가 없습니다. `nuke.jsonl`은 is_nuked/assigning/next_index/max_lemmings를 기록합니다. 기존 Bomber 카운트다운·폭발·입자와 Blocker 필드 복원, 두 영구 능력을 재사용합니다. 하단은 현재 텍스트 HUD이며 원본 기술 선택 패널·미니맵은 G08의 HUD 통합 범위입니다. 게임 영역320×160 대조와 하단40픽셀의 미완성을 구분합니다.

```sh
source assets/local/dev-tools/env.sh
cargo run --release --locked --offline -p lemmings-desktop -- --assets assets/local/dos-original --level 6
python3 tools/lemmix/build_nuke_reference.py assets/local/my-nuke-reference --compiler /usr/lib/fpc/3.2.2/ppcx64 --units /usr/lib/fpc/3.2.2/units/x86_64-linux/rtl
python3 tools/check_nuke.py --binary target/release/lemmings-desktop --reference assets/local/my-nuke-reference/reference
SDL_VIDEODRIVER=x11 python3 tools/verify_nuke.py assets/local/my-nuke-native --reference assets/local/my-nuke-reference/reference --xdotool /usr/bin/xdotool
make check
```

`--synthetic-nuke`는 자체 Builder 경기장의 지형/직사각 마스크를 사용합니다. 합성 사례는 생성 전/생성 직전·세 개체·중복·Pause/Resume·Blocker·기존 Bomber·OhNo 재설정·두 영구 능력·직업 변경을 다룹니다. CI는 실제 원문 실행에서 기록한 golden과 어댑터 해시를 원본 없이 검사하며 갱신에는 `--reference` 실제 실행이 필요합니다.

## DOS 실행과 범위

원본 LEM3.DOC은 F12 Armageddon의 이중 입력을 명시합니다. Fun06 “A task for blockers and bombers”의50명/20%/RR50/5분·Bomber5/Blocker5로 진입합니다. 네이티브 camera576과 DOS scroll592의16픽셀 테두리를 대응합니다. DOS 첫 F12는 counter118, 두 번째는121이며 두 입력 간격은 원본의5카운터 창 안입니다. 고정 네이티브 Nuke 경계120에 대응하는 관측입니다.

DOS412b는 DS:4b에255,4133은 DS:47에 이미 생성된 인원을 씁니다. DS:4c는 다음 순서이며 완료 후4b가0이 됩니다. 이를 영구 is_nuked 플래그로 오인하지 않습니다. actor 배열은 DS:88부터45바이트 간격이며 +20h가 타이머, +26h가 제거, +8h가 행동입니다. 원본/EXE·에뮬레이터·게임 메모리를 수정하지 않고 `/proc` 자식 메모리만 읽습니다. 게임 종료/결과 화면 진입 자체는 Rust에 아직 연결하지 않습니다.

```sh
python3 tools/observe_nuke_dos.py --dosbox /usr/bin/dosbox --xdotool /usr/bin/xdotool --output assets/local/my-nuke-dos --seed assets/local/issue-34-dos-proof-1/bootstrap/drive/RUSSELL.DAT --seed-sha256 04462851c5edc273303b6fd19b8ee3c47ef722b9c1803dfd5adbd0229041e313
python3 tools/observe_nuke_dos.py --dosbox /usr/bin/dosbox --xdotool /usr/bin/xdotool --output assets/local/my-nuke-repeat --seed assets/local/issue-34-dos-proof-1/bootstrap/drive/RUSSELL.DAT --seed-sha256 04462851c5edc273303b6fd19b8ee3c47ef722b9c1803dfd5adbd0229041e313
python3 tools/compare_nuke_dos.py --recording assets/local/my-nuke-dos/run assets/local/my-nuke-repeat/run --native assets/local/my-nuke-native/F06-single --output assets/local/my-nuke-comparison
```

DOSBox0.74-3·VGA/PC2·fixed10000cycles·같은 seed/mapper/컨트롤러·서로 다른 경로를 요구합니다. #34의256바이트 seed offset0만9→0으로 변경합니다. 전체320×160의 정확한2배 픽셀/스캔라인·RGB565 코드 정규화만 사용하며 커서 마스크·위치 이동·색 허용 오차는 없습니다. 비동기 숫자/장면 앵커는 원자적 전체 DOS 상태·입력 틱/속도·하단 패널·음향·구출/승패를 인증하지 않습니다. 전체 직업 전환/상호작용 마무리·G07 전체는 후속입니다.

DOS 기록은 해당 자식 창으로 키를 직접 전달하며, 녹화를 멈춘 뒤 그 기록용 자식만 종료합니다. 비교는 요청한 마지막 장면260까지 끝내고 이후 결과 화면은 별도 제외 프레임 수로 남깁니다. 결과 화면의 글꼴은 게임 영역의2배 스캔라인 규칙을 따르지 않으므로 이를 허용 오차로 바꾸지 않습니다. 최종 실행 수/범위·[보고서](../reports/issue-44_nuke.md)·[출처/해시](../../assets/manifests/nuke.json)를 구분해 기록합니다.
