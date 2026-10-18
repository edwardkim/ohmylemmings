# G07-03 Miner 계약과 재현

이슈: [#33](https://github.com/edwardkim/ohmylemmings/issues/33), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 기준과 코어 계약

고정 Lemmix `40e9bc34451f0e9127fd53b290a3877e700d143d`의 [Game.pas](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas)에서 AssignMiner·HandleMining·ApplyMinerMask 원문을 추출하고 기존21개와 합친24개 함수를 FPC3.2.2로 실행합니다. 원문 본문은 기존 두 inline 변수 문법 변환 외에 변경하지 않습니다. 마스크 DrawTo/사각형 계산·툴바/사운드/기록기는 최소 호스트이며 전체 Lemmix 실행 인증은 아닙니다. 이전 호스트의 미사용 Mining 플래그/Floating 충돌은 새 호스트에서 독립 플래그로 분리했습니다.

- `MiningContext`는 caller 소유104바이트(오른쪽2장·왼쪽2장, 각16×13/26바이트) MSB-first 삭제 마스크와 기존 객체 뷰를 받습니다. 저장된1을 삭제하며 원본 좌향 이미지를 미러링하지 않습니다.
- Walking/Digging/Bashing에서 수량·전방 `(x+8*방향,y-8)` 강철/역방향 일방통행·현재 발의 강철을 검사합니다. 성공하면 Mining/frame0으로 바꾸고 **즉시 y+1**·수량-1입니다. 좌표 증가가 불가능하면 상태/수량을 보존하고 거부합니다. Miner에서 Digger/Basher로도 교체합니다. 미구현 Building/Shrugging 전환은 후속입니다.
- 애니메이션24프레임을 먼저 증가합니다. 프레임1은 `(x-8,y-13)`에 첫 마스크, 프레임2는 `(x+방향-8,y+1-13)`에 둘째 마스크를 적용하며 객체 검사를 하지 않습니다.
- 프레임3/15는 한 칸씩 두 번 전진하고 **각 이동 후**0..1647 경계를 검사합니다. 이탈하면 Walking으로 전환하고 뒤돕니다. 프레임3만 y+1 후163 초과를 제거합니다. 지지가 없으면 Falling, 지지가 있으면 새 발의 강철/역방향 일방통행에서 Walking으로 뒤돕니다. 지형 공백이 객체 제한보다 우선합니다.
- 프레임0은 y+1/바닥 제거만 수행하며 지지를 검사하지 않습니다. 객체 검사 허용은0/3/15뿐이고 바닥 제거 시 취소합니다. 한 주기에 수평4·수직2픽셀 이동하며 상태 진입 y+1은 별도입니다.
- `one_way_right_bug=true`가 DOS 원작 기본값입니다. 이때 채굴 중 오른쪽 일방통행은 양방향 모두 중단합니다. 할당 시의 방향 검사는 정상 규칙입니다. 수정 규칙(false)도 코어768조합 테스트에 포함합니다.
- `advance_with_jobs`/`step_actors_with_jobs`가 세 직업을 처리합니다. 기존 API는 유지합니다. 산술 오류는 해당 배우/픽셀을 보존하고 ID 순서 오류는 모든 변경 전에 거부합니다. 앞 배우의 성공은 뒤 배우의 실패에도 유지합니다.

## 네이티브 실행

[SDL2 준비](../../apps/desktop/README.md) 후 저장소 루트에서 실행합니다.

```sh
cargo run --release --locked --offline -p lemmings-desktop -- --assets assets/local/dos-original --level 4
```

M/B/D로 Miner/Basher/Digger를 선택하고 왼쪽 클릭합니다. `80 0 AssignMiner 0` 같은 정수 경계/ID 입력을 재생합니다. 일시정지 중 할당은 수량/상태만 바꾸고 실제 굴착은 재개 후 갱신합니다. 기본 선택은 Digger입니다. 원본 Fun04는 Miner1·Climber10개이며 Climber/완주는 아직 지원하지 않습니다.

캡처의 `miner.jsonl`은 수량, 기존 `digger.jsonl`은 공통 지형 삭제/서명과 Digger 플래그, `basher.jsonl`은 Basher 수량입니다. `config.json`, `objects.bin`, `bash-masks.bin`, `mine-masks.bin`으로 실제 기준 실행 입력을 재현합니다. 원본 파생물은 ignored `assets/local/` 아래의 새 폴더에만 둡니다.

## 재현과 관측 한계

```sh
python3 tools/lemmix/build_miner_reference.py assets/local/my-miner-reference --compiler assets/local/issue-26-tools/fpc/usr/lib/x86_64-linux-gnu/fpc/3.2.2/ppcx64 --units assets/local/issue-26-tools/fpc/usr/lib/x86_64-linux-gnu/fpc/3.2.2/units/x86_64-linux/rtl
python3 tools/check_miner.py --reference assets/local/my-miner-reference/reference
make check
```

이 경로는 앞 단계에서 확보한 고정 FPC SDK입니다. 시스템 SDL2 개발 패키지가 없다면 LIBRARY_PATH·LD_LIBRARY_PATH를 `assets/local/issue-27-tools/sdk/usr/lib/x86_64-linux-gnu` 절대 경로로 설정합니다. CI는 원본/참조 사본 없이 자체 합성 마스크·실제 실행에서 기록한 golden과 어댑터 해시를 검사합니다. golden 갱신은 실제 `--reference`를 지정한 `--record`에서만 허용합니다.

격리 X11에서 `tools/verify_miner.py OUTPUT --reference REFERENCE --xdotool XDOTOOL`로 빠른1/7·실제 pacing·실제 M키/클릭을 대조합니다. `tools/observe_miner_dos.py --dosbox DOSBOX --xdotool XDOTOOL --output OUTPUT`은 원본 내장 F2 코드로 Fun04를 선택합니다. [코드 후보 표](https://www.dosdays.co.uk/media/games/lemmings/Lemmings_Level_Codes.pdf)는 원본 제목·수량으로 재검증합니다. DS:7c 수량·배우 상태0x800은 EXE의0x36b8 할당 명령에서 식별했습니다. 원본50개/EXE/메모리는 불변이고 #29의 별도 생성 RUSSELL.DAT seed 전제를 유지합니다.

DOS 실행은 전용 화면에서 순서대로 실행해야 합니다. `/proc` 폴링은 원자적 전체 DOS 상태가 아닙니다. 불완전한 메모리 표본은 살아 있는 프로세스에서 거부하며 종료 직후 EOF는 앞선 완전한 표본이 있을 때만 종료 관측으로 기록합니다. 네이티브 실제 입력 관측도 완성된 JSONL 줄만 읽습니다. `tools/compare_miner_dos.py`는 전체320×160 플레이 영역을 RGB565로 정규화하고 정확한2배 중복을 검사하며 커서 마스킹·좌표 이동·색 허용 오차를 사용하지 않습니다.

원본 좌향/강철/일방통행 실레벨·전체 확장 객체 지도·이중 후보/우클릭 우선순위·음향/출구/승패·Windows·전체 DOS 인증은 미검증입니다.
