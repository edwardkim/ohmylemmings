# G08-06 포인터 선택·RightClickGlitch 계약

이슈: [#58](https://github.com/edwardkim/ohmylemmings/issues/58) · [작업 기록](../working/issue-58_pointer_selection.md) · [결과](../reports/issue-58_pointer_selection.md)

## 기준과 모듈

Lemmix `40e9bc34451f0e9127fd53b290a3877e700d143d`의 `Game.pas` `PrioritizedHitTest`(3595)·`ProcessSkillAssignment`(3672)·`AssignSkill`(2024)와 기존8직업 할당/Transition을 기준으로 합니다. `Styles.Base.pas`의18행동 foot 좌표, `GameScreen.Player.pas`(590)의 hotspot `(-3,+2)`, `Game.SkillPanel.pas`(501)의 행동/겹침 수와 `Base.Strings.pas`의 영문 명칭을 사용합니다. `Dos.Consts.pas`의 RightClickGlitch 설명과 [COMP-14](issue-5_compatibility_baseline.md)를 따릅니다. 원문/호스트/증거 해시는 [매니페스트](../../assets/manifests/pointer_selection.json)에 기록합니다.

`lemmings-core::PointerSelection`은 개체 상태·월드 hotspot·우클릭 상태만 받는 무할당 no_std 순수 모델입니다. SDL·렌더·리소스에 의존하지 않습니다. 네이티브 `pointer.rs`는 화면/카메라 변환·표시 명칭·안정 ID 명령 변환을 맡고 `input.rs`는 SDL 이벤트, `Session`은 제어 경계의 실제 기술 할당을 맡습니다. 렌더 호출은 선택 기억을 바꾸지 않습니다.

## 선택 규칙

- Walking의 논리 커서 범위는 발 좌표 기준 X `-5..=7`, Y `-12..=0`입니다. 다른 행동은 고정 foot 좌표로 같은13×13 inclusive 사각형을 이동합니다. Exploding만 FootX16이고 나머지는8입니다.
- FootY: 기본10, Building/Mining/Exiting13, Digging/Climbing/Hoisting12, Floating16, Vaporizing14, Exploding25입니다.
- 제거된 개체/폭발 입자는 선택 대상에서 제외합니다. 실제 스프라이트의 불투명 픽셀 모양은 판정에 사용하지 않습니다.
- Blocking·Building·Shrugging·Bashing·Mining·Digging·OhNo가 우선 분류입니다. 분류별로 안정 ID 순 목록의 마지막 적중 개체를 유지합니다. 거리순이 아닙니다.
- 일반 클릭은 우선 분류를 먼저 사용하고 없으면 일반 분류를 사용합니다. 우클릭은 일반 분류만 사용합니다. 겹침 수는 두 분류 전체입니다.
- 마지막 일반 개체를 기억합니다. 빈 공간·우선 개체만 있는 위치로 이동해도 기억은 남습니다. 우클릭 중 우선 개체만 적중한 상태에서 왼쪽을 누르면 기억한 개체에 원격 할당합니다. 적중 자체가0이거나 기억이 없으면 명령을 만들지 않습니다.
- 실제 표시 대상의 Climber/Floater 영구 능력이 행동 명칭보다 우선하며 둘 다면 ATHLETE입니다. `명칭 + 공백 + 겹침 수`를14칸에 표시합니다. 우클릭 중 우선 개체만 적중하면 표시/선택 커서는 비어도 원격 할당은 가능합니다.

## 할당·죽은 대상·재생

Builder/Basher/Miner/Digger는 우선 대상의 현재 행동이 해당 직업으로 바뀔 수 없을 때만 일반 대상을 사용합니다. 수량 부족·강철·일방통행 등 실제 할당 실패 뒤에 다른 개체를 재시도하지 않습니다. Builder 머리 경계와 Digger ObjectBelow 강철 검사는 원문처럼 우선 대상에 적용하며 대체 대상에는 재적용하지 않습니다. 이 차이는 별도 코어 API로 명시합니다.

선택 시점의 안정 ID 두 개와 기술·glitch 여부를 저장하고 같은 경계의 앞선 명령을 적용한 뒤 현재 상태로 대체 여부를 결정합니다. 단일 대상의 기존 `AssignDigger ID` 등 형식은 유지합니다. 필요한 경우 아래 확장 형식을 사용합니다.

```text
105 0 AssignPointer Builder 0 1 false
105 1 AssignPointer Blocker 0 - true
```

형식은 `CONTROL_STEP SEQUENCE AssignPointer SKILL PRIMARY SECONDARY|- true|false`입니다. ID는 원시 포인터로 재선택하지 않습니다. 일반 할당은 제거된 대상을 거부합니다. 명시적 RightClickGlitch만 제거 표시를 보존한 채 원래 행동/수량/객체 검사를 실행합니다. 함정은 마지막 행동/위치를 남기므로 Blocker가 성공하면 수량이 줄고 필드가 남습니다. 제거된 개체는 계속 갱신에서 제외되므로 부활하거나 필드를 복원하지 않습니다. 화면 하단 제거 뒤 Builder/Basher/Blocker로 바뀐 기록도 `Actor::from_state`로 복원할 수 있습니다.

## 플랫폼 경계

게임 영역은320×160입니다. 패널·미니맵·letterbox는 개체를 할당하지 않습니다. 이동과 우클릭 Down/Up은 선택 상태에 반영하며 FocusLost와 새 레벨은 기억/버튼 상태를 초기화합니다. 카메라 변경과 논리 틱 뒤에도 선택을 갱신합니다. Pause의 기존 할당 정책·재생 중 실시간 게임 입력 차단·결과 화면 동작은 유지합니다. SDL 시스템 커서 Arrow/Crosshair/Hand를 사용하며 dummy처럼 시스템 커서를 지원하지 않는 드라이버는 HUD만 표시합니다. 원본 커서 비트맵 일치 판정은 이번 범위 밖입니다.

캡처 `pointer.jsonl`은 제어 경계별 논리 화면 좌표·right·적중 수·선택/일반 ID·카메라·14칸 문구를 기록합니다. 표현 기록은 논리 세션 상태 해시에 포함하지 않습니다.

## 재현과 한계

```sh
source assets/local/dev-tools/env.sh
make check
python3 tools/lemmix/build_pointer_reference.py assets/local/my-pointer-reference \
  --compiler /usr/lib/fpc/3.2.2/ppcx64 --units /usr/lib/fpc/3.2.2/units/x86_64-linux/rtl
python3 tools/check_pointer.py --reference assets/local/my-pointer-reference/reference \
  --evidence assets/local/my-pointer-check
# 독립 X11 DISPLAY에서 실행:
python3 tools/verify_pointer.py assets/local/my-pointer-x11
python3 tools/verify_pointer.py assets/local/my-pointer-trap --trap
python3 tools/verify_pointer.py assets/local/my-pointer-original --assets assets/local/dos-original
```

원문 본문53개를 유지하고 선택18행동 경계/겹침44604개·원격2개·Builder/Digger 대체3888개를 실행합니다. 기존 Nuke 호스트가 다루지 않던 Exiting/Vaporizing의 선택 foot 메타데이터는 원본 Styles 표의18행동 해시/값을 검증해 어댑터에서 보완합니다. 대체 행렬은 직접 두 대상을 전달하는 합성 할당 검사이며 모든 행이 실제 포인터로 동시에 겹친다는 뜻은 아닙니다.

예비 실행에서 기억이 아예 없는 우클릭은 고정 Pascal `ProcessSkillAssignment`의 begin/end 누락으로 nil 할당을 호출해 Access violation을 냈습니다. 네이티브는 이 상태를 명령 없음으로 처리하고 단위 테스트로 고정합니다. 원문 직접 비교는 유효한 기억이 있는 살아 있는 대상/함정 대상2건이며 원문 충돌 재현을 성공 사례로 세지 않습니다.

이 작업은 고정 Lemmix 계약을 직접 실행한 비교입니다. 새 DOS 실행, DOS의 모든 가장자리 clipping/원시 메모리 상태, 원본 커서 아트, 전체 레벨 호환성 인증은 포함하지 않습니다. DOS EXE `3e2a..3f82`의 두 분류/마지막 포인터 보존과는 정적으로 대조했으며 원자적 DOS 상태 일치로 표현하지 않습니다. G08 한 레벨 통합 완주와 음향·웹은 후속입니다.
