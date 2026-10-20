# G10-01 AdLib 자원과 사운드 통합 계약

이슈: [#72](https://github.com/edwardkim/ohmylemmings/issues/72) · 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1)

[G00 오디오 기준](issue-5_compatibility_baseline.md#오디오-목록과-판정)과 [G09 완료](../reports/issue-70_web_playthrough.md)를 선행 조건으로 합니다. 이번 구현은 구조/주소의 초기 읽기 검증입니다. 명령 실행·원본 재생 타이밍·OPL/PCM·게임 이벤트 연결은 후속 절편입니다.

## 고정 출처와 형식

DOS [ADLIB 출처/해시](../../assets/manifests/dos_original.json), [기존 전체 목록](../../assets/manifests/dos_compatibility.json), [TS/Lemmix 고정 소스](../../assets/manifests/reference_sources.json)를 사용합니다. TS는10c7c8b9167258503d02589cf048e2d83a43dd0a, Lemmix는40e9bc34451f0e9127fd53b290a3877e700d143d입니다.

| 항목 | DOS-original 계약 |
| --- | --- |
| ADLIB.DAT | 정확히 한 DAT 구간, 해제 크기22125바이트 |
| 음악 표 | 2926부터21개의 LE 절대 헤더 주소; 첫 데이터2968 |
| 곡 헤더 | LE factor, LE 악기 상대 주소, wait byte, channel-count byte, 각 LE 채널 프로그램 상대 주소 |
| 상대 주소 기준 | 2926; 프로그램 첫 LE word도 같은 기준의 첫 bar 주소 |
| 음악 주소 범위 | 2968..21731; 헤더/프로그램2바이트·악기 첫16바이트·bar 첫1바이트를 완전히 읽을 수 있어야 함 |
| 음악 채널 | 1..8, 입력 순서 보존. TS는 효과음에 채널8을 사용 |
| 효과음 표 | 21989부터18개의 LE 절대 주소; 데이터22025..22125 |
| 곡 순환 | 표시 전체 레벨 순번0..119를21로 나눈 나머지; 실제 DOS 재생 순서는 후속 검증 |

근거: [TS initMusic/initSound](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/sound/sound-image-player.ts#L53), [채널 initMusic](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/sound/sound-image-channel.ts#L389), [프로필 설정](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/public/data/config.json), [Lemmix21곡 순환](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Styles.Dos.pas#L214).

`readWordBE`의 실제 구현은 리틀 엔디언입니다. TS numberOfTracks20은 기존 G00 기준과 달라21로 공급합니다. 서로 다른 곡이 bar21727을 공유하고 music-09의 채널 프로그램 주소 순서도 정렬되어 있지 않습니다. 곡마다 독립 구간으로 잘라내거나 주소를 정렬하면 안 됩니다. 현재 파서는 음악 전체 영역의 읽기 범위를 검증하며 곡별 소유권·모든 악기 번호·후속 점프/명령의 유효성은 검증하지 않습니다. factor와 wait_cycles는 원시 값이고 Hz/ms로 간주하지 않습니다.

## 자원 API와 오프라인 검증

`lemmings-resources`의 `AdlibFile::parse`는 DAT framing/checksum·구간 수·선언 크기를 검사합니다. `decode_into`는 기존 DAT decoder로 호출자 버퍼에 해제하고 `AdlibImage`를 만듭니다. `AdlibImage::parse`는 정확한 크기와21곡/18효과음 모두를 먼저 검증합니다. `music(index)`는 헤더 값과 순서 있는 프로그램/첫 bar 주소를 제공하며 `effect_offset(index)`는18효과음의 주소를 반환합니다. `music_for_level(LevelId)`는120개 표시 레벨의21곡 순환 계약입니다. 모든 라이브러리 코드가 no_std·무할당·읽기 전용이고 I/O/합성기·게임 시계가 없습니다.

범위 밖 인덱스(usize::MAX 포함), 이미지 절단/추가, 빈/과도한 채널,0 factor, 헤더/악기/프로그램/bar/효과음의 불완전한 첫 읽기, 누락/추가/손상 DAT 구간을 거절합니다. Rust9개 합성 테스트와 Python6개 비교기 테스트를 TDD로 추가했습니다. `make check-rust`는 합성 이미지→실제 Rust inspection→전체 필드·120레벨 대응을 비교하는 `tools/check_adlib.py`도 실행합니다. 원본/Node/네트워크가 없는 CI 경로입니다.

```sh
source assets/local/dev-tools/env.sh
python3 tools/check_adlib.py --evidence assets/local/adlib-synthetic-check
python3 tools/check_adlib.py --original-assets assets/local/dos-original --evidence assets/local/adlib-original-check
```

선택 원본 검사는 Node를 사용합니다. 모든 원본50파일과 선택 TS4파일의 크기/SHA를 확인하고 DAT 해제 SHA도 기존 기준과 대조합니다. 실제 고정 initMusic/initSound/채널 initMusic/readWordBE/readByte/setOffset 함수 본문과 enum을 실행하되 채널 생성은 메타데이터만 제공하는 stub입니다. TS 형식 주석만 제거하고 곡 수는 명시적으로21을 공급합니다. 전체 명령 VM이나 OPL/PCM을 실행한 증거가 아닙니다. 원본21곡·69채널의 모든 필드/첫 bar와18효과음,120레벨 대응이 일치했고 원본 해시를 종료 후 다시 확인했습니다. 원본 파생물은 ignored assets/local에만 쓰며 기존 디렉터리 덮어쓰기를 거절합니다.

## 논리 이벤트와 숫자 ID

[Lemmix 기본 효과음 enum](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Dos.Consts.pas#L275)과 [cue 위치](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L1971)를 후보 의미 계약으로 유지합니다. 다음18개는 논리 이름입니다. enum 순번이나 SoundData의 동적 WAV 인덱스를 ADLIB sfx-00..17과 같다고 간주하지 않습니다.

| 논리 이벤트 | 후속 연결/검증 지점 |
| --- | --- |
| BuilderWarning | 마지막3계단의 실제 warning edge |
| AssignSkill | 성공한 기술 할당 경계·수량 소비와 같은 순서 |
| Yippee | Exiting 진입 edge; saved 최종 누적에서 역추정하지 않음 |
| Splat | 치명적 착지 전환 |
| LetsGo / EntranceOpening | 기존 StartupEvent::LetsGo/EntranceSound(논리15/34); DOS 시각 대응은 별도 |
| Vaporizing / Drowning | 해당 위험물 전환 edge |
| Explosion / Ohno | 폭발/OhNo 진입·Nuke 분기 실제 순서 |
| HitsSteel | 실제 채굴 충돌 경계; 작업 중 매 프레임 반복 생성 금지 |
| SkillButtonSelect | 공유 패널의 실제 선택 경계; Pause/입력 순서 보존 |
| RopeTrap / TenTonTrap / BearTrap / ElectroTrap / SpinningTrap / SquishingTrap | GROUND의 raw sound 필드와 함정 trigger edge, 실제 숫자·기기 대조 필요 |

현재 core/session은 일부 시작 이벤트만 제공하며 위 전체 오디오 이벤트를 구현하지 않았습니다. 후속 변경은 typed event 출력으로 정확한 발생 순서를 기록하고 기본 상태/입력/지형 결과를 바꾸지 않아야 합니다. 출구/폭발 등 여러 전환이 같은 틱에 일어나면 actor ID·객체 슬롯·발생 순번을 함께 보존합니다. 화면/스냅샷 차이만으로 누락되기 쉬운 전환을 재구성하지 않습니다. Miner/Digger/Basher/OpenUmbrella/SilentDeath/Nuke의 Lemmix custom6효과음은 DOS18개의 추가 항목으로 집계하지 않습니다.

## 시간·합성기·플랫폼 책임

| 경계 | 고정할 계약과 후속 검증 |
| --- | --- |
| 자산 공급 | 후속 어댑터에서 ADLIB를 명시적으로 추가. 네이티브 I/O와 브라우저 File 바이트는 기존 불변 자산 묶음/검증 규칙을 따름. 웹 업로드/자동 배포 없이 사용하고 음원 누락/손상은 기존 게임을 보존 |
| 게임→오디오 | 시도 generation, control_step/iteration, 동일 경계 sequence, 논리 이벤트·actor/object 정보. 장치의 지연/가용성/음소거가 시뮬레이션을 진행하거나 변경하지 않음 |
| ADLIB VM | 정수 audio_step과 순서 있는 register/value. 반복/정지/동시 효과·음악 채널 우선순위·불법/끝없는 점프·명령 예산을 명시하고 비교 |
| 시간 | 게임17Hz와 음원 명령 시계 분리. 오디오 callback/rAF 횟수를 게임 틱으로 사용하지 않음. TS factor/210은 경험값이며 DOS 타이밍 증거가 아님 |
| OPL/PCM | 고정 구현·버전/라이선스·샘플률/형식·초기 상태를 다음 절편에서 선택. 같은 스트림→같은 PCM 바이트/해시; 렌더 주기/청크 분할 독립 |
| SDL | 기존 SDL2의 오디오 큐에 PCM 공급, 큐 상한과 underrun을 관측. 재시작/레벨 전환은 이전 generation의 대기 음원 폐기 |
| Web | 사용자 조작에서 AudioContext 생성/resume. 같은 PCM을 소비하며 브라우저 샘플률 차이·버퍼 청크·suspend/resume 처리 검증 |
| Pause/숨김/결과 | 기존 게임 시계와 G09 숨김 경계 유지. 음악 지속/중단·일시정지 중 UI음·결과 전환 정책은 DOS 확인 후 명시하고 시험 |
| 음소거/볼륨 | 논리/명령 진행과 출력 gain을 구분. 다시 켤 때 예전 효과음을 쌓아서 재생하지 않음. 장치 오류는 게임을 보존 |

SDL 경로는 기존 [SDL_QueueAudio](https://wiki.libsdl.org/SDL2/SDL_QueueAudio)와 [큐 크기 조회](https://wiki.libsdl.org/SDL2/SDL_GetQueuedAudioSize)를 우선 검토합니다. 큐는 바이트 단위이며 게임 시계의 원천으로 사용하지 않습니다. 새 OPL 의존성은 이번 단계에서 추가하거나 특정 구현으로 확정하지 않았습니다.

웹의 [AudioWorklet](https://developer.mozilla.org/en-US/docs/Web/API/AudioWorklet)은 secure context가 필요합니다. 현재 사용자가 쓰는 LAN HTTP 미리보기를 유지할 수 있도록, 설계상 기본 출력은 같은 PCM을 [AudioBufferSourceNode로 예약](https://developer.mozilla.org/en-US/docs/Web/API/AudioBufferSourceNode)하는 경로를 후보로 두고 G10-05에서 실제 HTTP/loopback 브라우저를 검증합니다. Worklet은 HTTPS/지원 환경에서 별도 검토합니다. [AudioContext.resume](https://developer.mozilla.org/en-US/docs/Web/API/AudioContext/resume)과 사용자 조작·자동 재생 정책도 같은 단계에서 시험합니다. 이번에는 서버/방화벽·브라우저 설정을 바꾸거나 소리를 재생하지 않았습니다.

## G10 분할과 완료 기준

1. **G10-01 / #72:** 현재 메타데이터/주소·전체 목록·통합/검증 계약.
2. **G10-02:** 제한된 ADLIB 명령 해석기와21곡/18효과음 레지스터 스트림, 반복/정지/오류·타이밍 근거. TS version1의 end-of-song 미구현도 검증 필요.
3. **G10-03:** OPL 구현/라이선스 선택·고정, 결정론 PCM·청크/샘플률/혼합/초기화 검증.
4. **G10-04:** 논리 이벤트·숫자 ID 대응과 세션 연결. 실제 DOS 이벤트/타이밍 증거와 전체 무음 회귀.
5. **G10-05:** SDL/Web 출력·사용자 활성화·음소거/볼륨·오류/숨김/재시작·LAN HTTP 경로.
6. **G10-06:** 실제 DOS·네이티브·브라우저 통합 청취/시각·스트림/PCM 대조와 G10 마감.

현재 단계의 통과는 음악/효과음 재생 인증이 아닙니다. 각 절편은 이슈 등록 후 TDD·전체 검사·출처/해시·자체 검토·정확한 게시 CI를 마쳐 완료합니다. G10 전체는 계속 진행 중이며 다음은 G10-02입니다.
