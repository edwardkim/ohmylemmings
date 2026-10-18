# G07-02 Basher 계약과 재현

이슈: [#32](https://github.com/edwardkim/ohmylemmings/issues/32), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 기준과 경계

고정 Lemmix `40e9bc34451f0e9127fd53b290a3877e700d143d`의 [Game.pas](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas)에서 AssignBasher·HandleBashing·ApplyBashingMask 원문을 추출합니다. 기존 18개와 합친 21개 함수를 FPC 3.2.2로 실제 실행합니다. 마스크 DrawTo만 MSB-first 삭제/클리핑 어댑터이며 그래픽 프레임워크·사운드·툴바·기록기는 최소 호스트입니다. 전체 Lemmix 실행 인증은 아닙니다.

- 코어는 `no_std`이며 `BashingContext`로 caller 소유 160바이트 마스크와 강철/왼쪽/오른쪽 일방통행 지도를 받습니다. 맵은 상호 배타적인 객체 셀이며 강철 픽셀을 개별 보호하는 마스크가 아닙니다.
- 살아 있는 Walking/Digging에서 수량과 전방 `(x+8*방향,y-8)` 제한을 검사합니다. 거부 시 수량·배우 상태를 보존합니다. Basher→Digger도 지원합니다. 아직 없는 Building/Mining/Shrugging 전환은 후속입니다.
- 32프레임 애니메이션을 먼저 증가시킵니다. 2..5/18..21에서 방향별 네 마스크를 `(x-8,y-10)`에 적용합니다. 원본 마스크를 미러링하지 않습니다.
- 프레임5에서만 삭제 후 전방 네 픽셀 `(x+방향*(8..11),y-6)`이 모두 비면 Walking으로 바꿉니다. 프레임21은 이 검사를 생략합니다.
- 11..15/27..31은 한 픽셀 전진하고 세 바닥 픽셀을 검사합니다. 세 픽셀이 비면 세 칸 내려가 Falling, 지지가 있으면 새 위치의 강철/역방향 일방통행에서 Walking으로 전환하고 뒤돕니다. 수평 범위0..1647 이탈도 뒤돕니다.
- 산술 오류는 해당 배우/픽셀을 보존합니다. 배치는 ID 순이며 순서 오류는 모든 변경 전에 거부합니다. 뒤 배우 오류 시 앞 배우의 성공은 유지합니다.

## 네이티브 사용

[SDL2 준비](../../apps/desktop/README.md) 후 저장소 루트에서 실행합니다.

```sh
cargo run --release --locked --offline -p lemmings-desktop -- --assets assets/local/dos-original --level 5
cargo run --release --locked --offline -p lemmings-desktop -- --synthetic-basher
```

B/D로 Basher/Digger를 고르고 왼쪽 클릭합니다. 기본 선택은 Digger입니다. `--level`은 Fun1..30의 표준 그래픽 레벨을 선택하며 특수 VGASPEC 레벨은 명시적으로 거부합니다. `166 0 AssignBasher 0` 같은 정수 경계/ID 입력을 Pause/Resume/RR과 같은 순서로 재생합니다. 캡처의 `basher.jsonl`은 수량, 기존 `digger.jsonl`은 공통 지형 삭제와 Digger 플래그를 기록합니다. `config.json`, `objects.bin`, `bash-masks.bin`은 기준 실행의 입력입니다. 원본 파생물은 `assets/local/`만 허용합니다.

## 기준 실행과 오프라인 회귀

```sh
python3 tools/lemmix/build_basher_reference.py assets/local/my-basher-reference --compiler assets/local/issue-26-tools/fpc/usr/lib/x86_64-linux-gnu/fpc/3.2.2/ppcx64 --units assets/local/issue-26-tools/fpc/usr/lib/x86_64-linux-gnu/fpc/3.2.2/units/x86_64-linux/rtl
python3 tools/check_basher.py --reference assets/local/my-basher-reference/reference
make check
```

로컬 SDK는 앞 단계에서 확보한 고정 FPC/SDL2 도구입니다. 시스템 SDL2 개발 패키지를 설치하지 않았다면 `LIBRARY_PATH`와 `LD_LIBRARY_PATH`를 `assets/local/issue-27-tools/sdk/usr/lib/x86_64-linux-gnu`의 절대 경로로 설정합니다. 원본이 없는 CI는 자체 합성 마스크와 실제 함수 실행으로 기록한 golden을 사용하며 어댑터·마스크 해시 변경을 거부합니다. golden 갱신은 실제 `--reference`를 지정한 `--record`에서만 허용합니다.

격리 X11에서 `tools/verify_basher.py OUTPUT --reference REFERENCE --xdotool XDOTOOL`로 원본 Fun05의 빠른1/7·실제 pacing·실제 클릭을 검증합니다. `tools/observe_basher_dos.py --dosbox DOSBOX --xdotool XDOTOOL --output OUTPUT`은 내장 F2 코드 입력으로 같은 레벨을 고릅니다. 예비 코드 후보는 [DOS 코드 표](https://www.dosdays.co.uk/media/games/lemmings/Lemmings_Level_Codes.pdf)에서 찾고 원본 게임의 레벨 제목·수량으로 검증했습니다. EXE/DAT/메모리는 변경하지 않으며 #29의 별도 생성 RUSSELL.DAT seed 조건을 유지합니다. 읽기 전용 DS:79 수량과 배우 상태0x200·입력 표본은 비동기 관측입니다.

`tools/compare_basher_dos.py`는 전체320×160 플레이 영역을 RGB565로 정규화하고 정확한2배 중복을 검사합니다. 커서 마스킹·좌표 이동·색 허용 오차를 사용하지 않습니다. 전체 원자적 DOS 상태, 원본 일방통행/강철 실제 레벨, 이중 후보/우클릭 우선순위, 음향·출구·승패·Windows는 미검증입니다.
