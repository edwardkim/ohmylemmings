# G10-05 SDL/Web 오디오 출력

이슈: [#80](https://github.com/edwardkim/ohmylemmings/issues/80) · 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1)

## 소유와 시계

`lemmings-game::audio::AudioOutput`은 검증된 이미지 Vec와 opaque `PcmContinuation`, 고정4096쓰기 workspace를 소유합니다. `AdlibContinuation`/`PcmContinuation`은 자원 borrow 없이 VM/칩/샘플 위상을 옮깁니다. resume은 **같은 불변 이미지 내용**에 바인딩하며 새 이미지에는 새 인스턴스를 만듭니다. self-reference·leak·unsafe 없이 네이티브/실제 WASM의 owned 수명을 지원합니다. 기존 borrowed API와 정수 엔진은 유지합니다. resources/core와 audio는 no_std·무할당이며, 소유·장치는 std 게임/플랫폼 경계에 있습니다. 새 외부 패키지는 없고 Game→Audio 로컬 의존성만 추가했습니다.

PCM은 #76의 mono left i16·8000..192000Hz·VGA2190197/31250Hz driver clock을 사용합니다. 장치가 요청하는 샘플과 게임 제어 경계는 독립적입니다. Pause는 게임 iteration만 멈추므로 음악은 계속됩니다. `AudioOutput`은 게임을 tick하거나 렌더하지 않습니다. 출력 요청은8192프레임 이하, 음량은 정수0..100이며 `i32(sample)*volume/100`으로 스케일합니다. 0 음량은 PCM 시간을 진행하며0을 출력합니다.

원본은 선택적 ADLIB.DAT를 동일 parser/decoder로 읽습니다. 누락/손상은 음향 오류이며 기존 게임은 계속됩니다. 웹의 논리 파일 이름 목록에 ADLIB.DAT를 추가했고 이미 로드한 bank는 Rc 불변 snapshot입니다. 연습 모드는 새로 작성한 짧은 음악/효과음 데이터를 사용하며 DOS 음악을 복사하지 않습니다.

## 소비와 복구

한 소비자가 SoundBatch를 비웁니다. 전체 batch의 ID/세대/sequence를 사전 검사하며 잘못된 ID·이전 세대·중복 사건은 이전 재생 상태를 보존합니다. 정상 상태에서는 순서대로 typed 명령을 다음 driver boundary에 전달합니다. 첫 활성화·음소거/숨김 후 재개·세대 변경·손실 진단은 새 VM/칩/위상으로 **현재 선택 음악을 처음부터** 시작하고 이미 발행된 오래된 효과음을 버립니다. wall-time을 합성기/게임에 catch-up하지 않습니다. 아직 처리되지 않은 게임 입력의 기존 재생 의미는 유지합니다.

명시적 음소거/숨김/중단/결과는 장치 큐를 비우고 PCM 소비를 멈춥니다. 재시작/Continue는 #78의 음악 cursor/새 세대를 따릅니다. 결과 Stop 후 이전 음원이 재생되지 않습니다. Slider 음량 변경은 큐를 비우고 다음 샘플부터 새 gain을 씁니다. 오디오 설정은 gameplay 입력 로그에 들어가지 않습니다.

## SDL

SDL2 0.38 AudioQueue<i16> 모노, 원하는44100Hz와512샘플 callback quantum을 요청하고 실제 spec의 rate로 PCM을 만듭니다. 각 service는512프레임을 최대4번,2048목표까지 채우며3072상한을 검사합니다. 무한 refill이나 큰 지연 큐를 만들지 않습니다. 장치가 이미 소비한 시간을 되돌리거나 실제 DAC의 위상 일치를 주장하지 않습니다.

기본 음량40%, S로 켜기/음소거·실패 후 재시도, PageUp/PageDown으로10%씩 조절합니다. 창 제목에 상태를 표시하며 게임의 DOS canvas는 유지합니다. `--mute`, `--volume 0..100`; `--fast`는 장치를 사용하지 않습니다. 창 숨김/최소화는 음향을 중단하고 게임의 기존 진행 정책은 유지합니다. 장치/PCM 오류는 stderr/제목에 표시하고 게임을 계속합니다. 재시도 때 자원과 장치를 다시 구성합니다.

```sh
source assets/local/dev-tools/env.sh
cargo run -p lemmings-desktop --release --locked --offline -- --assets assets/local/dos-original
SDL_AUDIODRIVER=dummy target/release/lemmings-desktop --audio-probe
python3 tools/check_audio_output.py
```

## 웹과 LAN HTTP

AudioHost는 첫 명시적 버튼/키 조작까지 AudioContext를 만들지 않습니다. 사용자 handler 안에서 resume을 호출하며 실패/3000ms timeout은 음향만 중단하고 재시도를 제공합니다. ticket은 늦게 끝난 활성화가 음소거/숨김을 취소하지 못하게 합니다. 장치/컨텍스트 종료 시 새 컨텍스트의 rate로 새 WebAudio를 만듭니다. API가 없는 브라우저에서는 소리 버튼만 비활성화하고 게임을 유지합니다.

WebAudio의 prepare는 단일 큐 소비와 세대/음원 확인, render는 bounded PCM을 담당합니다. AudioBuffer로 i16/32768을 정확히 복사하고 AudioBufferSourceNode를 예약합니다.512프레임 단위, 서비스당 최대8개 node,60ms목표/5ms시작 여유이며20ms timer를 사용합니다. source 종료/음소거/숨김/결과/세대에서 stop/disconnect하고 큰 event-loop 지연은 큐와 음악을 재동기화합니다. requestAnimationFrame이나 오디오 timer는 상대편 시뮬레이션을 tick하지 않습니다. Page hide는 dispose합니다.

LAN HTTP에서도 사용하는 AudioContext/AudioBufferSource 경로이며 secure context를 요구하는 AudioWorklet을 사용하지 않습니다. 관련 근거: [Web Audio 명세](https://www.w3.org/TR/webaudio-1.1/), [SDL AudioQueue API](https://docs.rs/sdl2/0.38.0/sdl2/audio/struct.AudioQueue.html). 원격 HTTP8080/방화벽과 loopback CDP19222는 유지합니다. 웹도 S·PageUp/PageDown을 지원하고 소리 버튼/음량 slider를 제공합니다. 원본 파일을 선택할 때 ADLIB.DAT도 함께 선택합니다.

```sh
make web
python3 tools/check_audio_output.py --wasm
python3 tools/check_audio_output.py --wasm --original-assets assets/local/dos-original --host 192.168.2.146 --evidence assets/local/issue-80-new-original
```

## 라이선스와 재빌드 배포

브라우저 footer의 라이선스 페이지와 빌드 산출물 `LGPL-2.1-or-later.txt`, `audio-source.tar.gz`, `audio-source.json`을 함께 제공합니다. 수정된 코어·전체 라이선스·UPSTREAM.patch·relink 안내·애플리케이션/고정 빌드 파일을 소스 묶음에 포함합니다. 실행 중인 WASM과 묶음/라이선스 해시를 sidecar에 연결합니다. [기존 고지](../../third_party/README.md), [코어 재링크 안내](../../crates/lemmings-opl/README.md)를 유지합니다.

Git 공개 추적 소스의 현재 내용과 apps/crates/tools의 새 빌드 소스를 수집합니다. `assets/local`·target·무관한 미추적 경로·비공개 환경 파일·symlink를 제외하며 필수 고지/빌드 파일 누락은 실패합니다. SOURCE-MANIFEST에 정렬된 경로/크기/mode/SHA를 기록하며 tar/gzip 메타데이터를 정규화합니다. 추출본은 Git 없이 SOURCE-MANIFEST와 소스 파일을 사용해 수정/재빌드할 수 있습니다. 원본/PCM/영상·개인 로그는 묶음에 넣지 않습니다.

## 검증 범위

기본 회귀는 원본 없이6개 rate/volume 조합·249492프레임의 native/실제 WASM·고정 PCM/상태/지형/객체/RGB 해시, 실제 OfflineAudioContext의 float 샘플 일치와 실시간 AudioContext의 nonzero graph를 검사합니다. SDL 생산 service의 queue 소비·음소거/숨김/세대와 dummy 장치, 장치 부재에서도 게임 지속, 음향 켜기/끄기의20개 상태/캡처 파일 일치를 확인합니다. blocked/race/context recreation/clock jump와 queue 상한을 mock으로 별도 검사합니다.

CI headless graph 검사는 `--autoplay-policy=no-user-gesture-required`를 명시합니다. 이는 사용자 gesture 정책 인증이 아닙니다. 별도 실제 Chromium19222의 새 탭에서 autoplay flag 없이 trusted Input 클릭·userActivation을 관측해 LAN의 활성화/음소거/숨김/복귀·키보드·음량0·재시작을 검증했습니다. 이 탭과 관측 스크립트만 만들고 닫았으며 기존 탭은 보존했습니다. 연결된 CUA 표면이 없어 이미 허용된 CDP를 사용했습니다.

선택적 원본6조합/249492프레임·비보안 LAN origin·실시간 original graph를 대조했고 원본50개 전후 해시는 동일합니다. 실제 로컬 SDL backend는 PipeWire에서 큐 소비를 확인했습니다. 이는 물리 DAC/스피커 청취, 전체 곡/반복, DOS와의 실제 비동기 위상/전체 통합 비교가 아니며 G10-06에서 마감합니다. 재링크 실험의 의도적 무음 라이브러리는 별도 ignored 복사본이며 제품 합성 결과와 구별합니다.
