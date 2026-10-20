# G10-06 통합 사운드 대조

이슈: [#82](https://github.com/edwardkim/ohmylemmings/issues/82) · 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1)

## 게임에서 PCM까지

`sound_playthrough` 네이티브 실행과 실제 WASM이 같은580개 제어 경계를 실행한다. 기본은 합성 장면이며 원본 검사는 Fun1과 ADLIB.DAT를 사용한다. Pause/Resume, 선택·Digger 할당, 음소거/복귀, 음량0/100, Nuke, 결과 정지, 재시작을 포함한다. 원본은 Lost, 합성은 Won을 거쳐 새 세대로 바뀐다.

경계 n의 누적 출력은 `floor(n * 44100 * 156250 / 2190197)`이다. Pause에도 PCM 시간은 진행하고 명시적 mute/결과는 무음이다. native는 경계마다 한 번, WASM은1/127/512/3/2048/7프레임으로 나눈다. 매 경계의 전체 게임·사운드 사건/세대·flush·오디오 위치와18개 RGB/지형/객체 snapshot을 비교한다. 음향을 소비하지 않는 별도 게임과도 상태/맵/화면을 대조한다. 합성·원본 각각1,824,750프레임(41.377551초)이 바이트 전체로 같다. 실제 장치/활성화는 [#80](issue-80_audio_output.md)의 근거와 아래 녹음을 연결한다.

```sh
source assets/local/dev-tools/env.sh
python3 tools/sound_playthrough.py
python3 tools/sound_playthrough.py --original-assets assets/local/dos-original --evidence assets/local/NEW_SOUND
python3 tools/observe_audio_dos.py --evidence assets/local/NEW_DOS
python3 tools/sound_playthrough.py --dos-capture assets/local/NEW_DOS/run/capture --evidence assets/local/NEW_DOS_WAV
```

원본 검사/녹음은 새 ignored 경로만 허용한다. 원본 없는 `make check-web`은 합성 통합 경로를 실행한다. WAV 추출은 AVI의 PCM_s16le 첫 채널을 gain·정규화·재샘플링·downmix 없이 보존한다.

## 실제 DOS와 OPL

DOSBox0.74-3, `/v -o`, VGA/fixed3000cycles/OPL2/44100Hz로 실행했다. 생성 RUSSELL의 검증된 파일명 prefix를 유지하며 disk-counter byte0만0으로 seed하는 기존 절차다. 원본 key disk 인증은 아니며 EXE/DAT50파일은 전후 동일하다. AdLibFF·첫 음악 raw6·IRQ 약69.890621Hz를 관측했다. 게임 녹음25.967982초/1,145,188프레임은 nonzero이며 메뉴0.97초 무음은 음향 통과에 집계하지 않는다.

별도 소유 DOSBox mapper에 `hand_caprawopl "key 111"`을 추가하고 게임 진입 전/종료 전에 o를 눌러 DRO를 기록했다. 설정·입력·캡처와 해시는 ignored 증거에 있다. [DOSBox RawHeader/Capture 소스](https://sources.debian.org/src/dosbox/0.74-4/src/hardware/adlib.cpp)와 설치본 README를 확인했다. DRO v2에는 변환표·밀리초 지연·시작 cache가 있고 같은 값의 반복 쓰기는 생략된다. `tools/dro.py`는 버전/크기/표 인덱스/총 지연/절단·후행 데이터를 검사한다.

```sh
cargo build -p lemmings-resources --example trace_adlib --release --locked --offline
# IMAGE: DAT 해제로 검증한22125바이트 ADLIB 이미지
target/release/examples/trace_adlib IMAGE m5:1700 > assets/local/native-music.jsonl
python3 tools/dro.py --capture CAPTURE.dro --native assets/local/native-music.jsonl --evidence assets/local/NEW_OPL
```

음악4채널의 keyed F-number/block 순서를 정확히 비교한다. 채널별 첫 사건 정렬 뒤 상대 시간 허용차는 IRQ 한 번+1ms(15.268123ms)다. 실제23.099초/1,434쓰기에서272사건(95/93/72/12)이 같고 최대 오차는4.122914ms다. 첫 음악 주파수/템포의 증거이며 전체 register·전곡·효과음·절대 장치 위상의 동일성이 아니다. 전체 명령/반복·독립 C PCM·효과음 ID 근거는 [#74](issue-74_adlib_stream.md)·[#76](issue-76_opl_pcm.md)·[#78](issue-78_game_sound.md)에 있다.

## 실제 SDL과 사용자 청취

원본 Fun1을180경계 실행하고 소유 PID의 PulseAudio/PipeWire 스트림만 전용 null sink로 이동했다. 그 monitor만44100Hz mono로 녹음하고 임시 장치/프로세스를 정리했다.11.888617초/524,288프레임에서419,057nonzero 샘플을 확인했다. 마이크·다른 앱·물리 스피커 녹음이 아니다. 첫 두 라우팅 실패는 보존하고 비동기 이동 완료를 확인한 마지막 실행만 성공에 집계한다.

[로컬 청취 페이지](http://192.168.2.146:8080/listening/)에서 DOS/native/WASM/SDL을 하나씩 재생한다. DOS와 통합 시나리오의 입력/시작 시점은 다르다. native/WASM WAV는 완전히 같지만 DOSBox DBOPL/믹서와 Nuked PCM의 음량·음색/파형은 다르다. 탐색적10초 RMS-envelope 상관도0.161을 음질 동등성 통과로 쓰지 않았다. 실제 OPL 캡처로 주파수·템포 일치와 파형 차이를 분리했다. 2026-10-21 사용자에게서 “사운드 체크 통과 판정입니다.”라는 청취 검증 결과를 수신하여 G10을 마감한다. 물리 DAC 계측·전곡 전구간 검사·DOS 합성기와의 파형 또는 절대 위상 동등성은 이 판정의 범위에 포함하지 않는다.

[선정 증거 해시](../../assets/manifests/audio_playthrough.json) · [검사 결과/남은 작업](../reports/issue-82_audio_playthrough.md).
