# G10-04 게임 사운드 이벤트·DOS 숫자 ID

이슈: [#78](https://github.com/edwardkim/ohmylemmings/issues/78) · 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1)

논리 이벤트는 `lemmings-core::SoundCue`, 제어 경계 기록은 `lemmings-session::SessionSound`, 자원 명령은 `lemmings-resources::AdlibCommand`, 소비 큐·레벨 세대는 `lemmings-game::SoundTimeline`이 담당합니다. core는 자원/OPL/SDL/웹을 참조하지 않습니다. 기존 상태 문자열·입력 로그·지형/객체/렌더 계약은 유지합니다.

## 원본 숫자 ID

기준 EXE SHA-256은 `4f21e94d53b7ea814ade802b4a37d42481da3876e31fb5a23090e19c9600f21c`입니다. 아래 주소는 MZ 헤더 0x120을 제외한 **로드 이미지의 코드 오프셋**입니다. 원본 INT61 AH=4의 AL은 1부터 시작하며 자원 인덱스로 변환할 때 한 번만 1을 뺍니다. Lemmix WAV 열거형 번호를 사용하지 않습니다.

| 논리 사건 | 원본 AL | 코드 근거 |
| --- | --- | --- |
| 기술 버튼 선택 | 1 | 3f77 |
| 입구 열기 / Let's Go | 2 / 3 | 03a7 / 038e |
| 성공한 8개 기술 할당 전체 | 4 | 공통 3206 |
| OhNo | 5 | 2e59 |
| 치명적 착지 Splat | 8 | 229b |
| 작업 중 강철 충돌 | 10 | 24cb / 2a65 / 2d08 |
| 폭발 상태 진입 | 12 | 2d65 / 2ead |
| 불 / 출구 / 물 진입 | 13 / 16 / 17 | 30a0 / 2f9c / 3049 |
| Builder 마지막 세 벽돌 경고 | 18 | frame10, 26d0 |
| Basher/Miner 전방 강철로 할당 거절 | 18 | 3295 |
| 함정 진입 | GROUND byte27 원시값 | 2fe9..2ff5 |

할당 거절음은 재고가 있고 상태가 유효할 때의 전방 강철 분기만 해당합니다. Miner 발밑 강철 판정이 우선하므로 이 경우는 무음입니다. 빈 재고·부적격 상태·일방향 벽 거절·Digger 발밑 강철 거절도 무음입니다. 작업 중 일방향 벽에 의한 정지는 무음이고 실제 강철만 ID10입니다. 원본 Nuke 플래그 0/255 모두 OhNo를 발생시키므로 Lemmix의 Nuke 억제를 따르지 않습니다. 폭발음은 지형 삭제가 시작되는 다음 틱이 아니라 Exploding 상태 진입에서 한 번 발생합니다. Splat 직후 같은 경계에서 출구가 감지되면 Splat→Yippee 두 사건을 보존합니다.

GROUND는 입구(id1)를 제외한 첫16개 객체의 기존 compact slot과 같은 배열로 보관합니다. trigger_effect4 함정만 byte27을 보존하며 0은 무음, 1..18은 원시 효과음, 그 밖은 `TrapSound` 진단입니다. 잘못된 슬롯도 `TrapSlot`로 남기고 다음 사건 처리를 계속합니다. 이름 추정으로 ID를 바꾸지 않습니다.

## 음악과 시작 위상

원본 482f..4876의 일반 음악은 `[6,7,8,10,11,4,12,13,14,15,5,16,17,18,19,20,21]`, 특수 selector1..4는 `[1,9,3,2]`입니다. 일반 성공 Continue만 cursor를 증가시킵니다. 특수곡은 일반 cursor를 소비하지 않습니다. 실패·Restart·수동 선택은 cursor를 보존하되 **첫 일반곡6을 선택**합니다. 원본 비진행 경로에 AL=0이 남는 동작까지 재현합니다. 다음 성공은 보존된 cursor에서 계속 증가합니다. 기존 `music_for_level`은 과거 Lemmix 전역 레벨 목록 API로 유지하며 실제 재생 선택에 사용하지 않습니다. 특수 배경의 제품 로딩 지원 범위는 기존과 같고 selector 표는 별도 검증합니다.

CS019d가 시작 dispatcher를 호출한 다음 CS01a3에서 DS0052를 증가시킵니다. 기존 장면 추적의 post-counter와 구별하여 새 사운드는 논리 iteration **15/35/55**에서 Let's Go/입구/음악을 발행합니다. 기존 `EntranceSound34` 문자열은 회귀 호환 진단으로 유지합니다. 첫 spawn54와 음악55의 차이도 유지합니다. 원본 음악 시작은 AH3 뒤 즉시 AH0도 호출합니다. 게임 명령 비교는 AH3/AH4 요청을 비교하고, AH0 드라이버 실행은 #76의 별도 오디오 시계가 담당합니다. 비동기 IRQ와 제어 경계의 실제 위상 일치까지 인증하지 않습니다.

## 순서·소비·초기화 계약

`SessionSound`는 control_step/iteration/경계 내 sequence/input_sequence/actor_id를 보존합니다. UI 선택은 입력 로그 밖에 있으며 일시정지 중에도 다음 제어 경계에서 발행됩니다. repeat/focus release는 추가 선택음을 만들지 않습니다. 성공한 경계의 사건 프레임을 교체하며 종료 후 tick은 비웁니다. 기존 actor/batch 오류 의미는 유지하고 전체 Session tick의 트랜잭션 원자성은 주장하지 않습니다.

호스트는 매 경계 `LevelRun::tick`을 사용해 사건을 수집합니다. 큐는 세대마다 증가하는 sequence와 최대1024개를 가지며 오래된 사건을 버릴 때 dropped를 기록합니다. 틱 전 UI 요청도 최대1024개이며 초과분은 dropped로 전파합니다. u64 sequence 소진은 진단/손실로 보고하고 번호를 재사용하지 않습니다. 읽기/렌더는 사건을 만들지 않습니다. WebGame의 `drain_sound_events()`는 단일 소비자용 JSON 배치를 비우고 `sound_state()`는 진단만 읽습니다.

완전히 생성·검증된 replacement에만 이전 세대+1을 부여합니다. 로드/selector/세대 overflow 실패는 기존 장면·큐·음악을 보존합니다. 세대 전환 시 reset=true와 빈 큐를 발행하며 새 VM/chip/resampler/phase/device queue를 만드는 책임은 G10-05 소비자에게 있습니다. 결과 경계는 Stop 한 번, 동결된 후속 틱은 무음입니다. `AdlibPcm::apply`는 typed 명령을 기존 검증된 요청 함수로 전달합니다. #76 엔진/라이선스/샘플 계약은 변경하지 않았습니다.

## 재현

```sh
source assets/local/dev-tools/env.sh
python3 tools/check_game_sound.py --wasm
python3 tools/check_web.py
make check
```

기본 검사는 원본/Unicorn/장치 없이 고정 메타데이터 oracle과 합성 게임98경로·170음악조합을 검증합니다. 매 틱/분할 배달 trace, native/실제 WASM, 6개 명령→PCM 경로의 229665프레임·두 청크 분할·독립 direct 스케줄 결과를 비교합니다. PCM 위치는 `floor(control_delta * 44100 * 156250 / 2190197)`로 정한 합성 제어 경계 위상입니다. 음악/효과음 데이터는 합성 자원입니다.

선택적 원본 재검증은 새로운 ignored 경로를 지정합니다.

```sh
assets/local/issue-74-tools/venv/bin/python tools/dos_sound_reference.py --original-assets assets/local/dos-original --evidence assets/local/issue-78-new-proof/dos.json
python3 tools/check_web.py --original-assets assets/local/dos-original --evidence assets/local/issue-78-new-web
python3 tools/observe_audio_dos.py --help
```

원본 x86 증명은 Unicorn2.1.4에서 EXE opcode를 수정하지 않고 합성 guest 배우/메모리 상태의 분기를 제한 실행해 INT 요청을 기록합니다. 핸들러·전체 DOS 장면·장치 출력을 증명하지 않습니다. 별도 DOSBox 실제 실행은 첫 음악 RAM ID6과 IRQ 시계만 관측합니다. 생성 RUSSELL의 byte0만 0으로 seed하며 trailing mutable 메모리와 원본 key disk의 미검증 한계는 #76과 같습니다. 원본50파일의 전후 해시는 동일합니다. 원본 코드/음원/파생 PCM·영상은 ignored `assets/local/`에 남깁니다. 공개 fixture는 숫자·해시·합성 데이터만 포함합니다.
