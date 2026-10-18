# G00-03 호환성 범위와 비교 기준

작업 이슈: [#5](https://github.com/edwardkim/ohmylemmings/issues/5), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 기준 입력과 판정 우선순위

기준 프로필은 `dos-original-vga`입니다. [원본 매니페스트](../../assets/manifests/dos_original.json)의 50개 파일과 [소스 매니페스트](../../assets/manifests/reference_sources.json)의 고정 커밋을 사용합니다. 세부 소매판 리비전 이름은 부가 정보로 두고 파일 크기·SHA-256으로 입력을 식별합니다. 현재 확인한 것은 바이트와 정적 로직이며 DOS 실행 동작 인증은 후속 검증의 대상입니다.

판정 근거는 같은 입력을 사용한 DOS 실행 관찰, 고정 Lemmix의 계측 상태, 고정 Lemmings.ts의 리소스 해석 순으로 적용합니다. DOS와 레퍼런스가 다르면 차이 이슈를 만들고 원본 증거를 확보한 뒤 이 기준을 수정합니다. 기대값을 Rust 출력에서 생성해 통과시키지 않습니다.

- Lemmix: `40e9bc34451f0e9127fd53b290a3877e700d143d`.
- Lemmings.ts: `10c7c8b9167258503d02589cf048e2d83a43dd0a`.
- [기계 판독용 기준](../../assets/manifests/dos_compatibility.json): 입력 매니페스트 해시, 레벨 120개, 해제 구간 81개의 크기·해시, 오디오 목록.
- [레벨별 전체 목록](issue-5_level_inventory.md): 제목, DAT 구간, ODD 인덱스, 수량·시간·직업 수.

## 레벨·리소스 지원 범위

Fun·Tricky·Taxing·Mayhem 각 30개를 모두 지원합니다. DAT 10개에 담긴 기본 레코드 80개와 ODDTABLE 변형 40개를 조합한 120개 진입점입니다. ODD 레코드는 `기본 레코드 번호 × 56` 위치에서 24바이트 초기 수치와 32바이트 제목을 읽습니다. 기본 번호는 `파일 번호 × 8 + 구간`입니다.

[Lemmix GetEntry](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Styles.Dos.pas#L183)의 연결과 [TS 설정](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/public/data/config.json)의 파일·구간·변형 여부는 120개 모두 일치합니다. TS의 [LevelIndexResolve](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/level-index-resolve.ts#L27)는 `levelNumber`를 표시 순번으로 계산하고 [LevelLoader](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/level-loader.ts#L26)가 이를 ODD 조회에 넘기므로 이 경로를 그대로 이식하면 안 됩니다. 예: Fun 8의 ODD 인덱스는 6이며 제목은 `Not as complicated as it looks`입니다.

리소스 범위는 LEVEL 10개, GROUND 5개, VGAGR 5개, VGASPEC 4개와 MAIN·ODDTABLE·ADLIB입니다. VGASPEC 4개를 쓰는 진입점도 전체 목록에 포함했습니다. 이 기준은 DOS 원작 VGA·AdLib 프로필에 적용하며 Oh No!, Holiday, CGA/Tandy 그래픽과 사용자 제작 레벨은 후속 범위로 관리합니다.

## 직업과 상태

직업 수량의 고정 순서는 `Climber, Floater, Bomber, Blocker, Builder, Basher, Miner, Digger`입니다. 각 직업은 정상 할당·불가능 상태·수량 0·지속 동작·지형/다른 개체 상호작용을 검증합니다. 성공한 할당만 수량을 소비하고 영구 능력과 현재 행동을 구분합니다. [직업별 소스 진입점](issue-3_reference_code_map.md#g07-직업별-진입점)을 사용합니다.

| 직업 | 필수 비교 사례 |
|---|---|
| Climber | 벽 타기·머리 충돌·정상 등반·호이스팅·영구 능력 중복 할당 |
| Floater | 낙하 중 할당·우산 개방 프레임·착지·Climber와 능력 공존 |
| Bomber | 카운트다운·OhNo/즉시 폭발 분기·폭발 마스크·강철·다른 직업 중 할당 |
| Blocker | 좌우 반전 영역·영역 겹침·발판 소실·폭파·기존 지형 보존 |
| Builder | 12개 계단·마지막 3개 경고·벽/천장 충돌·연속 할당·Shrugging |
| Basher | 수평 마스크·무지형 종료·강철·일방향 지형·발판 소실 |
| Miner | 대각 마스크·강철·일방향 지형·경계·원작 방향 버그 |
| Digger | 수직 마스크·강철·빈 지형·낙하 전환·다른 개체와 동시 처리 |

할당 직업 외 행동은 Walking, Jumping, Falling, Climbing, Hoisting, Floating, Digging, Building, Bashing, Mining, Blocking, Shrugging, Ohnoing, Exploding, Exiting, Drowning, Splatting, Vaporizing입니다. 제거 여부와 폭발 카운터를 별도로 기록합니다. [행동·직업·메커니즘 정의](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Dos.Consts.pas#L90)를 기준으로 합니다.

## 게임 규칙과 채택한 계약

아래 수치·순서는 고정 Lemmix를 채택한 비교 계약입니다. 실제 DOS 동작과의 일치는 G05~G11에서 확인합니다.

| 영역 | 계약과 관찰 대상 |
|---|---|
| 시간 | 초당 17 논리 틱, 정수 카운터 사용. 60ms 반올림으로 누적하지 않음. 세션 시계와 개체·입구의 진행 상태를 각각 기록 |
| 생성 | 시작 카운트다운 20, 입구 개방 후 진행. 간격은 `floor((99-RR)/2)+4`, 초기 RR부터 99까지 UI 조정. RR 변경과 기존 카운트다운의 관계도 비교 |
| 입구 | 최대 4개. 1개 AAAA, 2개 ABBA, 3개 ABCB, 4개 ABCD. 원본 객체 순서를 유지 |
| 좌표 | 비교용 지형 영상은 1584×160. Lemmix 개체 판정의 x=0..1647, 하단 y=163 경계도 별도로 비교. 보이는 영역으로 개체 충돌 좌표를 잘라내지 않음 |
| 낙하 | Lemmix의 낙하 누적값 60 경계와 `FallerStartsWith3` 초기화, Floater의 개방·착지 프레임을 함께 검사 |
| 지형 | 지형/강철/일방향/Blocker/객체 트리거 마스크를 구분. 삭제·추가와 개체 순서에 따른 같은 틱 상호작용을 비교 |
| 객체 | 출구·입구·물·불·작동형 함정·일방향·장식. 객체 슬롯 16 이후의 트리거 비활성, 함정의 작동·재사용 프레임 포함 |
| 세션 | 생성 총수·생존·제거·구출·직업 재고·시간·성공/실패·재시작. 성공률 계산은 정수 백분율과 Nuke 분모 변화를 비교 |
| 선택/UI | 직업 선택, 겹친 개체의 우선순위, 마우스 위치와 버튼·키 전이, RR, Pause, Nuke, 스크롤·미니맵을 기록 |

근거: [틱 호출 순서](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L3408), [시계·시작 이벤트](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L3498), [생성 간격](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L4001), [종료 판정](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L2232).

원작 버그 재현도 지원 범위에 포함합니다. `DOSORIG_MECHANICS`에 NukeGlitch·PauseGlitch·RightClickGlitch를 추가한 프로필로 비교합니다. Lemmix의 기본 DOSORIG 집합만 선택하면 이 세 옵션이 빠집니다. [옵션 적용 위치](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L1389)와 [Nuke/Pause 처리](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L3930)를 확인합니다. 나머지 원작 메커니즘은 DOSORIG 집합을 유지하며 `Obsolete`는 리플레이 호환 비트로 취급합니다.

## 오디오 목록과 판정

압축된 `ADLIB.DAT`의 구간 0을 해제하면 22,125바이트입니다. 포인터는 리틀 엔디언입니다. TS의 `readWordBE`도 구현은 리틀 엔디언이므로 함수 이름만 보고 바이트 순서를 정하면 안 됩니다.

- 음악: `music-00`~`music-20`, 총 21개. 오프셋 2926에서 21개 포인터를 읽으면 첫 헤더 2968 직전까지 정확히 채우며 각 헤더·악기·채널 주소의 범위를 검증했습니다.
- 효과음: `sfx-00`~`sfx-17`, 총 18개. 포인터 표는 오프셋 21989에 있으며 모든 주소가 해제된 이미지 안에 있습니다.
- 모든 ID·주소·음악 채널은 기계 판독용 목록에 열거했습니다. 곡명과 효과음 번호의 의미를 추정해서 붙이지 않습니다.
- 논리 효과음 이벤트 목록: BuilderWarning, AssignSkill, Yippee, Splat, LetsGo, EntranceOpening, Vaporizing, Drowning, Explosion, HitsSteel, Ohno, SkillButtonSelect, RopeTrap, TenTonTrap, BearTrap, ElectroTrap, SpinningTrap, SquishingTrap. 이 목록과 원본 숫자 ID의 대응은 G10에서 DOS 재생으로 검증합니다.

TS 설정의 `audio.numberOfTracks=20`은 21번째 헤더를 누락합니다. [Lemmix의 1..21 음악 순환](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Styles.Dos.pas#L214)과 데이터 구조를 채택해 표시 전체 레벨 순번(0부터)을 21로 나눈 나머지를 음악 ID로 정합니다. 이 순서와 실제 재생 타이밍은 G10의 DOS 검증 항목입니다. [TS 음악·효과음 초기화](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/sound/sound-image-player.ts#L51)는 형식 해석 참고용입니다.

G10은 21곡·18효과음의 명령 해석·반복·중단·동시 발생·음소거·볼륨·재시작을 검증합니다. 먼저 같은 합성기에 전달하는 `(audio_step, register, value)` 스트림을 정확히 비교하고, 고정 OPL 구현·샘플률·초기 상태에서 PCM 해시를 비교합니다. 원본 장치 녹음은 장치 차이를 고려한 청취·타이밍 증거로 기록합니다. TS의 경험적 오디오 속도 상수를 원작 타이밍 증거로 취급하지 않습니다.

## 첫 비교 레벨과 입력 시나리오

첫 비교 대상은 `fun-01 / Just dig!`입니다.

| 항목 | 기준 |
|---|---|
| 입력 | LEVEL009.DAT 구간 1, ODD 없음, GROUND0O.DAT·VGAGR0.DAT·MAIN.DAT·ADLIB.DAT |
| 시작 수치 | RR 50, 총 10, 필요 구출 1(10%), 5분, Digger 10, 다른 직업 0 |
| 초기 화면 | 원본 screen_x=624, 그래픽 세트 0, 특수 그래픽 없음 |
| 객체 | 슬롯 0 입구 (704,36), 슬롯 1 출구 (848,116), 슬롯 2 장식 (848,108). x는 저장 값에서 16을 뺀 좌표 |
| 해제 레코드 | 2,048바이트, SHA-256 `d4476fa052453d6afab5f6a2d57df866c56e3625e0352eead81d25b11dbf3906` |

1. **F00 무입력:** 재시작 직후부터 340틱까지 입력 없이 실행합니다. 초기 상태와 매 틱 상태를 저장합니다. 고정 Lemmix에서 도출한 확인 지점은 LetsGo 15, 입구음 34, 입구 작동 35, 첫 생성 54, 이후 간격 28입니다. 이는 아직 DOS 실행에서 측정한 값이 아닙니다.
2. **F01 첫 Digger:** 기준 실행에서 actor 0이 처음 Walking이 된 틱의 직후 경계에 Digger 할당을 기록합니다. 해당 틱을 한 번 측정해 정수 입력 목록으로 고정한 뒤 모든 구현에 같은 목록을 공급합니다. 수량 10→9, 할당 결과, 파기 전환·지형 변화·최소 1개체 구출·종료 결과를 비교합니다.
3. **F02 생성률:** 경계 54에 RR 99 명령을 넣고 340틱까지 관찰합니다. 별도 실행에서 RR 98을 사용해 새 간격 4가 같고 기존 카운트다운 처리도 같은지 검사합니다.
4. **F03 Pause/실패:** 경계 16에 Pause, 제어 단계 34에 Resume을 기록합니다. 같은 일시정지 동작에서 PauseGlitch 유무의 시계·입구 차이를 비교합니다. 별도 무할당 실행과 시간 경계 fixture로 실패·재시작을 검사합니다.

F01의 계측 틱과 실제 완주 입력·기준 상태 파일은 G05/G08에서 생성합니다. 지금 상태를 실행한 golden 결과로 표시하지 않습니다. 조건을 각 구현에서 따로 평가하면 차이를 감출 수 있으므로 기준 실행에서만 입력을 확정합니다.

## 입력·상태 기록 프로토콜

형식 식별자는 `dos-compat-trace-v1`입니다. `control_step`은 17Hz 제어 호출 순번, `iteration`은 실제 시뮬레이션 진행 틱입니다. 시작은 모두 0이며 렌더링 프레임 수와 분리합니다. 일시정지 중에도 Resume 명령을 전달할 수 있도록 제어 순번을 별도로 유지합니다.

초기 스냅샷은 입력 전 `(control_step=0, iteration=0)`입니다. 경계 s의 입력을 처리하고 한 번 갱신한 결과는 `control_step=s+1`에 저장합니다. 각 스냅샷은 다음 경계 입력을 적용하기 전 상태입니다. Restart는 이전 실행을 종료하고 새 run ID·초기 스냅샷·입력 목록을 시작합니다.

각 단계는 **경계 입력 → 종료 조건/생성률 처리 → Pause 판단 → 시계·틱 증가 → 생성 → actor ID 순 처리 → Nuke → 객체 갱신 → 상태 저장**으로 정의합니다. Pause 시 개체 갱신을 건너뛰지만 PauseGlitch 카운터와 입력 상태는 기록합니다. Lemmix 리플레이의 틱 끝 할당은 다음 제어 호출 직전의 경계로 변환하며, 틱 0 입력도 명시적으로 처리합니다. 한 틱의 종료 판정이 앞쪽에 있다는 사실을 상태 비교에서 보존합니다.

입력은 `(control_step, sequence, command, arguments)` 목록입니다. `sequence`로 같은 경계의 순서를 정합니다. 직접 개체 할당은 안정된 actor ID를, UI 검증은 마우스 좌표·버튼/키의 누름·놓음과 선택 직업을 기록합니다. 지원 명령은 SelectSkill, AssignSkill, SetReleaseRate, Pause, Resume, Nuke, PointerMove, ButtonDown, ButtonUp, KeyDown, KeyUp, Scroll, Restart입니다. 유효하지 않은 입력의 거부 결과도 출력합니다. 기계 판독용 목록에 F00·F02·F03의 정수 입력과 F01의 기준 실행 계측 조건을 기록했습니다.

상태에 필요한 필드:

- 헤더: 프로필·버전, 입력 파일 해시, 레벨 ID, 소스/실행 파일·빌드·에뮬레이터 버전, 메커니즘 옵션, 좌표계, 입력 목록 해시, 관찰 종류(`structured`, `video`, `manual`).
- 세션: control_step·iteration·시계 프레임·분/초, Pause/Glitch 카운터, RR·생성 카운트다운, 입구 상태·순서, 생성/생존/제거/구출 수, 초기 총수·현재 Nuke 분모, 직업 재고, 결과·실패 이유.
- actor: 생성 순번 ID, x/y·방향, 행동·애니메이션 프레임·낙하 누적, 영구 능력, 폭발/계단/개별 행동 카운터, 제거 사유. 저장 목록은 ID 순서이며 제거된 개체도 이유를 남깁니다.
- 지형/객체: 지형·강철·일방향·Blocker·트리거 마스크의 크기와 SHA-256, 변경 사각형, 객체 슬롯·종류·작동/애니메이션 프레임·함정 카운터, 다음 Nuke 대상.
- 출력: 직업 할당 성공/실패, 논리 효과음·음악 이벤트. 렌더링 영상과 실제 오디오 장치 출력은 코어 해시와 분리합니다.

정수·불리언·문자열·배열만 쓰고 부동소수점과 타임스탬프를 코어 상태에서 제외합니다. JSON은 키 정렬, ASCII escape, 공백 없는 구분자 `,`/`:`의 UTF-8 바이트를 해시하며 해시 필드 자체와 마지막 개행을 포함하지 않습니다. 마스크는 좌상단 원점·행 우선·행당 width바이트(각 픽셀 1바이트, stride 패딩 없음)로 정규화합니다. 서로 다른 내부 표현을 이 형식으로 변환합니다.

구조화 상태는 매 틱 수치·상태·마스크 해시가 모두 같아야 통과입니다. 첫 불일치 틱·필드·좌표를 보고하고 차이 주변의 전체 마스크를 저장합니다. 누락된 기준 실행·지원 필드·입력 해시는 `미검증`이며 통과로 계산하지 않습니다. DOS 영상 관찰만 확보한 사례는 원본 내부 상태가 같은 것으로 표시하지 않습니다.

## 시나리오와 구현 목표

| ID | 비교 사례 | 책임 목표와 완료 증거 |
|---|---|---|
| COMP-01 | DAT 전체 구간, 체크섬·절단·잘못된 복사 | G02: 80개 레벨+오디오 구간의 크기·해시 일치, 손상 합성 입력 오류 |
| COMP-02 | 120개 레벨·ODD 40개·초기 수치·특수 그래픽 | G03: 전체 목록 일치, ODD 경계·잘못된 인덱스 오류 |
| COMP-03 | 지형 합성·덮어쓰기·삭제·팔레트·마스크 경계 | G03/G04/G06: 전체 레벨 합성 이미지·마스크 대조 |
| COMP-04 | F00 시작 순서·시계·입구·최초 생성 | G05: 계측 기준 틱과 상태 추적 일치 |
| COMP-05 | RR 최저/98/99·기존 카운트다운·1~4개 입구 | G05/G08: 간격·생성 순서와 상한 일치 |
| COMP-06 | 걷기·방향 전환·등반 턱·낙하 한계·경계 이탈 | G05/G07: 합성 경계 fixture와 실제 레벨 상태 대조 |
| COMP-07 | 직업 8종의 정상·실패·0 재고·영구 능력 | G07: 위 직업 표의 모든 사례, 전환·재고·지형 일치 |
| COMP-08 | 같은 틱 다중 할당·Blocker·파기·폭발 상호작용 | G07: actor 순서·마스크 갱신·충돌 결과 일치 |
| COMP-09 | 강철·일방향·Miner 버그·Splat 출구 | G07/G08: DOSORIG 메커니즘 경계별 증거 |
| COMP-10 | 객체 슬롯 15/16(0 기준)·함정 재사용·물·불·출구 우선순위 | G08: 상태·구출/제거·트리거 프레임 일치 |
| COMP-11 | 시간 0·구출 목표±1·정수 백분율·최종 판정 | G08: 성공·실패·결과 화면·재시작 재현 |
| COMP-12 | F03 Pause·시계·입구 개방 버그 | G08: PauseGlitch 양쪽 설정 기록, 원작 모드 일치 |
| COMP-13 | Nuke 분모·중복 입력·개별 폭발·생성 중단 | G08: NukeGlitch와 최종 구출률·폭발 순서 일치 |
| COMP-14 | 겹친 개체 선택·우클릭 원격 할당·함정 사망 | G08: 원시 포인터 입력과 RightClickGlitch 재현 |
| COMP-15 | 같은 입력의 네이티브/웹·30/60/144Hz 렌더링 | G09: 논리 추적 해시 일치, 중지 후 복귀 처리 기록 |
| COMP-16 | 음악 21곡·효과음 18개·동시 발생·중단 | G10: 원작 매핑·명령 스트림·고정 합성기 PCM 증거 |
| COMP-17 | 전체 120개 진입점의 로딩·성공·실패·재시작 | G11: 각 레벨에 실행 기록과 판정, 미검증 0개 |
| COMP-18 | F01 실제 한 레벨 완주·저장한 입력의 반복 | G08/G09/G11: 동일 입력 3회와 양 플랫폼 상태·결과 일치 |

## 기준 실행 확보와 G00 완료 판단

기준 실행마다 파일 해시와 옵션을 먼저 확인합니다. DOS 에뮬레이터는 사용 시 정확한 버전·실행 파일 해시·설정·CPU 속도·VGA/AdLib 모드를 고정하고, `VGALEMMI /v` 직접 실행 명령과 원시 입력·시작 화면·관찰 영상을 기록합니다. 현재 동봉 LEMMINGS.BAT는 안내 분기로 이동하므로 자동으로 게임이 실행된다고 가정하지 않습니다. 같은 실행을 3회 반복해 재현성을 확인합니다. 원본 바이너리와 전체 캡처는 `assets/local/`에 보관하고 식별 매니페스트와 관찰 결과를 커밋합니다.

G05는 계측 가능한 Lemmix 실행/하네스와 F00의 구조화 추적을 확보합니다. G06/G08은 같은 원본 파일의 DOS 실행·입력·화면을 대조합니다. G10은 효과음 ID와 이벤트·곡 순서·오디오 속도를 DOS 실행으로 확인합니다. G11은 전체 레벨의 판정과 차이를 마무리합니다. 현재 환경에는 DOSBox 계열 실행 파일이 없으며 원작 화면·입력 재생 검증은 수행하지 않았습니다.

G00의 완료 기준은 입력·지원 목록·판정 계약·미검증 항목의 검증 책임을 정하는 것입니다. 이 문서로 해당 설계 결정을 고정하고 G01 착수 조건을 갖춥니다. 레퍼런스의 17Hz·음악 순서·선택 메커니즘은 채택한 계약이며 DOS 검증 완료를 의미하지 않습니다. 후속 증거가 계약과 다르면 별도 이슈에서 수정하고 영향받는 회귀 사례를 다시 검증합니다.
