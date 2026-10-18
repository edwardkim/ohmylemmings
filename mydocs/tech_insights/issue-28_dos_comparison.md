# G06-02 실제 DOS 화면 대조

이슈: [#28](https://github.com/edwardkim/ohmylemmings/issues/28), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 실행과 출처

[원본 식별](issue-2_dos_original_baseline.md)의 50개를 실행마다 새 drive 폴더에 복사했습니다. 원본 VGALEMMI.EXE는 83194바이트, SHA-256 `4f21e94d53b7ea814ade802b4a37d42481da3876e31fb5a23090e19c9600f21c`입니다. 실행 전·후 모든 원본 파일의 해시가 같습니다. 계측 Lemmix 함수 대신 이 DOS 프로그램을 직접 실행했습니다.

DOSBox 0.74-3 / Ubuntu `0.74-3-5build2` 바이너리 SHA-256은 `9213fb131e11cb195bdff40c24513740ce497c5caf1cef1900ad1faad5c5026b`입니다. [증거 manifest](../../assets/manifests/dos_observation.json)에 실행 파일·패키지·설정·입력·영상·그림·네이티브 상태/도구 해시를 기록합니다. [DOSBox 공식 매뉴얼](https://www.dosbox.com/DOSBoxManual.html)의 conf/mapper/캡처 방식을 사용합니다. PyAV 14.2.0의 [VideoFrame/plane 변환](https://pyav.org/docs/stable/api/video.html)으로 ZMBV 영상을 해제하며 Pillow/numpy는 필요 없습니다.

## 고정 실행 조건

- VGA only, normal CPU / 386, fixed 10000 cycles, 메모리 16MiB, frameskip 0, aspect false, scaler none, surface 출력.
- PC 2 High Performance, 입력 Keyboard. F1 → Fun01 안내 → Space 후 게임 명령 없음. 포인터만 A/P로 HUD 모서리로 이동합니다. 카메라는 초기 624입니다.
- SB Pro 2 / 220 / IRQ7 / DMA1, OPL2·44100Hz. SDL audio dummy로 호스트 출력만 없애며 음원 매핑/오디오 일치는 판정하지 않습니다.
- SDL 1.2 호환 계층의 키 처리를 위해 usescancodes=false. 최소 mapper의 s/v/x는 에뮬레이터 스크린샷/영상/종료이며 DOS 게임 키에 전달하지 않습니다.
- 새 프로세스·새 50개 복사본·같은 런타임 seed로 세 실행. raw host 입력의 시간(ms)은 기록하며 DOS 틱 번호로 바꾸지 않습니다.

설정 템플릿·mapper·입력 절차·도구/원본 식별값을 합친 공통 설정 SHA-256: `055a326fddccde1cbe1afea7984e8445bb3811a0f22c2e65d1a0eef116651fb4`. 실행별 conf의 절대 경로만 다르며 검증기가 템플릿과 실제 파일을 대조합니다.

**런타임 전제:** 원본 키 디스크 이미지는 미확보입니다. 첫 실행은 디스크 1 부재로 종료하고 256바이트 RUSSELL.DAT을 생성합니다. [#29 기록](issue-29_dos_runtime.md)대로 이 별도 런타임 파일의 offset 0 카운터만 9→0으로 seed하여 진입했습니다. EXE/원본 DAT 패치는 없습니다. 이 조건의 실제 DOS 동작을 관측했으며 원본 매체의 무설정 기동을 인증하지 않습니다.

## 화면 판정 계약

원본 영상은 640×400 RGB565, VGA 영상률은 `2190197/31250` fps입니다. [비교 도구](../../tools/dos_observation.py)는 플레이 영역의 가로·세로 2배 픽셀 복제가 정확한지 먼저 검사하고 320×160으로 줄입니다. 양쪽 RGB를 같은 R5/G6/B5 성분 코드로 변환하여 캡처 색상 정밀도만 맞춥니다. 좌표 이동·커서 마스크·허용 오차·보간은 없습니다. HUD 40행은 별도 구현이므로 범위 밖입니다.

전체 320×160 판정과 **입구·상부 이동 지형 320×100** 판정을 각각 보고합니다. 상부 일치는 전체 화면/내부 상태 일치로 승격하지 않습니다. 같은 이미지가 반복되면 모든 영상 인덱스를 보존합니다. native control_step은 대응 그림의 이름이며 DOS control_step으로 해석하지 않습니다. 초기 정지 장면은 50프레임 동안 같아 절대 시작 틱을 추론할 수 없습니다.

## 실제 결과

세 실행의 게임 영상은 1811/1806/1811프레임, 합계 5428프레임입니다. 메뉴 영상 73프레임씩도 보존했습니다. 상부 일곱 장면이 세 번 모두 정확히 같으며, 세 실행의 대응 영상 범위도 같습니다.

| 네이티브 control_step | DOS 게임 영상 인덱스 | 관측 |
|---|---|---|
| 0 | 174~223 | 초기 정지 장면, 절대 틱 불명 |
| 35 | 224~228 | 입구 개방 |
| 54 | 319~323 | 첫 생성·낙하 |
| 60 | 349~353 | 첫 Walking |
| 100 | 549~553 | 두 개체 이동 |
| 170 | 899~903 | 다섯 개체·걷기/낙하 |
| 340 | 1749~1753 | 열 개체 |

전체 플레이 영역은 초기 장면이 219~223에서 일치하지만 이후 여섯 기준 장면은 일치하지 않습니다. 같은 상부 장면의 첫 프레임에서 출구 장식 주변 19~26픽셀이 다릅니다. [#30 차이](issue-30_dos_differences.md)로 원본 우선 보정을 추적하며 전체 판정은 UNVERIFIED입니다.

35~340의 대응 장면 간격은 네이티브 한 호출당 영상 5프레임입니다. 이 설정에서 관측한 장면 진행률은 약 14.017/초이며 채택한 네이티브 17Hz와 다릅니다. CPU 속도/기종·VGA 주사와 DOS 내부 루프를 분리해 후속 보정해야 하며 이 수치를 보편적인 DOS 틱률로 확정하지 않습니다.

## 재현

Ubuntu 24.04의 DOSBox 0.74-3·SDL2 SDK·Xvfb·xdotool, Rust 잠금 의존성, Python 3.12/PyAV 14.2.0이 필요합니다. PyAV wheel·DOSBox/SDL 패키지 식별값은 manifest에 있습니다. 원본 확보는 [리소스 안내](../../assets/README.md), SDL2 빌드는 [앱 안내](../../apps/desktop/README.md)를 따릅니다. 모든 출력 폴더는 새 경로여야 합니다.

```sh
python3 tools/reference_assets.py verify
cargo build --release --locked --offline -p lemmings-desktop
python3 -m pip install --target assets/local/dos-tools/python av==14.2.0
xvfb-run -a target/release/lemmings-desktop --assets assets/local/dos-original --fast --steps 340 --capture assets/local/dos-native-new
PYTHONPATH="$PWD/assets/local/dos-tools/python" xvfb-run -a python3 tools/capture_dos.py --dosbox /usr/bin/dosbox --xdotool /usr/bin/xdotool --native assets/local/dos-native-new --output assets/local/dos-recording-new
PYTHONPATH="$PWD/assets/local/dos-tools/python" python3 tools/compare_dos.py --recording assets/local/dos-recording-new --native assets/local/dos-native-new --output assets/local/dos-comparison-new
```

기존 관측을 독립 재해제하려면 로컬 PyAV 경로를 지정하고 다음 명령을 실행합니다. 실제 이 명령으로 기록된 50개 파일·설정/mapper·입력·영상·런타임 해시와 네이티브 341상태/10개 실제 SDL readback을 재검증했습니다.

```sh
PYTHONPATH="$PWD/assets/local/issue-28-tools/python" python3 tools/compare_dos.py --recording assets/local/issue-28-dos-proof-2 --native assets/local/issue-28-exploration/native --output assets/local/issue-28-dos-comparison-verified
```

출력은 upper_view_status=MATCH_THREE_RUNS, full_playfield_status=UNVERIFIED입니다. `run-1/pair-000170.png`는 왼쪽 DOS/오른쪽 네이티브의 같은 상부 장면이며 아래 출구 장식 차이도 숨기지 않습니다. 원본 영상/PNG/전체 추적은 Git 추적에서 제외합니다.
