# G07-10 직업 전환·혼합 상호작용 계약

이슈: [#46](https://github.com/edwardkim/ohmylemmings/issues/46), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 전제와 할당 행렬

다른 PC의14커밋을 main `2e3e478`로 동기화했습니다. 개별8직업·Nuke의 코어/SDL 구현과 기존 golden을 유지하고 고정 actor ID 할당을 검사합니다. Pascal의 두 번째 후보는 `nil`입니다. 포인터 우선순위·두 후보 선택·RightClickGlitch는 [COMP-14/G08](issue-5_compatibility_baseline.md)의 별도 범위입니다.

표에서 W=Walking, J=Jumping, F=Falling, D=Digging, B=Bashing, M=Mining, U=Building, S=Shrugging, K=Blocking, O=OhNo, C=Climbing, H=Hoisting, A=Floating입니다. Splatting/Exploding은 모든 일반 할당을 거부합니다.

| 직업 | 기본 허용 행동 | 추가 거부 조건 |
|---|---|---|
| Climber | W/J/F/D/B/M/U/S/O/C/H/A | 이미 Climber, K; C/H는 이미 능력이 있어 거부 |
| Floater | W/J/F/D/B/M/U/S/O/C/H/A | 이미 Floater, K; A는 이미 능력이 있어 거부 |
| Bomber | W/J/F/D/B/M/U/S/K/C/H/A | 기존 타이머, O |
| Blocker | W/D/B/M/U/S | 아홉 영역 셀 중 기존 Blocker/force 코드 |
| Builder | W/D/B/M/S | 현재 애니메이션 머리 좌표가-5 미만 |
| Basher | W/D/M/U/S | 앞 강철·반대 일방향 |
| Miner | W/D/B/U/S | 발 강철·앞 강철·반대 일방향 |
| Digger | W/B/M/U/S | 발 강철 |

모두 재고0·제거 개체를 거부합니다. 거부는 개체/재고/객체 맵을 보존합니다. 정상 할당만1을 소비합니다. 영구 능력·폭파 타이머는 다른 작업 전환에도 남으며 Miner는 할당 시 발Y를1 늘립니다. Climber의 Shrugging 버그는 Walking으로 바뀌면서 Once 애니메이션을 유지합니다.

행렬은15행동×8직업×좌우2×11조건=2640사례입니다. 조건은 기본/재고0/두 능력/기존 타이머/발 강철/앞 강철/좌·우 일방향/Blocker 겹침/머리 경계 바로 위·정확한 경계입니다. Splatting에는 타이머0, Climbing/Hoisting에는 Climber, Floating에는 Floater를 설정해 복원 가능한 상태만 만듭니다. 제거48조합은 별도 Rust 테스트입니다.

## 실제 기준 실행과 재생

`build_interactions_reference.py`는 기존 Nuke 하네스에 자체 `assignment_probe.pas` 드라이버를 추가합니다. 고정 Game.pas의50개 함수·FloatParametersTable 및 기존 FPC 구문 변환은 그대로 상속합니다. 원문 출처 SHA는 `c1ba9ffda9e8e277f5c9b171df2bb611210d3326b9623b28fcd35426f4d6a448`입니다. 원본 함수/생성 실행 파일은 ignored `assets/local/`에만 둡니다.

행렬은 수락/수량·행동/발Y·프레임/종료·영구 능력·폭파 타이머·우산 표 인덱스·Walking Once·전체 객체 맵 해시를 대조합니다. Fallen/NewDigger/남은 벽돌은 현재 행동에서 쓰는 값만 정규화합니다. Pascal의 미사용 이전 작업 필드를 Rust 상태의 의미 있는 필드와 혼동하지 않습니다.

혼합14사례는 같은 경계의8직업 연속 할당·Blocker 이후 거부·영구 능력/타이머 보존·Pause 중 전환·굴착 발판 소실·Blocker 폭파·Builder/Miner/다른 개체 지형 공유·서로 다른 두 Bomber·중복 거부·Shrugging 버그/재할당·Climbing/Hoisting 중 폭파·반전 영역과 굴착·Pause/Nuke를 다룹니다. 각 사례의 실제 행동 도달 조건도 검사해 무효 입력만 재생한 사례가 통과하지 못하게 합니다.

렌더 주기1/7의 매 경계에서 상태·입력 수락·8재고·지형 추가/삭제·Blocker 저장/복원·전체 객체 맵 변경·능력/타이머/입자·Nuke를 실제 원문 실행과 비교합니다. 자체 지형/마스크/스프라이트로1400개의 SDL 캡처도 정확히 검사합니다.

## 재현

```sh
cargo run --quiet --locked --offline -p lemmings-core --example skill_matrix
python3 tools/lemmix/build_interactions_reference.py assets/local/my-interactions-reference --compiler /path/to/ppcx64 --units /path/to/units/x86_64-linux/rtl
python3 tools/check_skill_interactions.py --binary target/release/lemmings-desktop --reference assets/local/my-interactions-reference/reference
make check
```

`--record`는 실제 `--reference` 실행이 있어야 허용합니다. CI는 같은 합성 행렬/14재생 입력·상태 golden과 기준 어댑터 해시를 원본/FPC 없이 검사합니다. `--evidence assets/local/새폴더`는 원시 대조 자료를 보관합니다. [출처·해시](../../assets/manifests/skill_interactions.json)와 [보고서](../reports/issue-46_skill_interactions.md)에 실행 결과를 기록합니다.

이번에는 새 DOS 관측을 수행하지 않습니다. 기존 직업별 DOS 장면은 각 선행 이슈의 증거입니다. 객체로 생기는 Exiting/Drowning/Vaporizing·포인터 선택/원격 할당·출구/구출/승패/재시작·원본 패널/음향·전체 DOS 인증은 G08 이후에 검증합니다.
