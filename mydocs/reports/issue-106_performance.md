# G11-06 성능 기준 측정

담당 [#106](https://github.com/edwardkim/ohmylemmings/issues/106), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1), 계약 [#85](../plans/issue-85_compatibility_validation.md#성능-측정-계약), 표시 주기 후속 [#107](https://github.com/edwardkim/ohmylemmings/issues/107).

## 판정과 환경

기준 main `973fa7868fd436844205435857e89e205b2cf908`에서 첫 성능 기준값을 측정했다. **측정 완료·표시 목표 미달 후속 필요**다. 이전 측정 기준이 없으므로 회귀 판정은 하지 않는다. SDL의 약53.2fps를 #107에서 보정·재측정하며 G11-06 전체 통과, G11 마감과 G12 완료는 선언하지 않는다.

Intel N100/4코어/약15.4GiB, Linux7.2.5/Wayland, 1920×1080/60Hz/scale1, 전원 profile performance·CPU governor powersave다. Rust1.93.1 release 기본 flags, SDL2.32.72 software renderer의960×600 창, Chromium152의320×200 Canvas/DPR1을 사용했다. 게임 논리 화면은320×200 RGB24, DOS pacing `2190197/156250`Hz다. 기준17Hz와 게임 내부 초를 구분한다. SDL은44100Hz mono/quantum512, Web은48000Hz mono/scheduled chunk512이며 실제 하드웨어 quantum은 Web API에서 노출하지 않는다.

공개 [performance_baseline.json](../../assets/manifests/performance_baseline.json)에 원시 시간·초기화·연산 수·통계·환경·입력/원본/바이너리 해시와 실제 호스트 분포를 고정했다. ignored 원시 JSON/PCM/맵/실행 파일은 `assets/local/issue-106-review/`에 있다. Rust Instant와 performance.now를 사용하며 알고리즘은 준비3회 뒤30회, p95는 정렬30개 중29번째다. 해시/덤프/다운로드는 측정 창 밖이다.

## 알고리즘 결과

아래 범위는 부하별 **median의 최솟값~최댓값**이다. 각30표본의 p95/max/처리량은 manifest에 있다. Native184개와 Web196개, 총380개 측정 행이다.

| 구간 | Native | WASM/JS | 범위 |
|---|---:|---:|---|
| 메모리 자산 로딩 |1.840~5.954ms|2.900~7.900ms|원본120레벨 각각33회|
| 논리 틱 |0.872~0.922ms/tick|0.915~1.000ms/tick|12고정 입력 구간|
| CPU 화면 |0.016~0.438ms/frame|0.111~0.606ms/frame|12고정 최종 장면,32프레임씩|
| Canvas 전송 |별도 SDL 아래|0.395~0.466ms/frame|RGB 변환+putImageData, scanout 제외|
| PCM |481~572ns/sample|621~735ns/sample|21곡·18효과·혼합1,각44100샘플|

메모리 Native 로딩은 owned AssetSource 복사와 Assets 생성을, WASM은 `load_original_rank`의 세션 교체까지 포함한다. 틱도 Native의 LevelRun/입력 dispatch와 WebGame/JS/panel/pointer 경계를 포함하는 차이가 있다. 이 수치만으로 언어·호스트 자체의 성능 우열을 단정하지 않는다. PCM 혼합은 music0과18효과 요청 큐이며 모든 효과를 동시에 계속 합성한다는 뜻은 아니다. 곡 전체 길이도 측정하지 않았다.

5일반 스타일·4특수 장면·기술/지형 변경과 Fun01 실제 완주를 선정했다. 긴 부하는500..1200경계로 고정했고 Mayhem22는80개체에 도달한다. 모든 가능한 입력의 최악 부하를 증명하지 않는다. 12개 각각33회에서 상태·지형·벽돌·객체·전체 RGB를 고정한 기존 증거와 대조했고40개 PCM도 Native/WASM SHA가 일치했다. 결과 화면에서 멈춘 틱을 세지 않는다. 예비6구간의 결과 화면/Next 설정은 보존하고 올바른 구간으로 다시 측정한 행만 교체했다.

파일 읽기는50파일/831245bytes를 신규 프로세스33개에서 측정했다. 준비3회 뒤 open/read+목록/owned bank median **0.790ms**다. 프로세스 시작 포함 벽시계는 별도다. 앞선 실행으로 OS cache가 따뜻하며 cache를 통제하지 않았으므로 cold-cache라고 부르지 않는다. 테스트와 겹친 예비 읽기는 제외 사유와 원시를 보존하고 조용한 구간에서 보충했다.

## 실제 표시·오디오

Mayhem22 같은 입력으로 **표시·오디오 ON 60초×3**을 각 호스트의 실제 production 경로에서 실행했다. 기록은 메모리에 모으고 종료 후 썼다.

| 호스트 | 프레임 수 3회 | 평균 FPS 3회 | 프레임 간격 p95 3회 | 틱 수 | 관측 underrun |
|---|---|---|---|---|---|
| SDL |3191/3195/3194|53.178/53.246/53.229|21.979/21.947/21.918ms|각841|각0|
| Web |3603/3600/3601|60.035/59.982/60.000|22.300/22.300/22.700ms|각841|각0|

DOS 틱 예산은약71.341ms, 기준17Hz는58.824ms,60Hz 표시는16.667ms, SDL audio512/44100은11.610ms다. 실제 SDL 최대 관측 tick 비용은약1.20ms 이하였고 CPU+전송+present 평균은약2.1ms다. 연산 비용보다 `last_render`를 present 뒤로 갱신하고16ms 기다리는 주기가 SDL 평균 FPS 차이의 유력한 원인이다. **원인 확정은 #107의 같은 기계 전후 비교에서 한다.** 현재 pacing/renderer/게임 규칙은 수정하지 않았다.

Web은 평균60fps지만 모든 프레임이16.667ms 이내라는 뜻은 아니다. 두 refresh(33.333ms)를 넘은 간격은0/2/17회, 최대24.0/52.9/38.8ms였다. raw 분포를 그대로 남긴다. 16.667ms 단순 초과 수는 rAF/시계 양자화와 스케줄링도 포함하므로 물리적 화면 손실 횟수와 같지 않다. 엄격한 모든 마감 통과로 판정하지 않는다.

Web 세 번째 예비 실행은59.3917초에 탭이 숨겨져832틱/3552프레임·오디오 suspended가 됐다. 원시를 보존하고 표시 조건으로 한 번만 보충했다. 별도 숨김1초에서는 control counter 증가0·오디오 suspended/sources0, 복귀 시 running을 확인했다. 첫 lifecycle 검사 오류는 async Promise를 기다리지 않은 측정 도구 문제였으며 수정 후 lifecycle만 다시 실행했다.

Native live 최종 상태는 이전841경계 SHA와 일치했고 Web live는 상태/세 맵/RGB 모두 일치했다. Native live에서 맵/RGB를 덤프했다고 주장하지 않는다. PCM/algorithm 비교와 별도 범위다. queue에서 관측한 underrun0은 물리 DAC 인증이 아니다. 메모리는 Native 프로세스 생애 VmHWM(자산 초기화/기록 버퍼 포함), WASM committed linear bytes, 브라우저 종료 시점의 대략적 JS heap이다. 브라우저 전용 interval peak는 확보하지 못했으며 해당 값으로 대신하지 않는다.

## CI·캡처 비용과 변경 영향

같은 제품 트리의 #104 push/PR/main CI3개(`39595613485`, `39595621030`, `39597260559`)를 재사용했다. clean Ubuntu24.04/no cache, 병렬 job의 전체 경과는711/697/689초, Rust 검사 step661/660/655초, Web 검사192/202/201초다. 초 단위 로그이며3개로 p95를 계산하지 않는다. 전체 job을 합해 실제 벽시계로 부르지 않는다.

현재 CI는 GNU make shell adapter로 원래 명령 각각의 벽시계/child CPU·peak RSS/종료 코드를 기록한다. 컴파일+링크 합계이며 linker만의 프로파일은 아니다. 합성64틱 비교에서 캡처 off52.2ms, on138.7ms·46파일/15929809bytes였다. 추가 렌더/SDL readback/직렬화/파일 쓰기를 모두 포함하는 단일 진단 표본이므로 순수 디스크 속도·p95·게임 FPS로 부르지 않는다.

측정 추가는 SDL의 opt-in `--performance`, 독립 Rust example/브라우저 도구, 검증용 PCM WASM export, CI 시간 기록뿐이다. core/resources/game/audio 엔진 및 production WebGame WASM 바이트는 기준과 동일하다. 기본 SDL 경로의 회귀는 현재 CI로 검사하며 이미 완료한120개 전체 장시간 리플레이를 반복하지 않는다. #104 감사에는 변경 전 소스 pin과 진단 추가의 영향 범위를 명시적으로 보존한다. 기존 ELF도 같은 경로/기준 소스로 재빌드해 이전 SHA와 정확히 일치시켜 ignored 고정 경로로 옮겼다.

## 검증·재현

Python 통계/잘못된 표본/입력 범위/CI 종료 코드·make 연동7개, Rust example2개, PCM adapter2개와 SDL 옵션/출력 경로 검사를 수행했다. 원본 없는 smoke6행은 각3+30표본과 캡처 유무를 검증한다. fmt/clippy와 변경 패키지 검사를 통과했다. 정확한 게시 head·self-review·push/PR/main CI 로그와 명령별 병목·병합은 [#106](https://github.com/edwardkim/ohmylemmings/issues/106)의 최종 기록을 따른다.

[도구 재현 절차](../../tools/performance/README.md)를 따른다. 다운로드된 원본·PCM·영상·실행 파일은 커밋하지 않는다. 관측한 목표 미달의 보정은 #107에서 별도 전후30쌍/실제 호스트3쌍으로 진행한다.
