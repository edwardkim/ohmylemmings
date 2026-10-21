# G11-05b Fun08~30 해법·반복·재시작 검증

이슈: [#96](https://github.com/edwardkim/ohmylemmings/issues/96) · 상위 [#93](https://github.com/edwardkim/ohmylemmings/issues/93) · 선행 [#94](issue-94_solution_replay.md) · [입력 목록](../../assets/manifests/solution_replay_fun.json).

## 검토 입력

기준 main `e51739a139f0fd31b644a0d2c44893647c43fc07`에서 Fun08~30의 성공23개와 Nuke 실패23개 입력을 직접 작성했다. 원본 수량·재고·구출 목표를 유지했다. 아래 수치는 해법 탐색 결과이며 최종 SDL/WASM·고정 기준 판정, 사례별 상태/맵/전체 RGB 체인 해시와 정확한 push/PR/main CI·병합/종료는 [GitHub #96](https://github.com/edwardkim/ohmylemmings/issues/96)에 기록한다. 원시 실행은 ignored `assets/local/issue-96-review/`에 보존한다.

| 레벨 | 구출/원본 목표 | 실제 종료 control_step | 성공 입력 SHA-256 |
| --- | ---: | ---: | --- |
| fun-08 | 79/76 | 1421 | `52ee5066332bb5bbe30a5a77cfed70c504d9cb995f0bc3cc419c740de151ac58` |
| fun-09 | 73/72 | 532 | `a46ea92bc415819135210869e1de7335b0527a0ea1ad86c130ac65712f63ad87` |
| fun-10 | 13/10 | 709 | `607e78c22cdaab45d98a01b6910dba148213dbc319ce570e51b403cb2f7430ee` |
| fun-11 | 59/50 | 2058 | `0067d545026e84f955fcf88bd96351bc321924400e756257067f920d59be424c` |
| fun-12 | 77/40 | 939 | `6782a3f02cdd05d9f2dbb93c437be7825c57746fec2efec8d977bdb6b939cd13` |
| fun-13 | 20/20 | 1751 | `beab7e21b1ce63e1bd79c99ab1650ec9210e974afd714f390da268fa18865aac` |
| fun-14 | 76/60 | 3082 | `648d1baa8357470c41467886c86ec63886a1d19a4ac2f6f4f3e7c9ba2f0e81af` |
| fun-15 | 43/40 | 3862 | `28c70ed3450ae622ab89126447117b19f29f210604d8ede9d8d6aac1a620fe14` |
| fun-16 | 59/50 | 2295 | `90cb4687ed5d3209719591ddbc65a33d293e27aff383787a202d71f2e2151094` |
| fun-17 | 20/20 | 642 | `a7b9ba51b9199f2404e4aec6fa58065e534602a0bfe525496b8c9e24317f858f` |
| fun-18 | 57/50 | 1457 | `0f78e13f732afbbd07016a9be40409011df0a10189d57cd89e038ee8de3a27b8` |
| fun-19 | 67/56 | 2992 | `660561ec1b45d3d44f8f6175d04a0eeee6219c59745053787121ed948870c1dd` |
| fun-20 | 44/30 | 1456 | `fdfe76527c249da4e04a45e4d47566ccc8f6f5b985958389e234f5c85247367f` |
| fun-21 | 75/48 | 2463 | `4d7517e069cd1b1f3191bc182552f18504ac03d7560ce7876684a84517717391` |
| fun-22 | 72/64 | 3323 | `29948774583d15e60135da71a1b2f46c2ddde0367edfd05f3dabb82a230047a1` |
| fun-23 | 53/20 | 3570 | `25f9752f565ea4bf33569b59fc088a280212a61fa5c48dc38a6916aa65e11baf` |
| fun-24 | 25/20 | 952 | `28526e8fff59fb7709d386b3a71e4b0fcad86a41adb635047b54680c5817bb8a` |
| fun-25 | 60/40 | 1528 | `56da311049f258ff775f85604520cfa676b97bf0addef932de4b0d41d1038339` |
| fun-26 | 2/2 | 930 | `61faa1b779145e95105e6218bbe49cf87140aa2ad78bfa0c02e9346c464722e3` |
| fun-27 | 29/25 | 1695 | `2fa8a3f37e25b12e32db4d1e5de2b55b7817afd16d5fc920a90834ff879eed5d` |
| fun-28 | 75/48 | 2813 | `f795e16905d44f181ced82ea330776f4eca3fb70bec4377d0e045507edfd6407` |
| fun-29 | 77/48 | 4082 | `8ed42b25745a333887eeb59d3ccd08ef4e975ea63c401dda867fcc49d5185753` |
| fun-30 | 58/40 | 522 | `958bfef0a27c93c646d758a81b30c7b92062fa78a2565750c869085fc73df9cc` |

실패 입력은 모두 `120 0 Nuke`이며 실제 Lost/원본 목표 미달을 요구한다. 성공 후 Nuke를 사용한 경우도 실제 원본 required 이상을 구출해야 한다. 목표/결과를 강제로 설정하는 명령은 허용하지 않는다. 무소비·거부된 직업 입력 자체를 성공으로 집계하지 않으며 실제 최종 수치로 판정한다.

## 반복과 재현

[기존 계약](../tech_insights/issue-94_solution_replay.md)의 엄격한 입력·매 경계 전체 v1 상태/actor 필드·terrain/brick/raw object map·전체 RGB/SDL 스트림/Canvas readback을 그대로 적용한다. 각 성공/실패를 SDL과 실제 WASM에서 렌더1/7/13·시간30/60/144Hz로 각각3회 실행한다. SDL 실제 Restart는 렌더37/101/257로 초기 자원/재고/맵/입력 폐기와 무입력 prefix를 확인하며, WASM은 실제 Restart의 음원 세대/빈 큐/sequence0/reset을 확인하고 같은 입력으로 재완주한다. 종료 뒤4 guarded probe도 고정 상태를 요구한다.

재현은 기존 계약의 추출 예제에서 manifest를 `assets/manifests/solution_replay_fun.json`으로, 새 ignored 입력/증거 폴더로 바꾼다. `source assets/local/dev-tools/env.sh` 후 `python3 tools/solution_replay.py --assets assets/local/dos-original --cases <ignored-cases.json> --evidence <new-ignored-directory> --workers 3`을 실행한다. 기존 증거를 덮어쓰지 않는다.

## 검증 도구의 긴 실행 처리

Fun15·23의 긴 브라우저 변형은 실행 중인데도 회차 완료 보고 전120초가 지나 stall로 중단됐다. 실패 원시 기록과 이미 완료한 변형을 보존하고, 미완료 사례만 새 증거 폴더에서 다시 실행한다. 기존/재시도 WASM 바이트와 원본50파일 크기/SHA, dense 기대값을 확인한다. 다른 비교 조건은 유지한다.

이제 상태/맵/화면 검사를 마친 뒤100 delivery마다 검증된 경계 수를 heartbeat로 전송한다. heartbeat는 회차 완료나 PASS 결과에 들어가지 않는다.120초 idle과 전체1,800초 기한을 유지하며 알 수 없는 HTTP 경로는 idle을 연장하지 않는다. 시계 단위 TDD2개는 긴 진행의 idle 연장/전체 기한 보존과 미등록 경로의 stall을 확인한다. 기존 초기 단위4개도 유지한다. 검토 소스와 실행 시작 시점이 다른 경우 원래 캡처의 소스 해시·실제 사용한 JS와 재시도 해시를 각각 기록한다.

## 기준과 범위

고정 Lemmix `40e9bc34451f0e9127fd53b290a3877e700d143d`의56함수/표와 기존 DOS 종료 경계 adapter의 지원 상태·지형 변경·최종 결과를 대조한다. 실제 SDL/WASM 일치는 새로운 DOS 영상 관측이나 전체 DOS 내부 상태/실시간 성능 인증이 아니다. 안정된 actor ID 입력은 실제 DOM 포인터·키 입력 인증과 구분한다.

경로 구상의 일부는 [Asthalis 공략](https://asthalis.fr/fichiers/pc/lemmings/lemmings_solution.pdf)(2025-01-23 rev1, 2026-10-21 열람)을 참고했다. 공략 문장·리플레이는 복제하지 않았으며 모든 입력 시각/ID는 직접 작성해 실행했다. 원본/파생 이미지·맵·음원은 공개하지 않는다. 제품 Rust 물리·렌더·UI·음향 변경이나 의존성 변경은 없어 새 UI 스크린샷은 추가하지 않는다.

#94의 첫7개와 이번23개를 합쳐 최종 판정 후 Fun30개/전체120개를 집계한다. 다른 난이도90개, COMP-01~18 전체 재감사와 성능·G11/G12 마감은 #93/#1에 남긴다. 미완료 검증을 통과로 합산하지 않는다.
