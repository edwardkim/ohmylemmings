# G11-04 전체120레벨 초기 재생 계약

이슈: [#91](https://github.com/edwardkim/ohmylemmings/issues/91) · [G11 계획](../plans/issue-85_compatibility_validation.md) · [진행](../working/issue-91_initial_replay.md) · [결과 기록](../reports/issue-91_initial_replay.md).

기준 main `dfaa1fafb55769f64c585c49105186f58c7630e4`, 네 난이도×30개를 대상으로 한다. [난이도 연결](issue-87_rank_selection.md)과 [특수4장면 연결](issue-89_special_scenes.md) 다음 절편이다. 340은 무입력 초기 재생 길이이며 유효 해법 완주 길이가 아니다. 기존 #85/#87/#89의 시점별 증거는 수정하지 않는다.

## 호스트 경계와 기록

[initial_replay.py](../../tools/initial_replay.py)는 새 ignored 증거 폴더에서 실제 `lemmings-desktop`과 actual WASM `WebGame`을 실행한다. 게임 규칙이나 시뮬레이션 상태 직렬화 v1은 변경하지 않는다. 기존 Web 전체 진단 문자열을 공통 `diagnostics::state`로 옮겨 SDL `full.jsonl`에도 같은 형식을 기록한다. 모든 개체 필드·예약 입력과 기존 직업/결과/트랩/Nuke 상태가 포함된다.

SDL의 선택적 `--capture-stream`은 모든 실제 렌더의 RGB를 `(probe:u64le, length:u32le, bytes)`로 stdout에 출력한다. 끝은 `u64::MAX,length0`이며 뒤에 기존 JSON 실행 요약이 온다. 크기·순서·절단·종료 누락을 검사한다. SDL 출력은 물리 화면 크기·정수 배율·letterbox 전체 픽셀을 Rust에서 원래 논리 RGB와 비교한 뒤 기록한다. 원본 파생 RGB는 해시와 최초 초기 프레임만 로컬에 보존해 대량 PPM 기록을 줄인다. 기존 파일 캡처와 기본 플레이 경로는 유지한다.

`--elapsed-replay FILE`은 양의 정수 마이크로초를 실제 `Session::due`에 전달한다. 두 옵션은 `--fast --steps --capture`를 요구하며 결과 전환/데모와 함께 사용할 수 없다. `--fast`라 장치 음향은 끄며 실시간 성능 통과로 계산하지 않는다. SDL은 기존 guarded tick·패널/포인터·렌더 경로를 사용한다.

각 레벨을 플랫폼별 세 번 실행한다. 렌더 주기는1/7/13, 시간 전달은30/60/144Hz의 정수 마이크로초 분할이다. Web은 실제 `FramePump`와 상한3인 tick budget을 사용하고 숨김/복귀 시의 큰 시간 점프도 배제한다. 입력 파일은 비어 있고 SHA-256은 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`이다.

초기0부터340까지341개 probe를 기록한다. 결과가 먼저 나오면 control_step은 멈추고, SDL은 guarded `Session::tick`을, Web은 `WebGame::step(1)`을 다시 호출해 실제 상태 고정을 확인한다. frozen probe는 새 simulation tick으로 계산하지 않는다. `probe`와 `control_step`은 별도 값이다. probe 누락/중복과 종료 뒤 상태·맵 변경은 실패한다.

## 비교 범위

- 모든 probe의 전체 v1 상태와 지형·다리·raw 객체 맵을 SHA-256으로 비교한다. SDL 맵은 기존 실제 셀 변경 기록에서 복원하고 최종 원시 맵과 다시 대조한다.
- SDL 렌더1은341개 전체 프레임을 내며 다른 SDL 실행의 모든 렌더를 이 기준과 비교한다. WASM의 모든 실제 렌더 RGB와 불투명 Canvas readback도 같은 경계의 SDL 기준과 비교한다. 렌더1/7/13의 프레임 수는 다를 수 있다.
- `asset_info()`와 `materials()`는 초기 헤더·목표/재고·입구·카메라·게임 팔레트와 강철/일방향 셀을 노출한다. 원본 헤더/재고는 고정120개 카탈로그와, 전체 지형/정적 객체는 기존 독립 검증 해시와 대조한다.
- [initial_reference.py](../../tools/initial_reference.py)는 고정 DAT 해제만 재사용한다. 원시 GROUND·레벨 레코드·객체 플레인에서 Python으로 독립적으로 초기 객체 합성, 트리거/강철/일방향/compact 객체 맵을 계산한다. first-frame 전체 RGBA를 #19 해시로 검증한 후 연속 객체 +2의 시작 위상을 적용한다. 320×160 게임 상부 전체 픽셀과 초기 SDL 화면을 대조한다.
- HUD를 포함한320×200 전체 프레임은 SDL/WASM에서 대조한다. HUD의 새 독립 전체 원작 oracle이나 전수 DOS 관측 증거로 부르지 않는다. 카메라·초기 수치·팔레트는 독립 목록/정적 증거와 별도로 검증한다.

실패 시 레벨/렌더/probe/상태 또는 맵 필드와 최초 차이를 남긴다. 누락·중복120 ID, 손상된 스트림, 개체의 중첩 필드·맵·상태 고정 오류를 원본 없는 테스트로 검증한다. 초기 진입·반복 결정론 통과와 G11-05의 전수 해법/COMP·G11-06 성능을 분리한다.

## 재현

```sh
source assets/local/dev-tools/env.sh
make check-rust check-python check-wasm
python3 tools/initial_replay.py --smoke
python3 tools/initial_replay.py --original-assets assets/local/dos-original --evidence assets/local/NEW_INITIAL_REPLAY --workers 3
python3 tools/initial_reference.py --assets assets/local/dos-original --evidence assets/local/NEW_INITIAL_REFERENCE --native assets/local/NEW_INITIAL_REPLAY/native
```

두 원본 검사는 병행할 수 있다. 독립 대조는 각 native 결과가 준비될 때까지 기다린다. 원본 검사는 새 ignored 경로를 요구하며 기본 CI의 `make check-web`은 창작5개(일반1·특수4개)만 실행한다. 120개 실행은 원본50파일의 전후 크기/해시를 확인하고 빌드/소스·입력·개별 실행·정적 참조·최종 요약의 해시와 환경을 남긴다.

[공개 실행 계약·출처](../../assets/manifests/initial_replay.json)는 코드/창작 증거를 고정하며 전수 실행의 최종 판정·레벨별 증거 해시·게시/병합 CI는 [#91](https://github.com/edwardkim/ohmylemmings/issues/91)에 기록한다. 전수 검증이 남아 있는 동안 PR은 초안으로 유지한다.
