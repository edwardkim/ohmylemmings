# G08-02 위험물·함정 계약

이슈: [#50](https://github.com/edwardkim/ohmylemmings/issues/50), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 판정과 사망 상태

기준은 기존 고정 Game.pas(`c1ba9ffda9e8e277f5c9b171df2bb611210d3326b9623b28fcd35426f4d6a448`)와 Styles.Base.pas입니다. `CheckLemmings`의 제거 검사→타이머 감소/만료→행동→행동이 요청한 객체 판정 순서를 유지합니다. 물133은 Drowning, 불134는 Vaporizing으로 전환합니다. Falling도 공중에서 진입하며 착지 Splat 직후라면 정지 방향과 취소된 타이머0을 보존합니다.

Drowning은16프레임 Once·발 기준10입니다. 종료 전에는 발Y를 보정하지 않고 앞8픽셀에 지형이 없을 때만 X를 한 픽셀 움직입니다. 화면/월드 끝에서 반전하거나 Y를0으로 제한하지 않습니다. Vaporizing은14프레임 Once·발 기준14이며 움직이지 않습니다. 둘 다 객체를 다시 검사하지 않고 종료 후 각각 Drowned/Vaporized로 한 번 제거합니다. 구출 수는 증가하지 않습니다.

진입 시 영구 능력·폭파 타이머·Blocker 저장 맵을 취소하지 않습니다. 물/불 상태에서 타이머가 끝나면 OhNo를 거치지 않고 즉시 Exploding으로 전환합니다. Drowning/Vaporizing 모두 Climber/Floater의 기존 ID 할당은 허용하고, Bomber는 Drowning만 허용합니다. 다른5직업과 제거 후 할당은 거부합니다. 일반 포인터 선택 계약은 별도 범위입니다.

## 함정 공유 상태와 갱신

작동형 함정 효과4는 객체 맵에 입구 제외 압축 인덱스0..15를 저장합니다. 호출자 소유 `TrapStates`의 해당 슬롯에 메타데이터·현재 프레임·작동 상태를 둡니다. `step_actors_with_hazards`는 ID 순서대로 갱신하고, 사용 가능한 함정에 처음 닿은 개체만 즉시 Trapped로 제거합니다. 같은 틱의 나머지는 지나갑니다. 타이머/행동/저장 맵을 임의로 지우지 않습니다.

DOSORIG의 `TriggeredTrapLemmixBugSolved` 설정에 따라 시작 프레임은0입니다. 모든 개체와 Nuke 처리 후 `advance_traps`를 논리 틱당 한 번 호출하여 첫 표시 프레임1로 진행합니다. 끝에서0/비작동으로 돌아가 다음 틱에 재사용합니다.1프레임 함정도 개체 배치가 끝나기 전에는 재사용하지 않습니다. 연속 애니메이션 메타데이터라면 비작동 중에도 진행합니다. Pause·렌더는 이 상태를 갱신하지 않습니다.

프레임 수0·잘못된 시작 프레임은 생성 시 거부합니다. 맵이나 저장된 Blocker 맵이 없는 함정/16번 이상을 참조하거나 ID 순서가 틀리면 배치 변경 전에 오류를 반환합니다. 이후 산술 오류는 기존 가변 배치와 같이 이전 성공 개체의 결과를 유지합니다.

화면 아래 Y>=160의 새 위험물/함정 판정은 원본 EXE2fe5의 DOS 발Y>=176 검사를 따라 비활성입니다. 고정 Lemmix의 안전용 패딩 맵은 이 범위를 허용하므로 해당 경계는 별도 DOS 지시문 기반 단위 테스트로 고정합니다.54함수의 실제 상태 대조 영역은 Y<160이며 이미 진입한 Drowning의 이동은 제한하지 않습니다.

## 네이티브와 기준 실행

리소스 해석은 데스크톱에 남기고 코어는 I/O·렌더링·자원 타입을 참조하지 않습니다. 원본 객체 슬롯을 압축 인덱스와 연결하여 작동 중인 실제 프레임을 그립니다. `hazards.jsonl`은 함정 `[인덱스,프레임,작동]`과 사망3종 누계를, `traps.bin`은16슬롯×`[프레임 수,시작 프레임,연속 여부]`를 기록합니다. 기존 추적/golden은 유지합니다.

별도 `build_hazards_reference.py`가 이전 출구 호스트에 원문 HandleDrowning/HandleVaporizing/UpdateInteractiveObjects를 추가해54함수·표를 실행합니다. 원문 함수는 변경하지 않으며 사망 사유·함정 메타데이터·추적·음향 stub은 자체 호스트입니다. 이 호스트의 입구 애니메이션 분기는 생략하고 기존 G06 화면 위상을 유지합니다. 자체 Hoisting 배열의 마지막 슬롯 용량도 수정했습니다. 이전 어댑터/golden은 변경하지 않습니다.

```sh
python3 tools/lemmix/build_hazards_reference.py assets/local/my-hazard-reference --compiler /path/to/ppcx64 --units /path/to/rtl
python3 tools/check_hazards.py --reference assets/local/my-hazard-reference/reference --record --evidence assets/local/my-hazards
python3 tools/verify_hazards.py assets/local/my-hazard-original --reference assets/local/my-hazard-reference/reference --xdotool /path/to/xdotool
make check
```

합성14사례/28재생은 물·불·공중 진입·죽어가는 개체의8직업 할당·함정 정상/경합/연속/재사용·Pause·물/함정과 Nuke를 다룹니다. CI는 원본/FPC 없이 같은 입력·상태 golden·어댑터 해시·SDL readback을 검사합니다. 원본 Fun09/17/29는 렌더1/7·실시간으로 대조하고 Fun17의 실제 Pause/Resume 입력은 별도로 재생합니다.

## DOS 관측과 범위

원본 DOS의 사망 완료 프레임은 Drowning0(순환), Vaporizing14입니다. Rust/Pascal의 유효 프레임15/13·종료 표지와 원시 값을 동일하다고 주장하지 않습니다. 원본 EXE 2415/2460의 사망 진행,30d6의 함정 작동 비트/제거,7d2a의 객체 프레임 갱신을 확인합니다. 별도 생성 RUSSELL seed와 원본50개 무수정 전제는 [#29](issue-29_dos_runtime.md)를 따릅니다.

레벨 코드 후보는 [고정 legacy 생성기](https://github.com/tomsoftware/lemmings.js/blob/82fb648e70595c5ee8fafce10a2133397eeacddb/codeGenerator.js)로 계산하고, 원본의 스크롤/최대 인원·게임 장면으로 검증합니다. 녹화는 코드 문자v와 충돌하지 않는 F11에 매핑합니다. 전용 X11 display·DOSBox·xdotool·PyAV14.2.0은 선택적 로컬 도구입니다.

```sh
DISPLAY=:92 python3 tools/observe_hazards_dos.py --profile trap --dosbox /path/to/dosbox --xdotool /path/to/xdotool --output assets/local/my-dos-trap
python3 tools/compare_hazards_dos.py --profile trap --recording assets/local/my-dos-trap --native assets/local/my-hazard-original/F17-single --output assets/local/my-trap-images
```

물/불은 `--profile water|fire`와 F29/F09를 사용합니다. 영상 대조는 전체320×160의 순서 있는 정확한 장면이며 비동기 수치 관측은 원자적 전체 DOS 상태를 인증하지 않습니다. 승패·시간/구출률·결과/재시작·원본 패널/미니맵·포인터/RightClickGlitch·음향은 후속입니다. [결과](../reports/issue-50_hazards.md)·[진행](../working/issue-50_hazards.md)·[출처/해시](../../assets/manifests/hazards.json)에서 추적합니다.
