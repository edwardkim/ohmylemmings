# G08-01 객체·출구/구출 계약

이슈: [#48](https://github.com/edwardkim/ohmylemmings/issues/48), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 객체 맵과 판정 순서

고정 Game.pas SHA-256은 `c1ba9ffda9e8e277f5c9b171df2bb611210d3326b9623b28fcd35426f4d6a448`입니다. `Start`는 입구를 별도 배열로 옮깁니다. `InitializeObjectMap`은 그 나머지 객체의 압축 인덱스0..15만 처리하며 뒤 객체가 앞 객체를 덮습니다. 16번 이후 객체는 그리더라도 트리거가 없습니다. 4픽셀 격자·원점(-16,-16)·415×47 맵에서 출구 효과1은129입니다. 기존 맵 생성의 입구 제외·15/16 경계를 테스트로 고정했습니다.

`CheckLemmings`의 순서는 제거 검사→타이머 감소/만료→행동→행동이 요청한 객체 판정입니다. 발 좌표의 출구는 **Falling만 거부**합니다. 착지로 Walking 또는 Splatting이 된 경우 같은 틱에 Exiting으로 전환합니다. 치명적 착지의 정지 방향과 이미 취소된 폭파 타이머0을 보존합니다. Floating은 공중 출구에도 들어갑니다. 진행 중인 Splatting/Exiting은 객체를 다시 검사하지 않습니다.

## 애니메이션·제거·할당

Exiting은8프레임 Once·발 기준13이며 진입 프레임0 이후8번의 논리 갱신에서 구출됩니다. 정규화 상태는 frame7/end=true/removed=Rescued입니다. 구출은 제거의 부분집합입니다. `ActorTick.rescued`와 `removed`가 함께1 증가하고 이후 호출은 다시 집계하지 않습니다. 세션은 보존된 안정 ID 기록의 Rescued 수를 HUD/별도 `exit.jsonl`에 출력합니다. Pause와 렌더 주기는 진행에 영향을 주지 않습니다.

출구 진입은 폭파 타이머·영구 능력·Blocker 저장 맵을 취소하지 않습니다. 타이머가 먼저 만료되면 OhNo/폭발로 진행할 수 있습니다. Exiting 중 Climber/Floater/Bomber는 기존 중복/재고 조건에 따라 허용하고 다른5직업은 거부합니다. 제거 후에는 모두 거부합니다. OhNo에서 출구로 바뀐 Blocker의 저장 맵은 원문대로 유지하며 구출 시 임의로 복원하지 않습니다. 정지 Exiting에서 타이머가 만료된 OhNo/Exploding도 상태 복원을 허용합니다.

## 실제 기준과 재현

`build_exit_reference.py`는 기존50함수 호스트에 원문 `HandleExiting`을 그대로 더해51개 함수/표를 실행합니다. 변경은 자체 호스트의 메타데이터·저장 수/ID 추적·toolbar stub입니다. 기존8직업/Nuke/혼합 회귀 어댑터와 golden은 바꾸지 않습니다.

```sh
python3 tools/lemmix/build_exit_reference.py assets/local/my-exit-reference --compiler /path/to/ppcx64 --units /path/to/units/x86_64-linux/rtl
python3 tools/check_exit.py --reference assets/local/my-exit-reference/reference --record --evidence assets/local/my-exit-check
python3 tools/verify_exit.py assets/local/my-exit-original --reference assets/local/my-exit-reference/reference --xdotool /path/to/xdotool
make check
```

합성9사례×렌더1/7은 정상·Pause·영구 능력/타이머·Nuke·8직업 허용/거부·제거 후 입력·Splat 우선순위·공중 Floating·치명적 착지 타이머 취소를 검사합니다. CI는 원본/FPC 없이 같은 상태 golden·어댑터 해시·1770개 SDL readback을 재검사합니다. `--record`는 실제 기준 실행을 요구합니다. 원본 Fun01은 Digger 입력60 이후10마리 구출을 렌더1/7·실시간·실제 D키/마우스로 대조합니다.

## DOS 관측의 해석과 후속 범위

원본 EXE의20f8은 행동 비트0x2000을2446으로 분기하고,2455에서 DS:46 구출 수를 늘린 뒤2459에서 개체+26을255로 만듭니다. DOS는 마지막 프레임을0으로 감아 제거하므로 정규화된 frame7/end=true와 원시 프레임을 동일하다고 주장하지 않습니다. DS:42는 순회 임시 카운터이므로 생존 수로 읽지 않습니다. 별도 생성 RUSSELL seed·원본50개 해시·read-only 비동기 관측의 전제는 [#29](issue-29_dos_runtime.md)를 따릅니다.

물/불/함정, 승패·시간 종료·구출률·재시작, 원본 패널/미니맵, 포인터 우선순위/RightClickGlitch, 음향·전체 DOS 상태 인증은 후속입니다. 자세한 결과는 [보고서](../reports/issue-48_exit_rescue.md), [진행 기록](../working/issue-48_exit_rescue.md), [출처·해시](../../assets/manifests/exit_rescue.json)에서 추적합니다.

전용 X11 display에서 DOSBox/xdotool을 준비한 경우의 새 관측/영상 대조 명령입니다. PyAV14.2.0은 선택적 로컬 의존이며 CI에는 필요하지 않습니다.

```sh
DISPLAY=:92 python3 tools/observe_exit_dos.py --dosbox /path/to/dosbox --xdotool /path/to/xdotool --output assets/local/my-exit-dos-1
DISPLAY=:92 python3 tools/observe_exit_dos.py --dosbox /path/to/dosbox --xdotool /path/to/xdotool --output assets/local/my-exit-dos-2
python3 tools/compare_exit_dos.py --recording assets/local/my-exit-dos-1 assets/local/my-exit-dos-2 --native assets/local/my-exit-original/F01-single --output assets/local/my-exit-images
```
