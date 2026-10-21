# G11-05c Tricky01~30 해법·반복·재시작 검증

이슈: [#98](https://github.com/edwardkim/ohmylemmings/issues/98) · 상위 [#93](https://github.com/edwardkim/ohmylemmings/issues/93) · 선행 [#96](issue-96_fun_solutions.md) · [입력 목록](../../assets/manifests/solution_replay_tricky.json).

## 입력과 출처

기준 main `1ac6ddc57cb32688d9a1c44aba83e5fe06391194`에서 Tricky30개의 성공/실패60입력을 확보했다. 성공20개는 직접 작성했고10개는 아래 공개 DOS Lemmix 리플레이의 기술·RR·Nuke 기록을 안정된 actor ID 입력으로 변환했다. 원본 인원·재고·구출 목표를 유지한다. 아래 수치는 해법 확보 결과다. 최종 SDL/WASM·기준 대조·정확한 CI와 병합 판정은 [#98](https://github.com/edwardkim/ohmylemmings/issues/98)에 사례별 증거 해시와 함께 기록한다.

| 레벨 | 구출/원본 목표 | 실제 종료 control_step | 성공 입력 출처 | 입력 SHA-256 |
| --- | ---: | ---: | --- | --- |
| tricky-01 | 54/40 | 2276 | 자체 작성 | `74efedf742e3f216db75ca40644349487cd2fb07efdeadc0634b12d666f9fd79` |
| tricky-02 | 40/40 | 2811 | 자체 작성 | `bb12d8418695269f6add3f39287adc77d1ad408d7a32c34a186f4cd121bbdaf9` |
| tricky-03 | 75/40 | 2824 | 자체 작성 | `f22a871152d00d008edc9b9ac5e7da9803e6b9fd3875f7c20b33f7485aadcb31` |
| tricky-04 | 17/16 | 3171 | 자체 작성 | `f8318a96f4b1c389bc4d182ecd6cdcf8a9957b514e305077d58d3fbbcd154c50` |
| tricky-05 | 74/16 | 2484 | 자체 작성 | `294ddbebf65e9d160e176b780e4789cd7db07838ff0376f7e3515a7a307a0bc7` |
| tricky-06 | 5/4 | 1160 | 자체 작성 | `ea1b6ddc404d25e4045171e41cebf099315d98c9c689135d5971bd714cc0cab0` |
| tricky-07 | 63/55 | 2525 | 리플레이 변환 | `275c82cd6b6e4482152c2f96c67576734bab44553fe1f34dbc09e0c3ca04a130` |
| tricky-08 | 58/48 | 3796 | 리플레이 변환 | `6271a7fe55747683756df330d35bb47451e2306d6b46514e94888c8f013fb1ce` |
| tricky-09 | 72/70 | 3379 | 리플레이 변환 | `05c90180473176a70bc7bb754797f21219a96764030c1effe690f3e163881eb1` |
| tricky-10 | 74/74 | 2291 | 리플레이 변환 | `7401478940ddbdb46a3a9c30fc9edfa96ca807d70994bc55710859fdcd16e1bd` |
| tricky-11 | 47/42 | 2759 | 자체 작성 | `178b3802291f52f2dd228febf048d312f04e6e6a7ae0706120319b3f24906025` |
| tricky-12 | 49/40 | 1927 | 자체 작성 | `a814eb26935330820517e52d139cccd303a9a5c561d79ba02c1ce7e21948a71d` |
| tricky-13 | 62/56 | 2825 | 자체 작성 | `cc420a3a0282db214f4774b012306b779764a0f22fb000108ad1527ce81d6c40` |
| tricky-14 | 71/70 | 3819 | 리플레이 변환 | `4c819ac20ac2efea87fae1dc96b750ad174add09a92a4eeababc1e0768cc032b` |
| tricky-15 | 6/6 | 480 | 자체 작성 | `e4f4679765fd6b972498434d3716127c733f6bb96c1a3b3efa0bc1d21fbe33a8` |
| tricky-16 | 40/40 | 2232 | 리플레이 변환 | `8ce925ee8dd9f3404e00b97e3461433ff1b886dd3a3768f550bec8f182e0a8d1` |
| tricky-17 | 48/48 | 1690 | 자체 작성 | `13442ce7ff26436a0a4f5cc59b7e2206284793d7a68d24597e8d1ba091ae313c` |
| tricky-18 | 9/9 | 591 | 자체 작성 | `f1df7b27c42a2c125dcb238fb4ce8322a55f0ba765c273390b8440ecb19a991d` |
| tricky-19 | 50/50 | 2979 | 자체 작성 | `c819b09069dffc267da239a407a0173007bd82e863962f5e1f2b90ba01538628` |
| tricky-20 | 77/76 | 1920 | 자체 작성 | `c0bcc1e55e377b2289c2ccfca0107c28dad85c7a1f5652a6c210a9d8fba21516` |
| tricky-21 | 65/44 | 1884 | 자체 작성 | `2bf896e3efc143e48e8d61871d900176c9c88a6ee90db348bb5587e8587feda4` |
| tricky-22 | 78/72 | 1223 | 자체 작성 | `0f2df25c43742de83b29bc3ea7bc0cdad9c19d022de6a988af050ce80bb33f6b` |
| tricky-23 | 50/48 | 881 | 리플레이 변환 | `e4c88c915045e1c3ce75bcc6a9f883d19fc51b7e93979b10dab1b7e732132f1e` |
| tricky-24 | 77/75 | 2029 | 자체 작성 | `c955ce26d3fd94c43fdb05f216df4a73cf0d3d09fd5dc75543b1977bbd551d92` |
| tricky-25 | 10/10 | 1217 | 자체 작성 | `e97186c6da77553341a24ebc88379fc49d21494f82ef300aef4149acbedf2a9b` |
| tricky-26 | 80/80 | 708 | 자체 작성 | `4eee8596ca99880b7177f74ddb8a87063e0973c1d0d9f677a4236aac7f3d25a1` |
| tricky-27 | 73/72 | 4354 | 리플레이 변환 | `e42ff9c22f20c69a32ab429c21e83c7ec39773808b1e113b4917c6a938854a0f` |
| tricky-28 | 79/72 | 1710 | 자체 작성 | `cacaadb91de006ca44e497505c0aa136da856520ca28917d90cb33e2adc4f52c` |
| tricky-29 | 80/79 | 2430 | 리플레이 변환 | `d13128f737b6265bb66ec74248fa4b068ab26e8337493789b8ffe96310b5aef7` |
| tricky-30 | 76/72 | 3706 | 리플레이 변환 | `d0b9a0fe557ad78dfc9a87ef0d3b0c34d0713a325e2c3d32ff64fcbd2fe03ab2` |

공개 리플레이 출처는 ccexplore(07·09·10·14·23·27·30), Clam(08·16), namida(29)다. [한 레밍당 한 기술](https://www.lemmingsforums.net/index.php?topic=1050.0), [필수 기술 조사](https://www.lemmingsforums.net/index.php?topic=1084.0)의 실제 첨부를 사용했으며 정확한 URL·원본 ZIP/LRB/멤버 크기와 SHA-256을 입력 manifest에 남긴다. UI 선택·pause 기록을 제외하고 기능 기록만 변환했다.08은 원본80마리 중48마리 이상 구조한 뒤 Nuke를 추가해 타이머 대기를 줄였다. 원본 텍스트·리플레이 파일·이미지는 공개하지 않는다.

자체 해법의 경로 구상은 [StrategyWiki Tricky](https://strategywiki.org/wiki/Lemmings/Tricky_Levels_1-5)와 [LemSteven의 강철 가장자리 해법](https://www.lemmingsforums.net/index.php?topic=1068.75)(2019-12-14)을 참고했다. 공략 문장은 복제하지 않았다.13은 기초 계단의 진입 높이, 복귀 경로, Blocker 위 우회로까지 새 입력으로 확인했다.25는 중간 기둥의 Digger 대기 구역과 Miner 복귀 경로로10마리 구조를 확인했다. 열람일은2026-10-21이다.

실패 입력은 모두 `120 0 Nuke`이며 실제 Lost와 원본 목표 미달을 요구한다. 성공 입력의 Nuke도 원본 required 이상 구조한 경우에만 인정한다. 목표·인원·재고·결과를 강제 설정하지 않는다. 거부된 기술 명령을 기술 성공으로 합산하지 않으며 실제 최종 정수 필드로 판정한다.

## 검증과 재현

[기존 계약](../tech_insights/issue-94_solution_replay.md)의 엄격한 입력·매 경계 전체 v1 상태/actor 필드·terrain/brick/raw object map·전체 RGB와 SDL/Canvas readback을 적용한다. 각 성공/실패를 SDL과 실제 WASM에서 렌더1/7/13·시간30/60/144Hz로3회 실행한다. SDL 실제 Restart는 렌더37/101/257에서 초기 자원/재고/맵·기존 입력 폐기·무입력 prefix를 검사한다. WASM은 Restart 음원 세대+1/빈 큐/sequence0/reset과 같은 입력의 재완주를 검사한다. 결과 뒤4 guarded probe도 고정 상태를 요구한다.

고정 Lemmix `40e9bc34451f0e9127fd53b290a3877e700d143d`의56함수/표와 기존 DOS 종료 경계 adapter의 지원 상태·지형 변경·최종 결과를 대조한다. 새 DOS 영상·전체 내부 상태·실시간 성능이나 DOM 포인터/키 입력 인증으로 범위를 확대하지 않는다.

재현은 기존 계약의 manifest 추출 예제에서 `assets/manifests/solution_replay_tricky.json`을 사용한다. `source assets/local/dev-tools/env.sh` 후 `python3 tools/solution_replay.py --assets assets/local/dos-original --cases <새 ignored-cases.json> --evidence <새 ignored-증거폴더> --workers 3`을 실행한다. 기존 증거를 덮어쓰지 않는다. 원본/파생 화면·맵·음원과 실행 원시는 ignored `assets/local/issue-98-review/`에 보존한다.

이번 변경은 입력과 문서이며 제품 Rust·렌더·UI·음향·의존성과 검증 도구 코드는 변경하지 않는다. 최종 검증을 마친 뒤 Fun30개와 합쳐60/120을 집계한다. 다른60개(Taxing·Mayhem), COMP-01~18 전체 재감사와 성능·G11/G12 마감은 #93/#1에 남긴다. 미완료 검증을 통과로 집계하지 않는다.
