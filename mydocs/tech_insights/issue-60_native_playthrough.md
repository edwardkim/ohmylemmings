# G08-07 네이티브 한 레벨 통합 완주 계약

이슈: [#60](https://github.com/edwardkim/ohmylemmings/issues/60) · [진행](../working/issue-60_native_playthrough.md) · [결과](../reports/issue-60_native_playthrough.md)

## 입력과 완료 조건

기준판은 [고정 DOS 자산](../../assets/manifests/dos_original.json)의 Fun01 `Just dig!`입니다. 초기 10마리·목표 1마리(10%)이며 원본 없는 검사는 자체 합성 자산 3마리·목표 2마리(66%)를 사용합니다. 기존 #48 출구·#50 위험물·#52 판정·#54 전환·#56 패널·#58 포인터 계약을 유지합니다.

한 SDL 프로세스에서 다음 순서로 진행합니다.

1. 패널에서 Digger(합성은 Climber)를 선택하고 첫 Walking 개체를 포인터로 할당합니다. Pause 중 생성 속도 버튼을 유지하고 미니맵 오른쪽/왼쪽/복귀, 1000×700 resize·letterbox 클릭을 거쳐 Resume합니다. 전원 구출 후 성공 화면을 확인합니다.
2. R로 같은 레벨을 새로 시작합니다. Nuke 단일 클릭은 명령을 만들지 않으며 충분한 간격 뒤 두 번 클릭하면 생성 상한을 줄이고 실패 결과에 도달합니다. 합성은 Digger로 지형도 변경합니다.
3. 실패 화면의 왼쪽 클릭으로 재시도하여 전원 구출합니다. Enter로 다음 레벨에 진입하고 새 ID 0의 생성을 확인합니다.

성공·실패 결과에서는 논리 상태/시간/렌더 결과가 고정됩니다. 전환은 새 Session/View를 준비하며 지형·객체·수량·시계·Nuke·입력 목록·포인터 기억/버튼·선택·미니맵 드래그를 초기화합니다. 성공의 Continue는 다음 레벨, 실패의 Continue와 R은 같은 레벨입니다. 안정 ID 입력은 시도별로 독립 기록합니다.

## 비교 계층

`playthrough_tests.rs`는 실제 input/panel/pointer/lifecycle 어댑터를 자체 자산으로 연결합니다. Rust 단위 테스트와 Python 검증기의 누락/잘못된 전환·부분 구출·초기화 오류 검사는 기존 `make check`에서 자동 실행됩니다. 원본·X11·DOSBox·FPC 없이 CI에서 실행할 수 있습니다. 자산 손상/누락과 전환 준비 실패의 원자성은 기존 asset/lifecycle 회귀를 함께 실행합니다.

`verify_playthrough.py`는 독립 X11 화면에 생성한 자기 프로세스의 창만 조작합니다. 각 시도의 전체 논리 상태를 렌더 간격 1/7/13의 독립 안정 ID 재생과 비교합니다. 간격 13의 첫 성공은 실시간, 나머지는 빠른 재생입니다. 실제 SDL readback은 정수 배율·letterbox와 논리 RGB가 정확히 일치해야 합니다. 다음 레벨의 실제 상태는 80경계까지 기록하며 픽셀은 기존 캡처 경계만 검사합니다. 프레임이 없는 종료 경계를 새 픽셀 증거로 계산하지 않습니다. 표현용 포인터/카메라는 논리 상태 해시에 포함하지 않으며 초기화와 실제 조작을 별도로 검사합니다.

원본 네 시도와 별도 고정 입력 완주는 Lemmix `40e9bc34451f0e9127fd53b290a3877e700d143d`의 고정 56함수/표를 실행해 전체 정규화 상태와 비교합니다. `Game.pas` SHA-256은 `c1ba9ffda9e8e277f5c9b171df2bb611210d3326b9623b28fcd35426f4d6a448`입니다. #52의 DOS 완료 경계 어댑터(개체 갱신 뒤·시계 전의 시간, timeout 시 particle 지연 처리)를 재사용하며 원본 `InternalUpdate` 전체 실행을 주장하지 않습니다.

새 DOS 녹화는 Fun01 성공 세 번과 Nuke 실패 한 번입니다. 일반 키 입력과 비동기 읽기 전용 RAM 관측으로 초기 레벨·Digger 소비·전원 구출·목표/구출률·승패를 확인합니다. 성공의 원본 결과는 10/10·100%·목표10%, 실패는 0/3·0%·목표10%입니다. 네이티브 원본 조작도 같은 결과를 내지만 호스트 조작 시각은 같지 않습니다. 합성 실패는 1/2·50%로 목표66% 미달입니다.

DOS는 성공 결과 화면에 진입하면서 내부 `level_index`를 0에서 1로 먼저 증가시킵니다. 완료 요청 직후에는 `success=0,index=0`인 과도 샘플도 관측됩니다. 초기 playing Fun01과 마지막 완료 샘플을 확인하고 판정·수량을 대조합니다. 네이티브는 Continue 때 레벨을 바꾸므로 결과 화면의 레벨 인덱스를 직접 동일시하지 않습니다. 이 차이를 Python 실패 테스트로 고정했습니다.

DOS 화면은 실제 UI 조작과 별도인 `60 0 AssignDigger 0` 네이티브 완주를 기준으로, 최초 Exiting부터 아홉 연속 장면의 320×160 전체 게임 영역을 세 녹화에서 순서대로 찾습니다. 기존 `compare_exit_dos.py`의 정확한 2배 축소/RGB565 정규화만 적용합니다. 커서 마스크·위치 이동·오차 허용은 없으며 영상으로 DOS 틱이나 원자적 전체 상태를 추정하지 않습니다. 패널·결과 아트워크는 이 장면 비교 범위 밖입니다.

## 재현

```sh
source assets/local/dev-tools/env.sh
make check
python3 tools/lemmix/build_completion_reference.py assets/local/my-completion-reference \
  --compiler /usr/lib/fpc/3.2.2/ppcx64 --units /usr/lib/fpc/3.2.2/units/x86_64-linux/rtl
# 독립 X11 DISPLAY에서, 각각 새 출력 경로 사용:
python3 tools/verify_playthrough.py assets/local/my-playthrough-synthetic
python3 tools/verify_playthrough.py assets/local/my-playthrough-original \
  --assets assets/local/dos-original --reference assets/local/my-completion-reference/reference
python3 tools/observe_playthrough_dos.py --dosbox /usr/bin/dosbox --xdotool /usr/bin/xdotool \
  --output assets/local/my-dos-win --profile win \
  --seed assets/local/issue-34-dos-proof-1/bootstrap/drive/RUSSELL.DAT
```

DOSBox 0.74-3·PyAV 14.2.0·FPC 3.2.2를 사용합니다. 성공 세 실행은 서로 다른 새 경로에서 반복하고 실패는 `--profile loss`로 실행합니다. Xvfb stderr는 파일로 흘려 작은 PIPE가 누적 경고로 막히지 않게 합니다. `Xvfb -displayfd`로 받은 화면 번호와 xdotool 연결을 확인한 후 시작합니다.

생성 seed 256바이트의 SHA-256은 `04462851c5edc273303b6fd19b8ee3c47ef722b9c1803dfd5adbd0229041e313`입니다. 별도 runtime 복사본의 offset0만 9→0으로 바꾸며 해시는 `56a8f2e2603488fe723bb107c94dbe9c7711e77ebf60dfbb3e2d1c692d1a182f`입니다. 원본 EXE/DAT 50파일은 전후 검증하고 수정하지 않습니다. [출처·증거 매니페스트](../../assets/manifests/native_playthrough.json)에 해시와 실행 범위를 고정합니다.

## 범위와 한계

G08은 네이티브 한 레벨의 조작·성공·실패·재시작과 기준 규칙 비교입니다. 120레벨 전체 DOS 인증, 원본 메뉴/접근 코드/결과 아트워크·커서 비트맵, 음향·웹은 후속 목표입니다. 원본 파생 화면·동영상은 ignored `assets/local/`에만 두고 공개 스크린샷은 자체 합성 자산을 사용합니다.
