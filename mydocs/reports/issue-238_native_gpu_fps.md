# 이 PC의 Native HD GPU FPS와 VNC 영향 측정

담당 [#238](https://github.com/edwardkim/ohmylemmings/issues/238), 선행 [시작 GPU 선택](issue-235_gpu_startup.md). 2026-10-25 측정. 제품 소스는 아래 commit/hash의 기존 release binary다.

## 결과

동일한 Native HD 게임을 실제 모니터에서 실행했다. **GPU 가속은 VNC 연결/해제 모두 약60FPS, software는44.30→45.78FPS**였다. GPU 가속의 이점은 VNC 해제 상태에서도 유지됐다. 현재 프로그램은 표시 목표가60Hz이므로 이 결과로GPU의 무제한 최대 처리량을 알 수는 없다.

| 렌더링 | VNC 연결 평균FPS | VNC 해제 평균FPS | 변화 | 프로세스 CPU 연결→해제 |
|---|---:|---:|---:|---:|
| OpenGL 가속 | 59.94 | 59.98 | +0.07% | 약35%→33% |
| Software | 44.30 | 45.78 | +3.34% | 약81%→76% |

CPU는 프로세스 user+system 시간/관측 전체실행시간으로 한 코어100% 기준의 근사치다. FPS 분석 구간만의 CPU나 PC 전체/VNC 서버 CPU가 아니다. 부모 프로세스의1초 종료 확인 간격이 분모에 포함되므로 정밀 CPU 벤치마크로 해석하지 않는다. 최대 프로세스RSS는 네 조건 모두약280MiB였으며 GPU VRAM 측정값은 아니다.

| 렌더링 | 상태 | 프레임 간격p95 | p99 | 33.33ms 초과 간격/분석 간격 |
|---|---|---:|---:|---:|
| OpenGL | 연결 | 23.28ms | 24.24ms | 1/2855 |
| OpenGL | 해제 | 23.43ms | 24.40ms | 0/2859 |
| Software | 연결 | 27.33ms | 40.10ms | 75/2112 |
| Software | 해제 | 27.25ms | 27.77ms | 9/2182 |

GPU의 평균FPS 차이0.04FPS는60Hz 상한 부근의 작은 차이다. GPU p99는24.24→24.40ms로 좋아졌다고 볼 수 없다. Software는 평균FPS가3.34% 높아졌고 p99가40.10→27.77ms,33.33ms 초과가75/2112→9/2182로 감소했다. 즉 이번에는 VNC 해제의 효과가 software 경로의 긴 지연에서 더 뚜렷했다. 두 상태를 시간 순서로 측정했으므로 배경 부하/온도 등의 영향을 완전히 분리한 인과 실험은 아니다.

## 조건과 실제 실행

- PC: Intel N100/4코어, Intel Alder Lake-N GPU/i915. Wayland/Hyprland,1920×1080·60Hz 모니터.
- Native release 최적화 바이너리,HD표시,실제 전체화면1920×1080. SDL2.32.72의 실제 renderer는 가속`opengl`,대조군`software`다. 게임과 사운드를 켜고 실행했으며 offscreen/dummy/--fast는 사용하지 않았다.
- Fun01 JUST DIG!, DOS-VGA pacing, 입력`60 0 AssignDigger 0`,420논리틱(약30초)까지 같은 자동입력으로 진행했다. 각 실행의 control_step/iteration420·input_count1·camera624를 확인했다. 게임 논리 속도는 렌더링FPS와 별개다.
- 각 VNC 상태에서GPU→software→software→GPU(ABBA) 순서,조건별2회씩 총8회 실행. 첫5초와 마지막1초를 제외하여 회당약23.8초,조건별약47.7초의 표시 간격을 분석했다. 평균FPS는 합산간격수/합산시간,분위수는 두 실행의 간격을 합쳐 nearest-rank로 계산했다.
- 첫 측정의 VNC 연결 상태는 메인테이너가 확인했다. 해당 단계의 실시간 client 수는 수집하지 않았다. 이후 연결해제는 wayvncctl client-list의0클라이언트를 실행 전후와 최종 확인했다. wayvnc 서버는 종료하지 않았고 모니터의 RECORD 차단 사유도 사라졌음을 확인했다.
- 별도 게임 창의 mapped/hidden/fullscreen/1920×1080 상태를 확인했다. 처음 창 생성 시 약950×1034인 임시 크기는 warmup에만 포함되고 분석 구간 전체가1920×1080이다. 종료 후 게임 창은 닫혔고 기존 IDE 화면으로 돌아왔다.

## 측정 정의와 관측기

기존 `--performance`는 HD 옵션을 거부하므로 제품 코드를 바꾸지 않고 임시 LD_PRELOAD 관측기를 사용했다. 이 관측기는 SDL_RenderPresent를 호출하기 전후 CLOCK_MONOTONIC을 읽고 미리 할당한 배열에 저장한다. 출력 크기는60호출마다 확인하며 종료 시에만CSV를 기록한다. 원본 게임 화면 캡처나 프레임별 파일I/O는 하지 않았다.

**보고한 FPS는 SDL present 완료 간격이며 모니터의 실제 scanout이나 원격 VNC 클라이언트의 수신FPS가 아니다.** 표시 호출 비용만 측정한 값도 아니며, 표시 완료 사이의 게임 처리·대기·업로드·present 비용을 포함한 주기다. 관측기 오버헤드를0이라고 주장하지 않는다. 모든 조건에 동일한 관측기를 사용하고CSV프레임수와 게임이 보고한frames가 같음·dropped0을 검증했다. Intel DRM의 해당 게임 프로세스 render-engine 누적 시간도 증가해 실제 GPU 작업을 확인했다(두 GPU 연결 실행 약4.86/4.51초의 누적render 시간). 이 값은 전체PC GPU 사용률이 아니다.

측정 준비에서 짧은probe의 replay범위 검증 실패와 /usr/bin/time 부재를 처리했다. 본 측정은 Python wait4의 프로세스 resource 관측을 사용했고,준비 실패는 위8회 결과에 포함하지 않았다.

## 개별 실행

| VNC | 실행 | 평균FPS | 분석시간 | p95(ms) | p99(ms) |
|---|---|---:|---:|---:|---:|
| 연결 | gpu-1 | 59.874 | 23.817s | 23.247 | 24.237 |
| 연결 | software-1 | 44.499 | 23.843s | 27.090 | 39.727 |
| 연결 | software-2 | 44.096 | 23.835s | 27.531 | 40.488 |
| 연결 | gpu-2 | 60.001 | 23.816s | 23.348 | 24.214 |
| 해제 | gpu-1 | 59.960 | 23.832s | 23.320 | 24.137 |
| 해제 | software-1 | 45.985 | 23.834s | 27.261 | 27.685 |
| 해제 | software-2 | 45.567 | 23.833s | 27.254 | 28.035 |
| 해제 | gpu-2 | 60.001 | 23.833s | 23.500 | 24.430 |

## 재현과 증거

원시CSV·실제실행stdout/stderr·DRM샘플·observer C/so·실행/분석자료는 ignored `assets/local/issue-238-fps/`와 그`no-vnc/`에 보관했다. 로컬 저장소의 같은 경로에도 복사했다. 원본 유래 자료는커밋하지 않는다. 모든 조건은 같은 바이너리와 같은observer.so를 사용했음을SHA-256으로 확인했다.

- 제품commit: `60d2373bb1b58cf2c701b033086db06df325d51d`
- 실행파일SHA-256: `575e7b1fecd5836fd1d8a13a5194524cce10151646419008036ea58cd665344c`
- `present_trace.c` SHA-256: `d1a6c6573aaef484814282ff12520e632ba44cfb6810cf6b88aaf3a84b83f798`
- `present_trace.so` SHA-256: `01e971251f9c821960efbf8e1248bf20744cf88e25dbda23175f81bcd9cc5b36`
- `run_measurements.py` SHA-256: `4c9b4d63f4b50ab6f43236fd430a81b673fdfb0e31f8a756897be05963bff674`
- `vnc-comparison.json` SHA-256: `2e64fbfebedff2be4e696baa1a84ef800defcc8cd4ab679a70def3edf0638a51`

재현 시에는 기존 증거를 덮어쓰지 않도록 새 빈 디렉터리에 observer C/so·fun-01.txt·실행 스크립트를 복사한다. 같은 바이너리에 `SDL_VIDEODRIVER=wayland`, 관측기의 `LD_PRELOAD`와 `LEMMINGS_PRESENT_TRACE`를 지정하고 다음 앱 인자를 사용한다.

```text
--assets <dos-original> --hd --hd-pack <hd-runtime/atlas>
--fullscreen --replay <fun-01.txt> --steps 420
```

대조군만 `SDL_RENDER_DRIVER=software`를 지정한다. 실제 renderer와 출력 크기는 각 실행의 로그/CSV로 확인한다. 두 줄의 앱 인자는 한 명령으로 전달한다.

이번은 한PC/한레벨/약30초씩의 제한된 실측이다. 기존v0.3.3과의전후비교나120레벨/장기메모리/60Hz 물리표시 전체인수가 아니며#113 NOT READY는 유지한다. 새 코드/그래픽/드라이버 설정/릴리즈 변경은 없다. 측정 요구는완료했다.
