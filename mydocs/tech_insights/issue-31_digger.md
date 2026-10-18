# G07-01 Digger 계약과 재현

이슈: [#31](https://github.com/edwardkim/ohmylemmings/issues/31), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

[진행 기록](../working/issue-31_digger.md) · [검증 보고서](../reports/issue-31_digger.md) · [일지](../orders/20261019.md#issue-31) · [해시 증거](../../assets/manifests/digger.json).

## 기준과 범위

고정 Lemmix `40e9bc34451f0e9127fd53b290a3877e700d143d` [Game.pas](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas)의 AssignDigger/HandleDigging/DigOneRow/Transition/HandleLemming과 Styles.Base.pas의 Digging(16프레임, foot 8/12, Loop)를 사용합니다. [원문 식별](../../assets/manifests/reference_sources.json)과 [기존 시뮬레이션 계약](issue-26_state_replay.md)을 따릅니다. 원문 사본/추출물/실행 파일은 assets/local에만 둡니다.

이번 코어 할당은 안정된 ID의 살아 있는 Walking 개체만 지원합니다. 수량 0·없는/제거된 ID·다른 상태·발 위치 강철은 거부하며 성공 때만 1을 소비합니다. 기준 함수의 Building/Bashing/Mining/Shrugging 전환과 secondary/right-click 후보 규칙은 다른 직업/선택 통합 때 검증합니다. 전체 AssignSkill/UI 우선순위 인증이 아닙니다.

## 갱신 순서

- 경계의 순서 있는 입력 → 시계/생성 → 엄격한 ID 순 개체 갱신입니다. Digger 요청도 일시정지 중 수락할 수 있으며 굴착은 다음 진행 틱에 시작합니다.
- 할당은 Digging/frame0/new_digger=true로 전환합니다. 최초 갱신에서 x-4..x+4의 y-2·y-1을 먼저 지웁니다. 최초 프레임은 증가하지 않습니다.
- 이후 프레임은 (frame+1)%16입니다. 프레임 0·8에서 발 y를 1 증가시키고 이전 발 y의 아홉 픽셀을 삭제합니다. 한 픽셀이라도 있으면 지지로 간주합니다.
- 이전 줄에 지지가 없으면 Falling(기본 fallen=3), 지지가 있고 새 발 위치가 강철이면 Walking으로 전환합니다. 개별 강철 픽셀 보호를 추가하지 않습니다. y>163이면 현재 줄을 지우기 전에 제거합니다.
- 음수 삭제 줄은 0으로 클램핑하고 x는 마스크 범위로 클리핑합니다. 중복된 삭제 좌표는 한 번만 적용합니다. 산술 오류는 그 개체와 삭제 픽셀을 보존합니다.

새 `advance_mut`/`step_actors_mut`는 변경 지형을 사용합니다. 뒤 개체가 앞 개체의 구멍을 같은 틱에 봅니다. 배치의 ID 오류는 전체 갱신 전에 거부합니다. 뒤 개체의 산술 오류는 앞의 성공한 갱신까지 되돌리지 않습니다. 기존 불변 `step_actors`의 전체 preflight/원자성은 유지하며 Digger는 MutableTerrainRequired로 거부합니다. 호출자 제공 steel 뷰는 유효한 객체 맵의 발 위치 판정입니다. Fun01 앱의 강철 영역 투영은 전체 트리거/객체 맵 구현을 대신하지 않습니다.

## 화면과 기록

왼쪽 클릭으로 현재 후보에게 Digger를 요청하며 ID·경계·sequence를 입력 파일에 기록합니다. `60 0 AssignDigger 0`으로 F01을 재생합니다. 수량은 레벨의 여덟 번째 skill 값이며 HUD에 표시합니다. 지형 opacity를 변경한 뒤 객체를 합성하므로 구멍 뒤 객체도 해당 그리기 플래그를 따릅니다. 원본 16×14 Digger 스프라이트/발 8/12를 사용합니다.

기존 `trace.jsonl`의 G05 필드를 유지하고 `digger.jsonl`에 수량·new_diggers·지형 FNV·정렬된 삭제 인덱스를 매 경계 기록합니다. terrain-initial.bin에 삭제 목록을 적용하면 terrain-final.bin의 모든 바이트가 같아야 합니다. FNV만으로 픽셀 일치를 판정하지 않습니다. 고정 golden은 전체 상태와 삭제 목록의 정규 JSON SHA-256입니다.

## 실제 기준 실행과 DOS 관측

[추출기](../../tools/lemmix/build_digger_reference.py)는 원문/애니메이션 해시를 먼저 확인하고 기존 15개와 Digger 3개 함수 원문을 FPC 3.2.2로 컴파일합니다. G05와 같은 두 inline 변수 선언만 FPC 구문에 맞춥니다. 호스트의 바이너리 지형/강철·미니맵/음향/툴바/기록기 스텁·ID 선택 범위를 provenance에 명시합니다. 전체 Lemmix 실행은 아닙니다.

원본 DOS는 [#29 seed](issue-29_dos_runtime.md)와 [#30 기본 설정](issue-30_dos_differences.md)으로 실행합니다. EXE/50개/메모리를 변경하지 않고 자신이 실행한 DOSBox 자식의 /proc/pid/mem을 O_RDONLY로 읽습니다. DS:10/12 커서, DS:7f 수량, DS:82 선택, DS:88의 0x2d 크기 actor 슬롯 중 위치/상태/프레임만 읽습니다. 프로그램 0x2039/0x210c(목록/stride), 0x3e2a(커서 선택), 0x332d(할당), 0x42ac(키보드 이동) 해석을 증거 해시로 고정합니다. DOS 좌표에는 16픽셀 테두리가 있습니다.

준비 키와 F10을 입력한 뒤 첫 Walking을 관측하여 Space를 누릅니다. counter61에서 위치744/86·Walking/frame0·수량10, 다음 counter62에서744/87·Digging/frame0·수량9를 관측했습니다. 이는 native snapshot60→61의728/70→728/71과 대응합니다. 일시정지 중에도 DOS startup counter는 증가하므로 전체 제어 호출 순번으로 일반화하지 않습니다. 폴링은 비동기이며 같은 루프 중간의 값도 포함합니다.

전체 320×160 비교는 정확한 2배 픽셀/스캔라인을 검증하고 양쪽을 RGB565 코드로 비교합니다. 커서를 HUD로 옮긴 뒤 100/170/340의 세 전체 장면을 검사하며 위치 이동·커서 마스크·색 허용 오차를 사용하지 않습니다. 영상 인덱스가 절대 DOS 입력 틱이라는 판정은 하지 않습니다.

## 재현

원본/FPC 없는 회귀:

```sh
make check
```

실제 기준 코드와 원본 확인에는 기존 SDL2/FPC/원문·DOSBox/PyAV/X11 준비가 필요합니다. 모든 원본 파생 출력은 새 assets/local 경로를 사용합니다.

```sh
python3 tools/lemmix/build_digger_reference.py assets/local/digger-reference-new --compiler /absolute/path/ppcx64 --units /absolute/path/rtl
python3 tools/check_digger.py --reference assets/local/digger-reference-new/reference --record
python3 tools/verify_digger.py assets/local/digger-native-new --reference assets/local/digger-reference-new/reference --xdotool /usr/bin/xdotool
python3 tools/observe_digger_dos.py --dosbox /usr/bin/dosbox --xdotool /usr/bin/xdotool --output assets/local/digger-dos-new
PYTHONPATH="$PWD/assets/local/issue-28-tools/python" python3 tools/compare_digger_dos.py --recording assets/local/digger-dos-new --native assets/local/digger-native-new/F01-single --output assets/local/digger-comparison-new
```

--record는 실제 참조를 실행해 추적 golden을 갱신하는 개발 명령입니다. 변경 diff와 참조 provenance를 함께 검토합니다. DOS 입력과 네이티브 클릭은 창 포커스가 필요하므로 전용 X11에서 순차 실행합니다. 원본 없는 CI는 기록된 참조 해시·전체 삭제 이력·SDL readback과 기존 회귀를 사용합니다.
