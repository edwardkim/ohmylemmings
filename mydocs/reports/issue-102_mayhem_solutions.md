# G11-05e Mayhem01~30 해법·반복·재시작 검증

이슈: [#102](https://github.com/edwardkim/ohmylemmings/issues/102) · 상위 [#93](https://github.com/edwardkim/ohmylemmings/issues/93) · 선행 [#100](issue-100_taxing_solutions.md) · [입력 목록](../../assets/manifests/solution_replay_mayhem.json).

## 입력과 출처

기준 main `1a6419cf2d7a1527995ecc08b7ecdb8f2a50258f`에서 성공30개/실패30개 입력을 확보했다. 성공8개(11·13·15·20·22·24·25·28)는 프로젝트 작성,22개는 공개 DOS Lemmix 리플레이의 기능 기록 변환이다. 원본 인원·재고·구출 목표를 유지한다. 아래는 해법 확보 결과이며 최종 SDL/WASM·고정 기준·정확한 CI/병합 판정은 [#102](https://github.com/edwardkim/ohmylemmings/issues/102)에 사례별 증거 해시와 함께 기록한다.

| 레벨 | 구출/원본 목표 | 실제 종료 control_step | 성공 입력 출처 | 입력 SHA-256 |
| --- | ---: | ---: | --- | --- |
| mayhem-01 | 75/72 | 4040 | 리플레이 변환 | `596ba92d15ce3dd56309215e3f51aa00efdf4c3d489c13595f88ac58a2cf4db4` |
| mayhem-02 | 72/72 | 2247 | 리플레이 변환 | `2cb67b85635ad672e8752507617a6af4c9110da6b654044525416c0a0601a0b8` |
| mayhem-03 | 25/25 | 871 | 리플레이 변환 | `e87b854d4faf8a4ac27ceb80af4b620423fda172e0db2e6769f8480d0d4118b4` |
| mayhem-04 | 50/40 | 816 | 리플레이 변환 | `5e002112881f2aa4a4ec9b2542da95b0c758f7cc5e2b2480fa3702f87b85f1ac` |
| mayhem-05 | 76/60 | 2238 | 리플레이 변환 | `7346966747409749963f55060aa5e5f9701d40ae3a6e233a79f20d755a079e57` |
| mayhem-06 | 75/75 | 3202 | 리플레이 변환 | `27616d344d4286eabd95de8a2f333581ab173b3d3cd01e8ec8b679e863639c2c` |
| mayhem-07 | 50/45 | 1796 | 리플레이 변환 | `2d84ad1350a80b0046b54db217ae253c7f27e6a11b855b0b42dec0d0e36036d5` |
| mayhem-08 | 72/72 | 1861 | 리플레이 변환 | `092daec0428c8ceed0448ef7401df662ac71d7a192a12fab277f2192945ddfb2` |
| mayhem-09 | 79/79 | 3373 | 리플레이 변환 | `089fa887284c59150e76c6e2d0645b1d6ac2fede3c00739cb23f2b50f687270f` |
| mayhem-10 | 51/50 | 2347 | 리플레이 변환 | `d1fb2836293486f3c8f19b54f5c79967715e6881c83f83ff5614a19932aa19f8` |
| mayhem-11 | 80/80 | 1060 | 자체 작성 | `f4a7b3d678e0bf77468ee75f12cf7bc8fb7f858df0e99d663a51b285b98711c7` |
| mayhem-12 | 75/75 | 3784 | 리플레이 변환 | `cf203fb989bca0ce2a68193f19f7281fb6f2ce08869bc9f80012d6a1fe0d05c9` |
| mayhem-13 | 2/2 | 1208 | 자체 작성 | `165ca941aee4295e975b3760d05abbcb748056b466f7703ba3131c222139bfcc` |
| mayhem-14 | 79/75 | 4515 | 리플레이 변환 | `3d8f44433cf05074b812f14ab7fa969ab2e74385509ef6d675cbbf96d4e622a1` |
| mayhem-15 | 74/60 | 3061 | 자체 작성 | `ae59164a5faef49c441283e5d595e129268e0a5a843fed3b319b0d288f669e3f` |
| mayhem-16 | 50/50 | 970 | 리플레이 변환 | `ce3c5156780dd751f050e66e578b8917ba20565c696826c8dc6c8e1a2b67d313` |
| mayhem-17 | 77/70 | 2366 | 리플레이 변환 | `1a398f45c81d95deedcf0584abd0da92c0299ecf98296377023164c85188c820` |
| mayhem-18 | 73/72 | 2403 | 리플레이 변환 | `342b2e27bae443ec348aa82f141f24292d2cadd8221abb9d5d9b2868be1c1cbd` |
| mayhem-19 | 46/46 | 2878 | 리플레이 변환 | `7f88f3ed127982c91bae65207e75a36c2c8483a125aa1f36bac01af90be862fc` |
| mayhem-20 | 50/50 | 1245 | 자체 작성 | `bdad0401416c064c6351171dd4948cf21168e5242c3b02808b76eb4ec8fb7337` |
| mayhem-21 | 50/50 | 3865 | 리플레이 변환 | `3bfd6d46455397c43f804dcf9ff58bd574cd9cdc7d70af1c34a078bfb84af93e` |
| mayhem-22 | 79/68 | 4095 | 자체 작성 | `08c3c63d99ba2725506c4f3f0536d0307537f7c112f01369f0732b32a69ae0f0` |
| mayhem-23 | 72/64 | 3878 | 리플레이 변환 | `484d5d531559c7926b5f67b72be34b73d45c95dd311bb6db9b782b074f8e929d` |
| mayhem-24 | 50/50 | 406 | 자체 작성 | `feb4159e2b5730b3d72e9845e2c1ce7fce6fec0b542ac256fe193858dcbf0e7d` |
| mayhem-25 | 72/72 | 2038 | 자체 작성 | `89a5d9f53dc5035c7dadb72e06aef147772ba3ce3ef5a3ae5a37ce28df42fb6a` |
| mayhem-26 | 76/60 | 4926 | 리플레이 변환 | `187b4056c076a0f7d8dca81fce38f735d6a482395f28b45dd5ec180a1aa0c90a` |
| mayhem-27 | 50/50 | 970 | 리플레이 변환 | `ce3c5156780dd751f050e66e578b8917ba20565c696826c8dc6c8e1a2b67d313` |
| mayhem-28 | 1/1 | 3312 | 자체 작성 | `d8c3385bdcc415533206119240db157327d10efb46b4f4c156d71b54de84db3f` |
| mayhem-29 | 75/64 | 4371 | 리플레이 변환 | `2c46e39238aa35cc98a5c34b9a6ea9fddff1391d6ea2faa61cc21f1cd018d41d` |
| mayhem-30 | 68/60 | 7332 | 리플레이 변환 | `4136a8d556513ffab7d3b671b3c5c4b8ab25042e925db3ab062a65bddae7d33e` |

공개 변환 입력의 작성자는 Clam(01·06·09·10·18·19·26·29·30); Pooty(17·23); ccexplore(04·05·07·08·12·14·16·21·27); namida(02·03)다. 정확한 게시물·날짜·다운로드 URL·원본 크기/SHA-256·선택 리플레이와 ZIP 멤버 해시를 manifest에 기록한다.16번은 같은 지형의27번 공개 해법을 재사용한 것으로 별도 명시했다. UI 선택·pause는 제외하고 기술/RR/Nuke 기능만 안정된 actor ID로 변환했다. 일부 입력에 실제 required 이상 구출한 뒤 Nuke를 추가한 변경도 개별 출처에 적었다. 원본 좌표 기록 전체를 재현했다고 인증하지 않는다.

자체 입력의 구상에는 [Jim & Anita Vasey의 공략](https://www.justadventure.com/walkthrough/lemmings/2/)을 참고했다.15번은 프로젝트 Tricky10 입력에서 조기 Nuke를 제외하고 실제 Mayhem 목표를 달성할 때까지 실행했다.25번은 [LemSteven의 경로 설명](https://www.lemmingsforums.net/index.php?msg=82236)(2020-05-10)을 참고하되 착지 다리와 후속 레밍의 Builder 대기를 추가했다.28번은 [Pooty의 천장 경로 구상](https://www.lemmingsforums.net/index.php?msg=31676)(2009-10-28)을 참고하여 출발 위치·다섯 번째 Builder 앞 두 픽셀 이동·후속 굴착 입력을 직접 작성했다. 정확한 입력 시각·actor ID와 교통 제어는 프로젝트 작성이며 원문·리플레이·이미지/맵/음원을 공개하지 않는다. 열람일은2026-10-21이다.

실패 입력은 모두 `120 0 Nuke`이며 실제 Lost와 원본 목표 미달을 요구한다. 성공 Nuke도 원본 required 이상 실제 구출한 경우만 인정한다. 목표·인원·재고·결과를 강제 설정하지 않으며 거부된 기술 명령을 성공으로 합산하지 않는다.

## 검증과 재현

[저장 해법 계약](../tech_insights/issue-94_solution_replay.md)의 엄격한 입력·매 경계 전체 v1 상태/actor·terrain/brick/raw object map·전체 RGB와 SDL/Canvas readback을 적용한다. 성공/실패마다 실제 SDL과 WASM에서 렌더1/7/13·시간30/60/144Hz로3회 실행한다. SDL 결과 뒤 실제 Restart는 렌더37/101/257에서 초기 자원·재고·맵·이전 입력 폐기·무입력 prefix를 검사한다. WASM은 Restart의 음원 세대+1·빈 큐·sequence0·reset과 같은 입력 재완주, 숨김/1시간 경과/복귀를 검사한다. 결과 뒤4 guarded probe도 고정 상태를 요구한다.

고정 Lemmix `40e9bc34451f0e9127fd53b290a3877e700d143d`의56함수/표와 기존 종료 adapter의 지원 상태·지형 변경·실제 결과를 대조한다. 새 DOS 영상·모든 내부 상태·실시간 성능·실제 DOM 포인터/키 입력 인증으로 범위를 확대하지 않는다.

재현은 기존 계약의 추출 예제에서 `assets/manifests/solution_replay_mayhem.json`을 사용한다. `source assets/local/dev-tools/env.sh` 후 `python3 tools/solution_replay.py --assets assets/local/dos-original --cases <새 ignored-cases.json> --evidence <새 ignored-증거폴더> --workers 2`를 실행한다. 기존 증거를 덮어쓰지 않는다. 실행 원시와 원본/파생 자료는 ignored `assets/local/issue-102-review/`에 보존한다.

이번 변경은 입력과 문서이며 제품 Rust·렌더·UI·음향·의존성과 검증 도구는 변경하지 않는다. 최종 main 검증 후 기존 Fun·Tricky·Taxing90개와 합쳐120/120 성공/실패/Restart 묶음으로 집계한다. COMP-01~18 전체 재감사·성능/G11/G12 마감은 #93/#1 후속이며 미완료 항목을 통과로 합산하지 않는다. 브라우저에 표시하는 저장 입력 영상과 정식 증거 실행도 구분한다.
