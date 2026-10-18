# G00-02 레퍼런스 코드 고정과 구현 진입점

작업 이슈: [#3](https://github.com/edwardkim/ohmylemmings/issues/3), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 고정 버전과 역할

G00-01의 데이터 분석에 사용한 두 커밋을 유지합니다. 참고 버전은 아래 커밋으로 식별합니다.

| 프로젝트 | 커밋 | 주된 참고 역할 |
|---|---|---|
| Lemmix | `40e9bc34451f0e9127fd53b290a3877e700d143d` | DOSORIG 틱·물리·직업 상태 및 리소스 해석 교차 확인 |
| Lemmings.ts | `10c7c8b9167258503d02589cf048e2d83a43dd0a` | 바이너리 파싱·리소스 조립·브라우저·AdLib 경계 |

파일 46개, 원본 446,049바이트와 소스 선언 290곳을 [소스 매니페스트](../../assets/manifests/reference_sources.json)에 고정했습니다. 원본 Git blob SHA-1, SHA-256, 인코딩, 선언의 줄 번호·접두 문자열을 기록합니다. 라이선스 원문 두 개도 같은 커밋으로 확보합니다.

## 확보·검증 명령

```sh
python3 tools/reference_assets.py fetch --manifest assets/manifests/reference_sources.json
python3 tools/reference_assets.py verify --manifest assets/manifests/reference_sources.json
```

로컬 위치는 `assets/local/reference-sources/`입니다. 파일명은 프로젝트 ID와 원본 경로를 `__`로 연결해 저장합니다. 원본 바이트를 유지하며, `text_encoding`으로 해석한 뒤 `symbols`의 선언 위치를 검증합니다.

## 영역별 읽을 순서

| 후속 목표 | 읽을 위치·함수 | 확인할 내용 |
|---|---|---|
| G02 DAT 구간·압축 | [FileContainer.read](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/file/file-container.ts#L31) / [UnpackFilePart.doUnpacking](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/file/unpack-file-part.ts#L55) / [TDosDatDecompressor.DecompressSection](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Dos.Compression.pas#L255) | 헤더·구간 경계, 역방향 비트 읽기, 압축 해제, 체크섬과 오류 처리 |
| G02 비트·바이트 조작 | [BinaryReader.readWord](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/file/binary-reader.ts#L121) / [BitReader.read](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/file/bit-reader.ts#L33) / [BitWriter.copyReferencedData](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/file/bit-writer.ts#L34) | 정수 읽기 순서와 출력 버퍼의 참조 복사; Rust의 경계 검사·오류 정책은 별도 정의 |
| G03 레벨 순서·변형 정보 | [TDosOrigLevelSystem.GetEntry](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Styles.Dos.pas#L183) / [LevelIndexResolve.resolve](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/level-index-resolve.ts#L27) / [OddTableReader.getLevelProperties](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/lemmings/odd-table-reader.ts#L16) | 난이도·레벨 번호를 DAT 구간과 ODDTABLE 정보로 연결 |
| G03 레벨 구조·통합 | [TLVLRec](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Dos.Structures.pas#L79) / [TLevelLoader.TranslateLevel](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Level.Loader.pas#L74) / [LevelReader.readLevelInfo](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/lemmings/level-reader.ts#L46) / [LevelLoader.getLevel](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/level-loader.ts#L26) | 메타데이터·객체·지형·강철 영역 해석과 리소스 결합 순서 |
| G03 지형·특수 그래픽 | [GroundReader.readImages](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/lemmings/ground-reader.ts#L73) / [VgaSpecReader.readImage](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/lemmings/vga-spec-reader.ts#L48) / [GroundRenderer.copyImageTo](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/ground-renderer.ts#L41) | GROUND·VGAGR·VGASPEC 해석, 팔레트와 그리기 플래그 |
| G03 스프라이트 | [TMainDatExtractor.ExtractAnimation](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Dos.MainDat.pas#L77) | MAIN.DAT의 애니메이션·비트맵 추출 진입점 |
| G04 픽셀 지형 | [SolidLayer.hasGroundAt](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/solid-layer.ts#L14) / [SolidLayer.clearGroundAt](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/solid-layer.ts#L31) / [TLemmingGame.HasPixelAt](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L1824) / [TLemmingGame.RemovePixelAt](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L1838) | 충돌 마스크와 지형 변형의 분리, 경계 처리 |
| G05 틱·생성·상태 전환 | [TLemmingGame.InternalUpdate](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L3408) / [TLemmingGame.IncrementIteration](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L3498) / [TLemmingGame.CheckSpawnLemming](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L4001) / [TLemmingGame.Transition](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L1927) | 한 틱의 호출 순서, 생성 간격, 상태 전환과 초기화 |
| G05 걷기·낙하 | [TLemmingGame.HandleWalking](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L2826) / [TLemmingGame.HandleFalling](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L3210) / [LemmingManager.tick](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/game-play/lemming-manager.ts#L102) | 개체 순서와 이동·충돌 처리; 틱 시간과 입구 선택 차이는 아래 항목 참고 |
| G06 지형 렌더링 | [TRenderer.RenderWorld](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.Rendering.pas#L571) / [GroundRenderer.fromGroundMap](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/ground-renderer.ts#L23) | 정적 지형 합성과 렌더러 경계; 네이티브 SDL2 어댑터는 Rust에서 구현 |
| G08 세션·판정 | [TLemmingGame.CheckForGameFinished](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L2232) / [TLemmingGame.CheckForInteractiveObjects](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L2342) / [Game.getGameState](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/game.ts#L215) | 시간·구출·생존 수와 상호작용 결과의 판정 진입점 |
| G09 웹 경계 | [Game.onGameTimerTick](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/game.ts#L206) / [Stage.redraw](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/view/stage.ts#L242) / [Stage.createImage](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/view/stage.ts#L261) | 브라우저 화면·입력과 코어 경계; Rust에서는 논리 틱과 렌더링 빈도를 분리 |
| G10 원작 오디오 | [SoundImageManager.getMusicTrack](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/sound/sound-image-manager.ts#L19) / [SoundImagePlayer.read](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/sound/sound-image-player.ts#L140) / [SoundImageChannels.read](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/sound/sound-image-channel.ts#L63) / [AudioPlayer.audioScriptProcessor](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/sound/audio-player.ts#L85) | ADLIB 이미지 → 채널 명령 → 오디오 출력 경로; 합성기 의존성은 G10에서 추가 분석 |

G01의 모듈 경계는 위 영역을 바탕으로 설계합니다. G11의 회귀·호환성 검증은 G00-03의 시나리오와 이 문서의 원본 위치를 연결합니다.

## G07 직업별 진입점

할당과 지속 행동을 함께 확인합니다. TS에서는 각 클래스의 `triggerLemAction`과 `process`가 대응 지점입니다.

| 직업 | Lemmix 할당·행동 | Lemmings.ts 대응 |
|---|---|---|
| Climber | [TLemmingGame.AssignClimber](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L2039) / [TLemmingGame.HandleClimbing](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L2947) | [ActionClimbSystem.process](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/game-play/actions/action-climb-system.ts#L47) |
| Floater | [TLemmingGame.AssignFloater](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L2059) / [TLemmingGame.HandleFloating](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L3255) | [ActionFloatingSystem.process](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/game-play/actions/action-floating-system.ts#L50) |
| Bomber | [TLemmingGame.AssignBomber](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L2075) / [TLemmingGame.HandleExploding](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L3363) | [ActionCountdownSystem.process](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/game-play/actions/action-countdown-system.ts#L44) |
| Blocker | [TLemmingGame.AssignBlocker](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L2089) / [TLemmingGame.HandleBlocking](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L3319) | [ActionBlockerSystem.process](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/game-play/actions/action-blocker-system.ts#L43) |
| Builder | [TLemmingGame.AssignBuilder](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L2103) / [TLemmingGame.HandleBuilding](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L2996) | [ActionBuildSystem.process](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/game-play/actions/action-build-system.ts#L45) |
| Basher | [TLemmingGame.AssignBasher](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L2129) / [TLemmingGame.HandleBashing](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L3056) | [ActionBashSystem.process](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/game-play/actions/action-bash-system.ts#L53) |
| Miner | [TLemmingGame.AssignMiner](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L2162) / [TLemmingGame.HandleMining](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L3125) | [ActionMineSystem.process](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/game-play/actions/action-mine-system.ts#L51) |
| Digger | [TLemmingGame.AssignDigger](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L2195) / [TLemmingGame.HandleDigging](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L2907) | [ActionDiggSystem.process](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/game-play/actions/action-digg-system.ts#L43) |

Bomber는 [ActionOhNoSystem.process](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/game-play/actions/action-ohno-system.ts#L37)와 [ActionExplodingSystem.process](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/game-play/actions/action-exploding-system.ts#L51)까지 이어서 확인합니다. 걷기·낙하 등의 공통 상태도 소스 매니페스트에 포함했습니다.

## 정적으로 확인한 차이와 후속 판단

| 항목 | 관찰한 차이 | 적용·검증 방향 |
|---|---|---|
| 논리 시간 | Lemmix는 초당 17틱, TS는 틱 간격 60ms와 그 간격을 이용한 시간 변환 | G05의 후보 기준은 Lemmix이며, G00-03에서 입력 적용 시점·시간 경계 시나리오 확정 |
| 압축 체크섬 | Lemmix는 오류를 발생시키고, TS는 불일치를 로그에 남긴 뒤 출력 버퍼 반환 | G02는 손상 입력과 출력 유효성을 명시적으로 검증하는 오류 API 설계 |
| 생성과 입구 | Lemmix는 생성 간격 산식·입구 순서 테이블을 사용하고, TS는 다른 간격 산식과 첫 입구를 사용 | 다중 입구·연속 생성은 Lemmix를 우선 분석하고 G00-03의 비교 사례에 포함 |
| 사운드 | Lemmix는 MOD 파일과 BASS 재생 경로를 사용하고, TS는 ADLIB 이미지의 명령을 해석 | 원작 ADLIB 데이터 해석은 TS 경로를 참고하고 G10에서 실제 명령·재생 결과 검증 |

근거 위치:

- 논리 시간: [DOS_FRAMES_PER_SECOND](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Dos.Consts.pas#L39) / [GameTimer.ticksToSeconds](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/game-play/game-timer.ts#L117)
- 체크섬: [TDosDatDecompressor.DecompressSection](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Dos.Compression.pas#L255) / [UnpackFilePart.doUnpacking](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/file/unpack-file-part.ts#L55)
- 생성: [TLemmingGame.CheckSpawnLemming](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L4001) / [LemmingManager.addNewLemmings](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/game-play/lemming-manager.ts#L139)
- 사운드: [TDosOrigLevelSystem.DoInitializeLevelSystem](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Styles.Dos.pas#L214) / [SoundLibrary.PlayMusic](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.Sound.pas#L233) / [SoundImagePlayer.initMusic](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/sound/sound-image-player.ts#L81)

Lemmix의 한 틱에서는 논리 카운터 증가 → 생성 → 개체 처리 → 전체 폭발 갱신 → 상호작용 객체 갱신이 이어집니다. 같은 메서드 안에 렌더링·리플레이·사운드 처리도 있으므로, Rust에서는 기존 설계에 맞춰 코어와 플랫폼 책임을 분리합니다. 입력 적용과 일시정지의 경계는 G00-03에서 비교 사례로 정의합니다.

## 확인 범위와 남은 작업

이번 결과는 고정 소스의 바이트·선언 위치와 정적 호출 관계입니다. 원작 실행 파일과 두 레퍼런스의 실행 결과를 대조한 상태는 아닙니다. 선택한 파일은 구현 진입점이며, 후속 작업에서 추가 의존 파일을 읽을 때도 같은 커밋을 사용하고 필요한 잠금 항목을 확장합니다.

일부 Delphi 파일은 CP1252입니다. 바이트 전송과 해시 불일치의 처리 방법은 [소스 전송 문제 해결 기록](../troubleshootings/issue-3_github_source_bytes.md)에 남겼습니다.

- [작업일지](../orders/20261018.md#issue-3)
- [진행 기록](../working/issue-3_reference_code_map.md)
- [결과 보고서](../reports/issue-3_reference_code_map.md)
