# Web rAF 지연·추가 그리기 진단

현재 상태: 2026-10-25 메인테이너 지시로 #113 추적을 종료했다(Not planned). [종료 근거](../reports/issue-113_closure.md)를 따른다. 아래 OPEN/잔여/NOT READY 표현은 종료 전 기록이다.

담당 [#113](https://github.com/edwardkim/ohmylemmings/issues/113), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1), 선행 [#111 보고서](../reports/issue-111_web_deadlines.md).

## 착수

- 2026-10-22 Asia/Seoul: 이슈 OPEN·번호/URL과 로컬/원격 main `3426212be0015d145b1e2f3ebdd1c726c6a1e250` 일치를 확인했다. `issue-113-web-dispatch`에서 착수한다.
- 기존 #111 원시 7개 구간·모든 초기/구간 내부 마감 초과·실행 소스/입력 pins를 보존한다. 게임 코드 변경 없이 관측 이벤트의 원인을 먼저 확인한다.
- 기존 정상 앱 observer는 `input`의 명령/인수와 `reset` 사유를 제공한다. #111 live observer는 이 상세를 저장하지 않았다. 새 진단은 이 입력과 RAF 밖 `present`의 대응을 기록한다.
- 정상 앱은 레벨 교체 시 `replaced()`의 `draw()`와 이를 감싼 `attempt()`의 `draw()`를 모두 호출한다. 이것이 관측한 초기 2회에 해당하는지는 새 이벤트 기록으로 확인한다. 다른 추가 그리기의 원인은 아직 미확정이다.
- Chromium CDP 19222 / HTTP 8080 연결 정상. 계측 보완과 실제 60초 성능 통과를 구분하고, 기존 마감 기준이나 초과 합계를 바꾸지 않는다.

## 다음 절차

1. 별도 ignored staging에 입력/강제 그리기와 rAF 상관 관측을 추가하고 원본·소스·입력 해시를 고정한다.
2. 표시/오디오 ON, heap 반복 조회 없는 동일 Mayhem22 입력으로 한 진단 구간을 관측한다. 진단 실행을 정식 3회 성능 통과로 계산하지 않는다.
3. 필요한 trace/대조와 준비 경계 확인 후에만 제품 보정 여부를 결정한다. 제품 수정이면 TDD·같은 호스트 60초 3쌍·동일성을 검증한다.
4. 증거·검증·self-review·PR/CI를 동기화한다. 원인 진단 및 합격 조건을 충족하기 전까지 #113/G11/G12는 미완료다.

## 첫 진단 구간

- 별도 `assets/local/web/issue113-diagnostic/`에 #111과 같은 제품 JS/WASM을 복사했다. 개발용 live observer에만 고정 크기 이벤트 버퍼 1,232,000바이트를 추가했다. 측정 소스/입력/실행 pins와 원시는 ignored `assets/local/issue-113-review/`에 보존한다. 제품 실행 코드 수정은 없다.
- heap 반복 조회 없음, 표시/오디오 ON Mayhem22 60.0219초: 841틱, rAF 콜백3,602회/전체 present3,604회. 기존 상태·지형·다리·객체·RGB 5개 SHA 전부 일치, visibility0·관측 underrun0, 별도 hidden/resume PASS.
- 콜백 최대11.9ms·실행 예산 초과0회. 완료 마감 초과는 **1회이며 제외하지 않는다**. 첫 콜백 timestamp -4.1ms, 진입14.6ms, 완료16.5ms로 계약 마감 대비3.9333ms 초과다. 이 구간 내부의 다른 초과는0회지만 한 진단 구간만으로 기존 초과가 해결됐다고 판정하지 않는다.
- RAF 밖 present2회는 모두 초기 step0, reset→Release→present11.0ms→present13.1ms 순서다. 정상 앱의 `replaced()`/`attempt()` 이중 그리기 경로와 일치한다. 새 구간에는 다른 실제 사용자 입력이 없었다. 기존 #111의 추가21/4회 원인을 이 결과만으로 확정할 수 없다.
- `python3 tools/web_deadlines.py --local`로 기존 7개 원시·공개 수치·소스/원본 pins가 유효함을 재확인했다. 기존 무조회3회 초과2/0/1과 G11/G12 미완료 판정을 유지한다.
- 다음은 최초 rAF 예약/초기화 경계와 구간 내부 지연을 분리하는 trace 및 통제된 입력 대조다. 정식3회/제품 보정/성능 합격/PR 게시·병합은 아직 수행하지 않았다.

## Trace·보정 진행

- 같은 기준 앱의 trace60초 구간은 완료 마감 초과69회를 보존했다. trace 수집/추가 관측이 있는 진단 profile이며 무조회 정식 측정과 합산하거나 대체하지 않는다. `dataLossOccurred=false`, 약210MiB 원시와 renderer 부분을 로컬에 보존했다.
- frame/pipeline ID를 대조한 사례에서 BeginImplFrame 직후 바로 main으로 전달되지 않았다. 약13~15ms 후 compositor ACK가 도착하고, 같은 compositor thread의 SendBeginMainFrame 및 renderer 실행이 이어진다. 긴 게임 콜백이나 GC만으로 지연을 단정하지 않는다. 브라우저의 이전 프레임 ACK 대기 경로를 확인했지만 ACK의 GPU/OS 내부 원인이나 기존 untraced 구간의 원인까지 확정한 것은 아니다.
- 별도5초 통제 입력은 trusted CDP pointer move10회→입력 관측10회→추가 RAF 밖 present10회(초기2회 포함 총12회)다. 기존 #111에서 저장하지 않은 실제 입력을 복원했다고 주장하지 않는다.
- 제품 보정: 틱/입력 변화가 없는 rAF에서는 기존 픽셀을 유지하고 오디오 service는 계속 호출한다. 입력의 동기 draw는 유지하고, release/lost-capture처럼 동기 draw가 없는 입력도 dirty 표시로 다음 rAF에 반영한다. `replaced()`가 이미 그린 경우 `attempt()`가 중복해서 그리지 않는다. Rust/시뮬레이션/자원/픽셀 생성/PCM 엔진은 불변이다.
- 실제 앱을 제어하는 회귀: 변화 없는 rAF에서 RGB 재생성 실패(red)→보정 green. 포커스 해제 뒤 no-tick 갱신 실패(red)→dirty 표시 보정 green. 준비 staging의 measure.js 누락 및 초 단위 HTTP 캐시로 생긴 실패도 별도 로그로 보존했고, 검증 도구는 cache-disabled 새 탐색으로 최종 소스를 읽는다.
- 첫 후보의60초3쌍은 baseline 초과0/12/1, 후보0/0/0·모든841틱/기존5해시/underrun0이었다. 이후 dirty 경계를 보정했으므로 이 묶음은 pilot로 보존하고 최종 후보의 새3쌍을 수행한다. 최종 소스라고 소급하지 않는다.
- 원본 포함 actual WASM/native 첫 대조는43사례×2·14,980상태/8,617전체RGB/Canvas를 통과했다. 최종 dirty 보정과 새 선택적 진단 도구의 대조/실행 확인은 별도 진행 중이다.

## 현재 상태: 성능 재검증 이월 / 사용자 승인

2026-10-22 사용자는 현재 저사양 장비의 60fps 조정을 중단하고 더 성능이 좋은 환경에서 재검증하며 다음 단계로 진행하도록 승인했다. 채택 최종3쌍은 기준1/40/84·보정0/1/0 초과이며 모든841틱/5 SHA·관측 underrun0이다. 첫 콜백1건을 제외하지 않는다. 기능 검증된 cache/dirty 보정과 진단 도구만 채택하고, 이후 시작 프레임/종료 window 실험은 ignored 증거에 보존했다. 채택 제품 소스는 `issue113-final-after/app.js`와 일치한다.

현재 장비의 추가 성능 조정·60초 반복은 중단했다. G11 성능 판정은 미완료다. #113은 더 좋은 호스트 재검증을 위해 OPEN을 유지한다. G12 준비는 [#115](issue-115_release_preparation.md)의 사용자 승인 예외로 진행한다. [보고서](../reports/issue-113_web_dispatch.md)와 기계 기록이 현재 채택 범위다.
