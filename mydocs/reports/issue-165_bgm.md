# #165 BGM 감쇠 원인 조사

이슈: https://github.com/edwardkim/ohmylemmings/issues/165

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
