# G10-03 OPL 합성기와 PCM·시간 계약

이슈: [#76](https://github.com/edwardkim/ohmylemmings/issues/76) · 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1)

`lemmings-resources`의 순서 있는 쓰기를 새 `lemmings-audio`가 샘플 시간으로 배치하고, 독립 `lemmings-opl`이 정수 PCM으로 변환합니다. 두 새 라이브러리는 no_std·무할당·unsafe 금지이며 플랫폼/렌더/게임 시뮬레이션에 의존하지 않습니다. SDL/Web 장치 출력과 게임 이벤트 연결은 G10-04/05 후속입니다.

## 합성기 선택·출처·라이선스

[공식 nuked-opl3 0.1.0](https://crates.io/crates/nuked-opl3/0.1.0)의 순수 Rust `src/core.rs`를 고정했습니다. [Rust 상류](https://github.com/tgies/nuked-opl3-rs)의 revision은 `eea782bc5838bfc42add4de690d6ec5d960dc1a7`, crate SHA-256은 `fe763ce652db623a34372b1628588faaee86af83195175acee070d92d1fe27b7`입니다. LGPL-2.1-or-later 원문·저작권·수정 표시·선호 소스/빌드 설정·재링크 설명을 [별도 라이브러리](../../crates/lemmings-opl/README.md)와 [고지](../../third_party/README.md)에 포함합니다. 제품 바이너리 배포 시 이 소스/라이선스 및 수정 라이브러리를 재링크할 자료도 함께 제공합니다. 저장소 전체의 라이선스를 새로 지정하지 않습니다.

상류 장치 래퍼/빌드 스크립트/C FFI는 채택하지 않았습니다. stereo-ext를 false로 고정하고 도달할 수 없는 부동소수 panpot을 제거했습니다. 정수 합성·상태 전이·샘플 순서는 유지합니다. 정확한 차이는 [UPSTREAM.patch](../../crates/lemmings-opl/UPSTREAM.patch), 원본/수정 소스 해시는 [provenance](../../assets/manifests/adlib_pcm.json)에 있습니다. API는 u8 저위 뱅크 레지스터만 허용해 OPL2 모드로 제한합니다. 상류 코어 스타일의 Clippy 예외는 비공개 모듈에만 적용하며 unsafe 금지는 그대로 적용됩니다. 새 외부 Cargo 패키지는 추가하지 않았습니다. Cargo.lock의 변경은 로컬 workspace 패키지 세 개뿐입니다.

후보 `oplon 1.0.0`은 MIT/no_std 정수 구현과 자체 테스트를 통과했지만 독립 C 비교에서 원본 음악0/6/15의 RMS가 기준의 약2.22/2.65/2.97배였습니다. 탐색적 스펙트럼 코사인도 약0.934/0.731/0.840이었습니다. 이를 동등한 음원으로 승인하지 않고 교체했습니다. 이 결과는 해당 입력/버전의 관측이며 패키지 일반 품질 판정이 아닙니다. `opl-emu 0.4.2`(GPL-3.0), `opl-audio 0.1.0`(MIT OR Apache-2.0)의 공식 crate 메타데이터/소스도 별도 보관했으며 런타임에 설치하지 않았습니다.

독립 기준은 [Nuked-OPL3 C1.8](https://github.com/nukeykt/Nuked-OPL3/tree/765ec962e473aeb767e4cba74ffdc8f588ffbfe8)입니다. 수정하지 않은 C/헤더/전체 LGPL 라이선스를 [검증 전용 폴더](../../tools/audio/reference/README.md)에 포함하고 [세 파일의 크기/SHA](../../assets/manifests/opl_reference.json)를 컴파일 전에 검사합니다. 별도 host 프로세스로만 실행하며 제품에 링크하지 않습니다. Rust PCM과 C PCM을 같은 순서의 쓰기·절대 샘플 경계에서 **바이트 전체** 대조합니다. 같은 Rust 합성기를 두 번 실행하는 검사와 구별합니다.

## 원본 시간 근거

고정 `VGALEMMI.EXE` SHA-256은 `4f21e94d53b7ea814ade802b4a37d42481da3876e31fb5a23090e19c9600f21c`입니다. MZ 헤더0x120 이후 원본 코드의0x1843에서 DS:001c를 매 IRQ 증가시킵니다. 홀짝 분기는 PC speaker INT60에만 적용하며 AdLib 선택(DS:2010=FF) 및 enable mask(DS:25a7 & 3)에서는0x1872의 INT61/AX=0을 **매 IRQ** 호출합니다. 35Hz나 게임 제어 빈도로 나누지 않습니다. DS:1fcd는 게임이 매5IRQ 초기화하는 budget이며 누적 시계가 아닙니다.

`tools/verify_adlib_timing.py`는 검증된 원본 코드0x1843..0x1874를 Unicorn2.1.4에서 직접 실행합니다. 합성 enable flags와 512IRQ(두 번 u8 wrap)를 공급해 mask1/2/3 각각512AdLib호출, speaker256호출, muted0호출을 확인했습니다. INT payload만 기록하며 handler·전체DOS·벽시계는 실행하지 않습니다.

기본 VGA 경로의 IRQ는 수직 재추적에 동기화합니다. 기존 [DOS 시간 기준](../../assets/manifests/dos_calibration.json)과 새 실제 DOSBox0.74-3 실행(`/v -o`, fixed3000cycles)의640×400 캡처에서 VGA 메타데이터 `2190197/31250 Hz`를 확인했습니다. 읽기 전용 RAM 샘플1659개/19.889104237초에서 DS:001c의1390증분, 약69.887511Hz를 관측했습니다(비동기 관측 허용오차2Hz). AdLib 선택FF·잠금해제 flag0을 확인했습니다. 캡처 메타데이터는 가상 VGA 주기이며 RAM 관측은 비원자적 호스트 샘플이므로 실제 하드웨어 크리스털이나 DAC/장치 PCM 인증은 아닙니다.

`-x` 경로는 재추적 동기화를 건너뛰고 PIT1193182/divisor16384를 사용합니다. `AudioRate::DOS_UNLOCKED=596591/8192`는 그 정적 경로의 유리수이며 이번 실제 DOS 관측은 기본 VGA 경로입니다. 헤더 `sample_rate_factor`는 드라이버 상태에 저장되지만 IRQ 스케줄을 바꾸지 않습니다. TS 경험식 factor/210은 사용하지 않습니다.

DOS 실험은 소유한 별도 Xvfb/DOSBox에서 실행했습니다. 원본 EXE/DAT50파일 해시를 실행 전후 확인하고 수정하지 않았습니다. 원본이 생성한256바이트 RUSSELL DS window의 카운터9·불변 파일명 prefix를 EXE와 대조한 후 로컬 런타임 사본의 **byte0만0**으로 설정했습니다. 후미 가변 메모리 때문에 역사적 생성 해시와 다르며 새 해시/변경 offset을 보존했습니다. 원본 key disk의 진위/우회 없는 실행을 인증하지 않습니다. fixed10000cycles에서는 AdLib 선택이 관측되지 않아 실패로 남겼고3000에서 선택FF를 확인했습니다. 초기 관측기의 DS:1fcd 오용은 회귀 테스트와 DS:001c로 수정했습니다. 실패한 자료도 삭제하거나 성공 증거로 집계하지 않았습니다.

## PCM 계약

- 출력: mono signed i16, 명시적 little-endian 직렬화, 고정 left 신호, saturating 변환. Nuked OPL2 모드에서도 좌우 pipeline의 작은 시간차가 있으므로 좌우 동일성을 가정하지 않습니다. 자동 음량 정규화/가변 gain은 없습니다.
- 샘플률8000..192000Hz, 호출률1..1000Hz 양의 기약 유리수. 기본 `DOS_VGA=2190197/31250`이며 게임 틱·렌더·호스트 경과시간과 독립입니다.
- interval n의 끝은 `floor(n * sample_rate * denominator / numerator)`. u64 나머지를 보존해 청크 분할에 따른 누적 반올림을 피합니다. 해당 호출의 레지스터 쓰기를 interval 첫 샘플 전에 순서대로 적용합니다.
- render당 최대8192프레임. 호출자 소유 레지스터 scratch와 호출별 최대4096명령 한도를 사용합니다. 먼저 복사한 작은 VM/위상으로 **전체 요청**을 검증하고 같은 불변 이미지로 실제 처리합니다. 늦게 발견된 명령/용량/오버플로 오류도 기존 PCM·VM·칩 이력·위상을 보존합니다. `advance_silent`는 쓰기를 생략하되 원본 VM과 동일한 상태/쓰기 개수를 제공합니다.
- 음악/효과음/stop은 다음 호출 경계에서 적용합니다. 빈 render는 상태를 소비하지 않습니다. stop은 단조 샘플 시계/칩 release를 유지합니다. `reset_music`은 인덱스를 검증한 뒤 VM·칩·resampler·위상을 새 세대로 초기화합니다. 잘못된 인덱스는 기존 스트림을 보존합니다. 장치 queue/generation의 폐기는 플랫폼 책임이며 후속입니다.

## 재현과 검증 범위

`source assets/local/dev-tools/env.sh` 후 저장소 루트에서 실행합니다. 기본 검사는 원본 없이 오프라인이며 C99 컴파일러(`cc`), Rust1.93.1, actual WASM 검사에는 고정 wasm-bindgen과 Chrome/Chromium이 필요합니다.

```sh
python3 tools/check_opl_reference.py
python3 tools/check_audio.py
python3 tools/check_audio.py --wasm
make check
```

선택적 원본 검사는 검증된50파일이 있어야 하며 **새 ignored 경로**를 사용합니다.

```sh
python3 tools/check_audio.py --wasm --original-assets assets/local/dos-original --evidence assets/local/NEW_PCM_PATH
python3 tools/check_opl_reference.py --original-assets assets/local/dos-original --evidence assets/local/NEW_C_PATH
assets/local/issue-74-tools/venv/bin/python tools/verify_adlib_timing.py --original-assets assets/local/dos-original --evidence assets/local/NEW_IRQ_PATH/summary.json
python3 tools/observe_audio_dos.py --evidence assets/local/NEW_DOS_PATH
```

마지막 두 검사는 선택적 Unicorn2.1.4 또는 DOSBox/Xvfb/xdotool/PyAV 환경을 사용하며 CI 의존성이 아닙니다. 현재 기록된 원본/WASM은21곡·18효과음 각각3초22050Hz, C는각2초44100Hz입니다. 전곡 전체/효과음 종료와 모든 loop의 레지스터 검증은 #74의 별도 근거입니다. PCM 모든 조합/전체 길이·실제 DOS 믹서 출력/물리 OPL·듣기·네이티브/Web 장치 활성화는 이번 검증 범위에 포함하지 않습니다.
