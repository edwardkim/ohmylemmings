# #165 BGM 감쇠 원인 조사

이슈: https://github.com/edwardkim/ohmylemmings/issues/165

현재: 공통 BGM 제품 수정·7개 플랫폼 CI·실제 병합·v0.2.0 배포를 완료했다. 2026-10-24 메인테이너가 BGM 정상 재생을 확인하고 이슈 종료를 지시하여 실제 청취 인수를 완료했다. 아래 미수정/대기·OPEN 표기는 당시 기록이다.

메인테이너가 Web·Native·다운로드 패키지에서 효과음은 들리지만 BGM은 들리지 않는다고 보고했다. 기준 main `9e734a7b26542ad5ec1e50ed2d1742e9001597f9`에서 공통 PCM 경로의 음표 재시작 손실을 재현했다. 이번 변경은 조사 기록이며 제품 수정·수정 후 청취 판정은 아직 하지 않았다. #163 팩 정리는 별도로 완료했다.

## 확인된 원인

`lemmings-audio::AdlibPcm::render`는 한 드라이버 호출의 모든 OPL register 쓰기를 다음 샘플 생성 전에 연속으로 전달한다. `lemmings-opl::Opl2::write_reg`는 이를 `Chip::write_reg`에 즉시 전달한다. 음악 드라이버의 새 음표는 같은 호출에서 key-off 후 key-on을 쓴다. OPL envelope의 key 상태가 샘플 처리 전에 다시 켜지므로 종료 상태를 관찰하지 못하고 다음 음표의 attack으로 재시작하지 않는다. 결과적으로 기존 음의 envelope가 감쇠하며 BGM이 거의 들리지 않게 된다. 효과음은 독립 슬롯/별도 종료 구간 때문에 계속 들릴 수 있다.

이는 세 플랫폼에서 공유하는 합성 경로다. 파일 누락이나 브라우저 재생 권한으로 설명할 필요 없이 장치 없는 실행에서도 재현된다. 원본 곡 선택·시작 명령은 전달되며 기본 일반 곡은0기준5번이다. 실제 통합 trace에서도 시작 명령과 초기 음악 파형을 확인했다. 사용자의 모든 장치에서 직접 청취했다고 주장하지 않는다.

## 인과 대조

원본 ADLIB 이미지 SHA-256 `ad4f33eb383c60b5a67945987f27580fc0540e38558d4c158efe1a672db5f490`의 곡5를60초/22050Hz로 합성했다. Rust 현행 출력과 별도 프로세스의 고정 C Nuked-OPL3 즉시 쓰기 출력이 바이트 단위로 같았다. 동일한 register/event 시퀀스에서 C harness의 `OPL3_WriteReg` 호출만 `OPL3_WriteRegBuffered`로 바꾸면 지속 출력이 복원됐다. 게임 틱/곡/음량/드라이버 명령을 변경하지 않았다. RMS는 DC를 뺀 AC 값이며 물리 음량 측정이 아니다.

| 재생 시점 | 현행 즉시 쓰기 AC RMS | 버퍼 쓰기 AC RMS |
|---|---:|---:|
| 0초 | 2065.86 | 2937.31 |
| 5초 | 658.88 | 2843.91 |
| 10초 | 181.42 | 2743.09 |
| 15초 | 83.94 | 2913.04 |
| 30초 | 83.95 | 2860.28 |
| 40초 | 83.88 | 2683.86 |
| 55초 | 83.89 | 2744.66 |

원본 없는 단일 음표10회 반복으로도 같은 원인을 분리했다.22050Hz·0.5초 간격으로 같은 timestamp에 B0 key-off(0x11)/key-on(0x31)을 쓴다. 초기 register는20/23=01,40=3f,43=00,60/63=f4,80/83=44,c0=00,a0=98이다. 매 음표 첫0.1초의 AC RMS:

- 현행 즉시 쓰기: `2483.62, 512.7, 105.7, 21.61, 4.47, 1.05, 0.5, 0.5, 0.5, 0.5`
- 버퍼 쓰기: `2474.25, 2490.57, 2498.81, 2507.33, 2461.53, 2468.83, 2476.98, 2485.64, 2492.14, 2500.56`

동일 음표를 반복해도 현행은2483→0.5로 감쇠하고 버퍼 쓰기는약2500을 유지한다. 따라서 음악 편곡의 쉼표·곡 자체 음량이 아닌 재시작 전달 손실임을 확인했다.

## 기존 검증이 놓친 이유와 수정 범위

