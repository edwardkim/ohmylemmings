# Web 표시 완료 마감·구간 메모리 관측

담당 [#111](https://github.com/edwardkim/ohmylemmings/issues/111), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1), 선행 [#110](issue-110_g11_closure.md), 기준 main `c4e935f5000cf79c3e4d8c0c404fa409d07f9cff`. 2026-10-22(Asia/Seoul), self-review.

**계측 보완·측정 완료, Web 마감/G11/G12 미통과.** 기존 원시에 없던 rAF timestamp·콜백 진입/완료와 구간 메모리 표본을 확보했다. 콜백 실행 비용은 세 번 모두 예산 안이었지만, 호출이 늦게 시작해 완료 마감 초과가 남았다. 원인 진단·필요 보정은 [#113](https://github.com/edwardkim/ohmylemmings/issues/113)에서 추적한다. #111 종료는 관측 보완 완료이며 성능 합격을 뜻하지 않는다.

## 변경과 관측 계약

제품의 Rust/JS 게임·자원·시뮬레이션·렌더러·입력/표시 시계·오디오 코드는 변경하지 않았다. 개발용 live observer와 CDP 실행 도구, 원본 없는 통계/판정 검사와 브라우저 회귀만 추가했다.

- `rAF timestamp`, `performance.now()` 진입/반환을 같은 시계 영역으로 저장한다. 이번 관측 계약은 **완료 <= timestamp + 1000/60ms**다. 시작 지연·실행 비용·완료 마감 초과·timestamp 간격을 따로 집계한다. 이 예산은 브라우저가 보장하는 deadline이나 실제 scanout/drop 수가 아니다. [HTML 표준의 rAF 전달 시각](https://html.spec.whatwg.org/multipage/imagebitmap-and-animations.html#animation-frames)을 기준으로 측정 가능한60Hz 계약을 정의했다.
- frame/state/step/Canvas와 오디오 service를 원래 인수·receiver·반환값·예외를 보존해 감싼다. `state_ms`는 직렬화 함수이며 JSON.parse/DOM/UI 등 나머지 비용은 별도 미분류다. 콜백 밖 Audio service와 render도 전체 구간 원시에 남는다. observer bookkeeping은 일부 직접 측정 범위이며 시계/래퍼/최종 기록의 모든 오버헤드를 정확히 뺀 엔진 CPU 비용으로 부르지 않는다.
- 구간 전에 Float64 저장 공간을 확보한다.60초 구간당 예약4,065,600bytes이며 overflow는 실패다. 표본별 영구 배열 증가는 줄였지만 관측 오버헤드가0인 것은 아니다. 행 변환·최종 상태/맵/RGB 추출·SHA 계산·결과 JSON은 종료 시각과 WASM 끝값을 고정한 뒤 실행한다.
- CDP `Runtime.getHeapUsage`는 해당 **isolate 전체**이며 Runtime/게임만의 전용 크기가 아니다.500ms 표본의 최고값과 간격·조회 왕복 시간을 보존한다. 연속 peak를 관측했다고 주장하지 않는다. [CDP API](https://chromedevtools.github.io/devtools-protocol/tot/Runtime/#method-getHeapUsage)의 실제 Chromium152 protocol 설명도 확인했다.

## 동일 부하 결과

Intel N100/4cores·Linux/Wayland, Chromium152, 논리320×200·표시/오디오 ON·기존 Mayhem22 입력/DOS pacing, 동일 production WASM SHA `4d7ff9519a1347160fefee062d2b70e70396e51e090c4ae0b48e3319345281c2`다. 환경·입력/소스/실행 파일·원시 pins와 공개 수치 벡터는 [web_deadlines.json](../../assets/manifests/web_deadlines.json)에 연결했다. 원본/맵/화면/PCM 바이트는 ignored 경로에 둔다.

|프로파일/회|콜백 수|콜백 최대|콜백 예산 초과|완료 마감 초과|초기 timestamp 경계 초과|RAF 밖 present|
|---|---:|---:|---:|---:|---:|---:|
|메모리 조회1|3600|10.0ms|0|5|0|2|
|메모리 조회2|3600|9.6ms|0|0|0|2|
|메모리 조회3|3597|9.3ms|0|3|0|2|
|무조회 단독 대조|3601|10.3ms|0|1|1|2|
|무조회3회/1|3601|10.2ms|0|2|1|2|
|무조회3회/2|3600|11.1ms|0|0|0|23|
|무조회3회/3|3599|11.0ms|0|1|0|6|

모든7구간이60초 이상·841틱, 기존841경계의 상태/지형/벽돌/객체/전체 RGB **5개 SHA와 일치**, 숨김0·관측 underrun0이다. 이 결과는 실제로 수집한7구간을 모두 보존한 것이며 통과하는 표본만 선택하지 않았다. 표의 초기 경계는 timestamp가 구간 시작 전인 첫 콜백이다. 초과 합계에서 제외하지 않았다. RAF 밖 present에는 레벨 교체 직후의 두 강제 draw와 추가 관측이 있다. 입력/강제 draw의 구체적 원인은 이번 observer가 기록하지 않아 #113에서 상관 분석한다. 최종 해시 일치를 구간 중 모든 UI 입력/표시 부하가 동일했다는 증거로 확대하지 않는다.

메모리 프로파일에서 조회 왕복 최대는8.067/8.840/9.722ms였다. 왕복에는 CDP·스케줄링도 포함되므로 게임 처리 시간으로 계산하지 않는다. 조회를 끈 단독 대조 뒤, 같은 조건의 무조회3회를 추가했다. 무조회에서도 구간 내부 완료 마감 초과가1/0/1건으로 남았다. 메모리 조회만을 원인으로 확정할 수 없다. 두 프로파일은 무작위 교차3쌍이 아니므로 인과/성능 개선 증명으로 부르지 않는다.

구간 내부 초과의 진입 지연은 주로15.3~15.7ms였다. 이때 남은60Hz 예산이 작아2~9ms 콜백도 마감을 넘었다. RGB p95 약5.1~5.4ms, state p95 약1.0ms 등의 범위를 확보했으나 OS/브라우저 dispatch·입력/긴 task/GC 중 무엇이 원인인지는 확인하지 못했다. 근거 없는 엔진 보정을 추가하지 않았다. 제품 변경이 없어 알고리즘30쌍이나120레벨 전수 플레이는 반복하지 않았다. #113에서 실제 원인과 수정 필요성을 먼저 확인한다.

## 구간 메모리와 범위

|메모리 프로파일 회|JS used 표본 최고|JS capacity 표본 최고|WASM committed 시작→최고|최대 표본 간격|
|---|---:|---:|---|---:|
|1|2,645,184bytes|5,505,024bytes|9,371,648→10,420,224bytes|510.5ms|
|2|3,380,792bytes|8,650,752bytes|10,420,224→10,420,224bytes|510.1ms|
|3|2,693,060bytes|9,175,040bytes|10,420,224→10,420,224bytes|509.7ms|

각120개 표본이며 첫 표본 약9.7~17.8ms, 마지막 약59.58~59.59초다. 표본 최고는 연속 최고값의 하한이며 끝값과 다르다. 별도의 embedder/backing-storage 카운터도 원시와 공개 벡터에 보존했다. WASM/버퍼/heap/외부 저장소에는 포함 관계가 있어 합산하지 않는다. 같은 WASM 인스턴스의 committed 선형 메모리는 줄지 않으므로, 종료 진단 전에 읽은 끝 크기가 이 카운터의 구간 최고다. 물리 resident/실사용/게임 전용 크기와는 다르다.

기록 버퍼·초기 자산 로드의 잔존 용량·이전 구간 결과/GC의 영향이 포함될 수 있다. #106 알고리즘 종료 WASM/근사 JS heap과 이번 live isolate 카운터는 조건이 달라 메모리 회귀로 비교하지 않는다. 실제 hardware quantum·물리 DAC/scanout과 연속 게임 전용 JS peak는 미관측이다.

## 검사·재현

계측 모듈 누락 red→통계4개 green, 판정 함수 누락 red(기존4개 통과)→총5개 green을 확인했다. 짧은5초 시험에서 Canvas 래퍼가 인수를 하나 더 전달해 실패한 기록을 보존했다. 원래 인수를 그대로 전달하도록 수정했고, receiver/인수 수/반환/예외·Canvas3인수 overload·고정 버퍼/overflow 회귀를 추가했다. 수정된5초 sanity와 실제 브라우저 검사를 통과했으며5초 자료는60초 성능 표본으로 계산하지 않는다.

`tools/check_web.py`는 실제 WASM/네이티브33사례×2,11,960상태·6,877프레임/Canvas readback과 기존 DOM/직업/재시작/특수/음향 경계를 통과했다. 기존 원본40PCM 증거는 #106/#107에서 재사용하며 새40개를 실행한 것으로 계산하지 않는다. 오디오/PCM과 게임 실행 소스는 불변이다.

```sh
source assets/local/dev-tools/env.sh
make check-python
python3 tools/web_deadlines.py --local
python3 tools/web_deadlines.py --require-ready
# 기존 performance staging 준비 절차 후 현재 live.js/measure.js를 함께 복사한다.
PERF_HEAP_INTERVAL_MS=500 node tools/performance/cdp.cjs live NEW_MEMORY_DIR new 3
PERF_HEAP_INTERVAL_MS=0 node tools/performance/cdp.cjs live NEW_TIMING_DIR new 3
```

각 출력 폴더는 새 경로여야 한다. 파일 재사용은 lifecycle에만 허용하며 원시 run은 덮어쓰지 않는다. 공개 [numeric observations](../../assets/manifests/web_deadline_observations.jsonl)는 시간 ns 변환(원래 시계의 ns 정밀도 보장 아님)과 메모리 표본만 담는다. 원본 없이 통계/판정을 재계산하며 `--local`은7원시와 공개 변환·비용 요약·원본50개 pin을 추가 확인한다. `evidence_valid:true`, `web_deadline_ready:false`이고 `--require-ready`의 예상 종료코드는1이다.

#110의 역사적 초과0/3/0과 NOT-READY 판정을 덮어쓰지 않았다. 현재 기능 색인만 계측 도구354개로 연결하고 이전 source pins/변경 범위를 source_impact_111에 기록했다. Python 전체200개·COMP/이전 검토/새 감사·motion 및 정확한 게시 push/PR/main 검사 수/checkout·전체 로그·동일 tree·self-review·병합/대응 소스 전달은 [#111 최종 기록](https://github.com/edwardkim/ohmylemmings/issues/111)을 따른다. G11/G12와 상위 #1은 #113 해결과 전체 재검토까지 미완료다.
