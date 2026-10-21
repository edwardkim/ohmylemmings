# G11-05d Taxing01~30 해법·반복·재시작 검증

이슈: [#100](https://github.com/edwardkim/ohmylemmings/issues/100) · 상위 [#93](https://github.com/edwardkim/ohmylemmings/issues/93) · 선행 [#98](issue-98_tricky_solutions.md) · [입력 목록](../../assets/manifests/solution_replay_taxing.json).

## 입력과 출처

기준 main `1d4f977f5c8a028f67164249cbcc1473cf6d807a`에서 성공30개/실패30개 입력을 확보했다. 성공7개(01·18·19·24·27·28·30)는 프로젝트 작성 입력이며,23개는 작성자를 밝힌 공개 DOS Lemmix 리플레이의 기능 기록 변환이다. 원본 인원·재고·구출 목표를 유지한다. 아래 수치는 해법 확보 결과이며 최종 SDL/WASM·기준 대조·정확한 CI/병합 판정은 [#100](https://github.com/edwardkim/ohmylemmings/issues/100)에 사례별 증거 해시와 함께 남긴다.

| 레벨 | 구출/원본 목표 | 실제 종료 control_step | 성공 입력 출처 | 입력 SHA-256 |
| --- | ---: | ---: | --- | --- |
| taxing-01 | 79/79 | 3989 | 자체 작성 | `bb55ec7763f538f89e0ac7564db8a9ee2496e5904756481749ac4fb61ae89185` |
| taxing-02 | 67/64 | 2520 | 리플레이 변환 | `d2387cbf5a47f0c0a6a04356575466e2e73a05eb4304aef179674fdb38a8f7fd` |
| taxing-03 | 80/80 | 2026 | 리플레이 변환 | `63550d5016ce19f8f410385c5fdd64bec302eb468a58f94fbdc6530641ee5945` |
| taxing-04 | 34/30 | 2366 | 리플레이 변환 | `9ca8cfe08dc3838697aa9c6a29463d47e8e4ee7d7f63f3fba5ea218d8ff64346` |
| taxing-05 | 53/45 | 3925 | 리플레이 변환 | `9da992e6d806031ce3e684006cd1f12302c321913739cae0bc949c99d9edd953` |
| taxing-06 | 39/30 | 1631 | 리플레이 변환 | `925b11635e72d2f693d906bd235a188eef4905c2b4edf707f42c895d0f081a0c` |
| taxing-07 | 78/78 | 2979 | 리플레이 변환 | `59b32d2caf74a04a509d3c83963ac346229719d8b204700a6f1ef422ff54320c` |
| taxing-08 | 80/80 | 2214 | 리플레이 변환 | `1eed85b8ccf465fff9cb67a203689cc99b06f837e92fe5bcdf7af58de251dd53` |
| taxing-09 | 20/20 | 809 | 리플레이 변환 | `2baf5038386d1aa44e73ca235a55b0edcf92a3d1e579097e1728748d03379185` |
| taxing-10 | 5/5 | 1256 | 리플레이 변환 | `4d2edda35e97964533184d7df41fc19c0c0f45d5433033d6f58feeecceef85cc` |
| taxing-11 | 50/50 | 2602 | 리플레이 변환 | `7be4b22c64812908536d971afdefad67965118880e2779c549f67bf147eb9543` |
| taxing-12 | 8/8 | 3045 | 리플레이 변환 | `683336ad215e16c898b82593e77a9e864b587f5eb2c6c4f48410f95fda148624` |
| taxing-13 | 80/79 | 3342 | 리플레이 변환 | `478b0045d64359140d0ed8d3d1924199cb23f5c77e1cdba694b950028fea8583` |
| taxing-14 | 77/76 | 6252 | 리플레이 변환 | `959fb03d212d163192d5db789da7dd7107e895655a08304290e8d5930ca89ccf` |
| taxing-15 | 80/64 | 5192 | 리플레이 변환 | `d83fbf6b4149dc9a16714d570ad53ab8f21dffc7c3df02f2444206e81b7e4daf` |
| taxing-16 | 77/77 | 2742 | 리플레이 변환 | `7e5806a809711f64327d225fb9a00b2f44f91657e6199b38a28662944b5921bf` |
| taxing-17 | 80/72 | 5939 | 리플레이 변환 | `973ce41eea3d829b67b88b0d8b704f65e9f9fe62d541a2977efd785926dfe4fe` |
| taxing-18 | 73/65 | 1663 | 자체 작성 | `6c9d141e8b44e8f78a058df39c0430d1f4793901e34be0f56f4c00a6571d83b0` |
| taxing-19 | 64/64 | 1471 | 자체 작성 | `b8174eb032f058261bff6c2f5cda6d58c05b35c34c23d5c0bc22ef51bafeb883` |
| taxing-20 | 70/70 | 4394 | 리플레이 변환 | `35445f0f18d4c3a3a7018bb3eb2daf5cf0f6a14913e767636e4dc4a24cf1718f` |
| taxing-21 | 20/20 | 1393 | 리플레이 변환 | `0902fe65d4837a56cadd793f20f19156eb2d0d54e9091d60fcc168b3d04d2628` |
| taxing-22 | 50/40 | 805 | 리플레이 변환 | `4664d7403b979d571af6dd6a834510640c330b6c47279fdd72d591b4c4677e70` |
| taxing-23 | 76/76 | 4863 | 리플레이 변환 | `cf39916a5bbcd5ef10ee9c91f4c25416245e88f1f532efd6f867b065c3a0d628` |
| taxing-24 | 80/79 | 3021 | 자체 작성 | `47c87f9c7bde148766db41b990f453a6906a9a3278e6c621191e34a3ae0d5251` |
| taxing-25 | 75/72 | 3840 | 리플레이 변환 | `cd57d7f641dcf919657c417283cf3cda1e4063849f0d3ae96d87cce93f4cd21d` |
| taxing-26 | 80/79 | 2108 | 리플레이 변환 | `f134352026244efba03665d2876a0d1057eb8e5bc8ecab6e987482bf1c636ab0` |
| taxing-27 | 48/48 | 2825 | 자체 작성 | `094c2971ebe58d6f7f82729fbef37b8bd72ff3aefd8d1190d3b2ca85edbf6ae2` |
| taxing-28 | 56/56 | 3113 | 자체 작성 | `a76072e1b4505e922c3c7b8e1af46a8ac6265841ee52a591c6145a6b4b56dc08` |
| taxing-29 | 80/76 | 2782 | 리플레이 변환 | `10e24b88ea09db772cd18a36cd2ae2f9f03710890413efabdd802d31eaeadfd5` |
| taxing-30 | 60/60 | 980 | 자체 작성 | `0a2b5886052a0f3bcece4433961e84704069833c501412713ecb3d098208a1b3` |

공개 변환 입력의 작성자는 Clam(04·08·11·13·15·25); Pooty(10·12); Proxima(03·26); ccexplore(06·14·16·17·20·21·22·29); namida(02·05·07·09·23)다. [필수 기술 조사](https://www.lemmingsforums.net/index.php?topic=1084.0), [한 레밍당 한 기술](https://www.lemmingsforums.net/index.php?topic=1050.0), [최소 기술/최대 구출](https://www.lemmingsforums.net/index.php?topic=1068.0), [지형 보존](https://www.lemmingsforums.net/index.php?topic=1490.0)의 실제 첨부를 사용했다. 정확한 게시물·날짜·다운로드 URL·원본 크기/SHA-256·선택 리플레이와 ZIP 멤버 해시를 manifest에 기록한다. UI 선택·pause 기록은 제외하고 기술/RR/Nuke 기능만 안정된 actor ID로 변환했다. 원본 좌표 기록의 완전한 재현을 인증하지 않으며 실제 원본 목표와 고정 기준의 지원 상태로 검증한다. 일부 입력은 원본 required 이상 구조한 뒤 Nuke를 추가해 타이머 대기를 줄였고, 해당 변경을 개별 출처에 적었다.

자체 작성 경로는 manifest의 공략/게시물을 구상에 참고했다.28의 착지 경사로는 [ccexplore의 DOS 스크린샷](https://www.lemmingsforums.net/index.php?msg=38691)(2012-07-28)을 참고했으며, 실제 Miner/Builder 취소·무리 대기·통로/함정 연결과 입력 시각은 새로 작성했다. 원본 공략 문장·리플레이·이미지와 파생 화면·맵·음원은 공개하지 않는다. 열람일은2026-10-21이다.

실패 입력은 모두 `120 0 Nuke`이며 실제 Lost와 원본 목표 미달을 요구한다. 성공의 Nuke도 원본 required 이상 실제 구출한 경우에만 인정한다. 목표·인원·재고·결과를 강제 설정하지 않으며 거부된 기술 명령을 성공으로 합산하지 않는다.

## 검증과 재현

[저장 해법 계약](../tech_insights/issue-94_solution_replay.md)의 엄격한 입력·매 경계 전체 v1 상태/actor·terrain/brick/raw object map·전체 RGB와 SDL/Canvas readback을 적용한다. 각 성공/실패를 SDL과 실제 WASM에서 렌더1/7/13·시간30/60/144Hz로3회 실행한다. SDL 결과 뒤 실제 Restart는 렌더37/101/257에서 초기 자원·재고·맵·이전 입력 폐기·무입력 prefix를 검사한다. WASM은 Restart의 음원 세대+1·빈 큐·sequence0·reset과 같은 입력의 재완주를 검사한다. 결과 뒤4 guarded probe도 고정 상태를 요구한다.

고정 Lemmix `40e9bc34451f0e9127fd53b290a3877e700d143d`의56함수/표와 기존 종료 adapter의 지원 상태·지형 변경·실제 결과를 대조한다. 새 DOS 영상·모든 내부 상태·실시간 성능·실제 DOM 포인터/키 입력 인증으로 범위를 확대하지 않는다.

재현은 기존 계약의 추출 예제에서 `assets/manifests/solution_replay_taxing.json`을 사용한다. `source assets/local/dev-tools/env.sh` 후 `python3 tools/solution_replay.py --assets assets/local/dos-original --cases <새 ignored-cases.json> --evidence <새 ignored-증거폴더> --workers 2`를 실행한다. 기존 증거를 덮어쓰지 않는다. 원본/파생 자료와 실행 원시는 ignored `assets/local/issue-100-review/`에 보존한다.

이번 변경은 입력과 문서이며 제품 Rust·렌더·UI·음향·의존성과 검증 도구는 변경하지 않는다. 최종 검증을 마친 뒤 Fun/Tricky/Taxing90/120으로 집계한다. Mayhem30개·COMP-01~18 전체 재감사·성능/G11/G12는 #93/#1 후속이며 미완료 검증은 통과에 합산하지 않는다.