기존 C 비교 harness `tools/audio/nuked_reference.c` 역시 즉시 `OPL3_WriteReg`를 사용한다. 이 비교는 두 구현의 동일성을 검증했지만 잘못된 전달 방식도 함께 재현했다. 원본 곡 C 비교는2초, 일반 PCM 검사는3초와 비영 샘플 존재를 중심으로 했으므로 이후 감쇠/음표별 재시작 손실을 검출하지 못했다. 장치 smoke도 효과음만으로 비영 출력을 통과할 수 있었다. 이전 PASS 기록은 당시 검사 범위로 유지하며 BGM 지속 재생 보증으로 해석하지 않는다.

수정은 공통 OPL 쓰기에 bounded buffered delivery를 적용해 register 간 처리 시간을 보존해야 한다. upstream 버퍼의 내부 샘플 시계·용량·overflow/clone/continuation 동작을 먼저 확인한다. 게임 틱이나 전체 음량을 바꾸는 방식으로 우회하지 않는다. 반복 음표·긴 음악 전용 구간·효과음 후 음악 지속·chunk/rate/Native/WASM 동일성과 고정 C buffered 기준을 회귀 검사에 추가한 후 PCM golden/출처 pin을 근거와 함께 갱신해야 한다. 아직 이 수정은 적용하지 않았다.

재현 산출물은 별도 worktree의 ignored `assets/local/issue-165-review/`에 보관했다: `direct-pcm.json`, `integrated/trace.json`, `music5-60s.json`, `music5-trace.jsonl`, `buffer-comparison.json`, `synthetic-retrigger.json`, `compare-buffered.py`. 원본 PCM/trace/바이너리는 커밋하지 않는다. 합성 비교는 `tools/audio/nuked_reference.c`와 고정 `tools/audio/reference/opl3.c`를 C99/O2로 별도 빌드하고 함수 호출 하나만 바꾸어 재현한다.

상태: 원인 조사 완료, 제품 수정·플랫폼 재검증·메인테이너 청취는 후속으로 #165 OPEN 유지.

## 제품 수정·로컬 검증 (2026-10-24)

상위 #137은 #187/PR188·189로 마감했고, 메인테이너의 다음 진행 승인으로 main `c1bbb3c4154766bf0f7aff884cfaa8b9f7849c67`에서 수정을 재개했다. 앞선 ‘미수정/대기’ 표기는 당시 조사 기록이다.

`Opl2::write_reg`가 기존 upstream `Chip::write_reg_buffered`를 선택하도록 변경했다. 고정1024항목 ring과 원래 overflow 정책을 그대로 사용하며 쓰기 사이의2개 native OPL 샘플을 보존한다. CPU/장치 sample rate에 맞춘 임의 sleep·게임 틱·전체 음량 조절로 우회하지 않는다. 정수 합성 core·리샘플러·AdLib VM·음악 선택/효과음 이벤트·게임 규칙은 바꾸지 않았다. Clone과 PCM continuation에 대기 register도 그대로 남는다. LGPL 수정 내용은 library README에 명시했고 원본 core/라이선스/고지는 유지한다.

- 반복10음표가 수정 전8000Hz의 두 번째 음표에서 실패(AC 분산6,141,201→258,693)하고, 수정 후8k/22.05k/44.1k/48k/49.716k/96k/192kHz에서 통과했다.1023/1024/1025/4096연속 쓰기·복제·1프레임 이후 continuation 복원을 검증했다.
- 고정 C core 자체는 그대로 두고 비교 harness만 `OPL3_WriteRegBuffered`로 바꿨다. 기존24합성/63원본 포함 사례가 바이트 단위로 같고, 새11반복/overflow 사례도 C와 일치했다. 기존 immediate-write C 동일성은 과거 전달 방식의 기록이다.
- 원본21곡 각각60초와 음악5+효과음0/17 이후 음악 지속60초의22사례가 C와 바이트 단위로 일치했다. 기본 곡의0/5/15/30/40/55초 AC RMS는2937.31/2843.91/2913.04/2860.28/2683.86/2744.66으로 유지됐다. 이는 스피커 음량이나 하드웨어 OPL 동일성 판정이 아니다.
- 실제 WASM은11register 사례+기본 곡/효과음 혼합60초의13사례/4,993,330프레임을 통과했다. 기존24 PCM/72Native 전달·24WASM, 게임98사례/97효과음 분기/170음악 조합, output6사례/249,492프레임·실제 Web Audio running/비영/예약 node·원본 대응 소스/라이선스 delivery와 SDL dummy queue도 통과했다.
- Rust715개·Python311개·format/Clippy를 통과했다. 변경된 합성 PCM golden은 C 대조 후에만 갱신했으며 게임 이벤트 trace SHA `7fd0d4d1c324d868387b77e03c65b87e4d0dea0d22113527eb033cadb854b64c`와 상태/RGB/지형/객체 해시는 그대로다. 원본 PCM/곡/바이너리/녹음은 ignored 경로에만 둔다.

