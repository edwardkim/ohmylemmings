# G06-03 DOS 객체 위상·실행 간격 보정

이슈: [#30](https://github.com/edwardkim/ohmylemmings/issues/30), 원본 관측 [#28](https://github.com/edwardkim/ohmylemmings/issues/28), 런타임 전제 [#29](issue-29_dos_runtime.md).

## 입력과 판정 범위

[원본 50개](../../assets/manifests/dos_original.json)와 VGALEMMI.EXE/DOSBox 0.74-3의 해시를 유지했습니다. 원본 EXE/리소스·에뮬레이터·게임 메모리 패치는 없습니다. 별도 생성 RUSSELL.DAT 카운터 seed 조건은 #29 그대로입니다. [교정 증거](../../assets/manifests/dos_calibration.json)에 원본 코드 범위 해시·8개 추가 DOS 실행·제한된 읽기 전용 메모리 관측·원본 세 영상 대조·네이티브 실행을 고정합니다.

원본 프로그램은 전체 파일 기준 i8086 disassembly로 조사했습니다. MZ 헤더 크기 0x120을 빼면 코드 세그먼트 오프셋입니다. 코드 일부를 저장소에 복제하지 않고 파일 오프셋/크기/해시와 해석을 기록합니다. DOSBox의 고정 원본 timer.h는 PIT_TICK_RATE=1193182를 정의합니다. [공식 매뉴얼](https://www.dosbox.com/DOSBoxManual.html)의 cycles 설정과 실제 영상/카운터 관측을 함께 사용합니다.

## 객체 위상의 원인

원본은 GROUND의 시작 프레임을 복사한 뒤 초기 표시 전에 객체를 한 번 증가시킵니다(0x38aa, 0x5b5). 메인 루프는 startup counter를 먼저 증가시키고(0x2c3), 객체 프레임을 증가시킵니다(0x2d6→0x7d2a). 입구는 별도 trigger 조건을 따릅니다.

읽기 전용 폴링에서 입구 프레임 2는 counter 36, 최초 생성은 counter 55입니다. 같은 입구/생성 이미지의 네이티브 이름은 각각 35/54입니다. 최초 생성 시 장식 ID7의 프레임은 2이며 초기화 때는 0→1 증가도 관측했습니다. 따라서 F00의 대응 이동 이미지에서 DOS counter는 native label+1, 연속 객체의 표시 위상은 `first + native_iteration + 2 (mod frame_count)`입니다. 입구 계산은 이미 맞아 변경하지 않았습니다.

이것은 모든 DOS 내부 상태나 절대 입력 경계가 같다는 뜻이 아닙니다. 초기 정지 이미지는 여러 counter에서 반복됩니다. F01의 네이티브 입력 경계 60을 DOS에 주입하는 작업은 별도 계측해야 합니다.

## 타이머 경로와 실행 간격

0x2a6의 루프는 IRQ counter가 4보다 커질 때까지 기다린 뒤 counter를 지웁니다. 즉 업데이트당 IRQ 5개입니다. 게임 시계 자체는 별도로 17호출마다 감소합니다(0x538/0x53d). 두 단위를 구분합니다.

| 선택 | 실제 DS:200e | 타이머 경로 | 관측 진행률 |
|---|---|---|---|
| 기존 메뉴 PC2 | 0 | IRQ0에서 VGA retrace 대기/재설정 | `2190197/31250 / 5` ≈ 14.017호출/초 |
| 명령줄 -o | 0 | VGA 동기 경로 | 같은 영상 5프레임 간격 |
| 명령줄 -x + 메뉴 PC2 | 255 | retrace를 건너뛰고 PIT divisor 16384 | `1193182/(16384×5)` ≈ 14.565호출/초 |

메뉴 PC2와 -x를 같은 경로로 가정하면 안 됩니다. 실제 메뉴 PC2의 flags는 [0,0,0,0,0], PIT divisor는 13450/13451이었고, -x는 [0,0,0,0,255]/16384였습니다. 원본 0x18f6의 분기와 0x16cd의 설정이 이를 설명합니다.

추가 8실행에서 모든 이동 이미지가 나타났습니다. 7실행은 해당 타이머 모델과 영상 양자화 미만(1프레임 미만)의 오차로 맞습니다. GUI PC2/5000 cycles의 마지막 340 이미지는 이상적인 간격보다 **영상 1프레임 늦어 DIFFERENT**로 보존했습니다. 이 결과를 통과로 바꾸거나 5000 cycles까지 동일 속도라고 일반화하지 않습니다. 기준 10000과 별도 GUI 30000은 정확히 5프레임 간격입니다.

## 구현

[PlaybackClock](../../apps/desktop/src/pacing.rs)은 데스크톱 어댑터에서 정수 유리수로 시간을 누적합니다. 원본 기본값은 `dos-vga-5`(2190197/156250), 합성 기본값과 `--pacing reference`는 기존 17Hz입니다. `--pacing dos`로 명시할 수도 있습니다. 큰 프레임의 미처리 호출을 버리지 않으며 초과 오류는 누적 상태를 바꾸지 않습니다.

코어의 논리 게임 시계·경계 입력·시작/생성·개체 규칙·전체 상태 서명은 바꾸지 않았습니다. 연속 객체의 위상만 별도로 보정했습니다. run.json에 pacing 이름/분자/분모를 기록합니다. --fast는 벽시계 대기만 생략합니다.

## 읽기 전용 계측

[observe_dos.py](../../tools/observe_dos.py)는 직접 실행한 DOSBox 자식의 /proc/pid/mem을 O_RDONLY로 열고 재배치된 MZ 코드로 RAM을 찾습니다. 원본 고정 바이트에서 DS를 계산하며 주소를 하드코딩하지 않습니다. 문서화한 counter·개체 수·객체 프레임·PC 플래그·PIT divisor만 2ms 간격으로 읽습니다. 원본 50개의 실행 전후 해시도 확인합니다.

폴링은 비동기이며 한 루프 도중의 값도 포함합니다. 이것을 atomic full-state trace로 취급하지 않습니다. SDL 종료 중 RAM이 먼저 해제되는 EIO는 종료 이유로 남기고, 정상 프로세스 종료·최대 생성 수 10·별도 영상 기준 장면을 함께 확인합니다. 위치를 못 찾거나 다중 후보/관측 오류가 있으면 통과하지 않습니다. 원본 메모리 덤프는 저장하지 않습니다.

## 재현

#28의 DOSBox/SDK·PyAV 14.2.0·원본 및 #29 bootstrap 파일을 준비합니다. 모든 출력은 새 assets/local 경로를 사용합니다. 격리된 X11에서 실행합니다.

```sh
python3 -m unittest discover -s tools/tests -v
cargo build --release --locked --offline -p lemmings-desktop
xvfb-run -a python3 tools/verify_native_view.py assets/local/calibrated-native-new --xdotool /usr/bin/xdotool
PYTHONPATH="$PWD/assets/local/issue-28-tools/python" python3 tools/compare_dos.py --recording assets/local/issue-28-dos-proof-2 --native assets/local/calibrated-native-new/F00-realtime --output assets/local/calibrated-comparison-new
xvfb-run -a python3 tools/observe_dos.py --dosbox /usr/bin/dosbox --xdotool /usr/bin/xdotool --pc 0 --cycles 10000 --output assets/local/observed-gui-new
xvfb-run -a python3 tools/observe_dos.py --dosbox /usr/bin/dosbox --xdotool /usr/bin/xdotool --pc 2 --cycles 10000 --output assets/local/observed-x-new
PYTHONPATH="$PWD/assets/local/issue-28-tools/python" python3 tools/verify_dos_timing.py --recording assets/local/observed-gui-new assets/local/observed-x-new --native assets/local/calibrated-native-new/F00-realtime --output assets/local/timing-analysis-new
```

5000/30000 cycles도 같은 --cycles 옵션으로 분리합니다. 영상 해제 도구만 PyAV를 사용하고 unittest/오프라인 CI는 원본·DOSBox·PyAV 없이 실행합니다. 관측 중 원본 입력/출력을 섞거나 기존 파일을 덮어쓰지 않습니다.

이번 실제 재검증 출력은 assets/local/issue-30-full-comparison-verified/와 issue-30-timing-proof-complete/입니다. 비교 PNG는 run-1/pair-000170.png, 왼쪽 DOS/오른쪽 네이티브입니다. 각 scope verdict는 현재 계산한 기준 장면에서 다시 계산하며 예전 캐시 판정을 재사용하지 않습니다. 초기 장면의 그림은 가능한 첫 전체 일치 프레임을 선택하고 전체 일치 범위/상부 일치 범위는 모두 보존합니다.