CI에는 자산 없는 반복/overflow 검사, Linux 원본21곡/효과음 혼합60초, Web 원본 기본/혼합60초 WASM을 추가한다. 이전 #187 Intel Mac 설치 검사가571초로10분 한도에 근접했으므로 해당 job만15분으로 조정했다. 기능/오류 판정은 유지하며 성능 #113 NOT READY를 바꾸지 않는다.

실행 중인 기존8080 서버의 `/preview-165/`에 수정 HD 게임, `/preview-165/listen.html`에 같은 기본 곡의 수정 전후60초 비교를 준비했다. user CDP19222/기존4탭과 `/preview-172/`는 이동·교체하지 않았다. 실제 사람이 BGM/효과음을 듣는 판정은 디지털 결과와 구분해 요청했다. 아직 플랫폼 CI·self-review·실제 병합과 청취 응답 확인 전이므로 #165는 OPEN이다. 공개 v0.1.0/Pages는 바꾸지 않는다.

현재 정량 기록: [BGM 전달 검증](../../assets/manifests/bgm_delivery.json). 새 원시 자료는 ignored `assets/local/issue-165-fix/`에 보관한다. 원본 VM/register 일정과 전체 게임/화면은 유지하므로 이 수정에120개 스테이지의 재완주나 새60fps 측정을 포함하지 않는다.

## 최종 CI·병합·버전 반영

BGM [PR #190](https://github.com/edwardkim/ohmylemmings/pull/190) 실제 merge `9a0ab3bb4e1c4607ee3500445e1cc3ceedc911f7`. head `95c2e221acff5d62f530e312231b8847db22b484`의 [CI39982379823](https://github.com/edwardkim/ohmylemmings/actions/runs/39982379823)에서7개 모두 SUCCESS. CI checkout `6c571f10202c3897f09d965a4df6a26567ef5b62`/실제 merge tree `25a01ea4e47454f856ce457a1a9b4b921e8bffb0`가 같다.

버전 [PR #192](https://github.com/edwardkim/ohmylemmings/pull/192) 실제 merge `c7da0071cd8d056bea4bad83fa912b70be407e60`. head `232019603a36d487149c1cd9b8f93732c3111b15`의 [CI39984622798](https://github.com/edwardkim/ohmylemmings/actions/runs/39984622798)에서7개 모두 SUCCESS. CI checkout `d405a53a140de2c6ded78ea5c1c1a0be8a92a231`/실제 merge tree `daac94826be9b8fd15c084eb364dd06d00635c4e`가 같다.

Linux의 원본21곡/효과음 혼합60초, Web의 원본 기본 곡/효과음 혼합60초 WASM과 기존 전체 회귀·4플랫폼 새 설치/원본/HD/오디오 probe를 통과했다. Windows hosted 환경은 Server2022, Mac은 arm64/Intel에서 같은 universal 앱을 검증했다.

메인테이너의 실제 청취 응답은 아직 없다. 디지털/장치 검증 및 제품 수정은 완료했으며 #165는 청취 확인 대기로 OPEN을 유지한다. 버전0.2.0 선택을 청취 정상 판정으로 간주하지 않았다.

취소한 중복 push/main 실행은 PASS에 포함하지 않는다. 완료 문서는 별도 Markdown 변경으로 검토하고, 실제 문서 merge 및 root main0.2.0/Native/Web 재빌드·원본13사례 C/Native/WASM 재확인·preview-172/165 보존 증거는 해당 이슈와 ignored root-ready.json에 고정한다. 공개0.1.0/Pages는 유지하며0.2.0 태그/공개는 별도다.

## 메인테이너 청취 인수 완료 (2026-10-24)

v0.2.0 공개 후 메인테이너가 “bgm 도 잘 나옵니다. 이슈를 close 하면 되겠습니다.”라고 정상 재생을 확인하고 종료를 지시했다. 기존 제품 수정·회귀·플랫폼 CI·병합·배포 검증에 실제 청취 인수를 더해 #165를 완료한다. 이 응답은 메인테이너의 재생 확인으로 기록하며, 새로 모든 기기·곡을 청취했다고 확대하지 않는다. 문서 내 청취 대기/OPEN 문구는 이번 응답 전의 기록이다.
