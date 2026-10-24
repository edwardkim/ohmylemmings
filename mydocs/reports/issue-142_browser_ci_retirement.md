# #142 과거 Chrome156 CI 타임아웃 추적 중단

이슈: https://github.com/edwardkim/ohmylemmings/issues/142

2026-10-24 메인테이너가 현재 WASM 정상 동작을 근거로 이슈 유지 필요성도 검토하도록 요청했고, **해결 완료 대신 Not planned로 종료하고 재발 시 재개한다는 권고를 수용**했다. 이는 과거 타임아웃의 원인 규명이나 Chrome156 수정 검증을 완료했다는 판정이 아니다.

## 확인한 근거

- [#140 당시 실패 Web job](https://github.com/edwardkim/ohmylemmings/actions/runs/39789406619/job/119861103949): head `355d37c14e675a23af2dd09306b08c9dd49efea6`, Chrome156.0.8078.47 / runner image20261021.345. check_web.py의180초 결과 대기에서 TimeoutError를 한 번 관측했다. 임시 browser.log는 CI 산출물에 보존되지 않았다.
- [동일 head 재실행](https://github.com/edwardkim/ohmylemmings/actions/runs/39789406619/job/119872576356)은 Chrome155.0.8059.39에서 통과했다. 브라우저와 runner 조합이 달라 버전 차이를 원인으로 확정할 수 없다. [#140 기록](issue-140_hd_presentation.md)을 보존한다.
- 최신 [v0.3.1 Web CI](https://github.com/edwardkim/ohmylemmings/actions/runs/40067006448/job/120792546172)는 Chrome155.0.8059.39 / image20261012.334에서 동등성·DOM/HD·완주·오디오·패키지 검사를 통과했다. 공개 WASM의 HD Fun01 10구출 Won·진행률/원본 WebAudio/예외0은 [#208](issue-208_native_polish.md)에 기록했다. 메인테이너도 현재 WASM 정상 동작을 확인했다.
- 현재 로컬 실행/사용자 CDP19222의 브라우저는 Chromium152.0.7977.82다. 최신 CI와 현재 플레이를 Chrome156 검증 결과로 확대하지 않는다.
- 소스 확인 시 check_web.py의180초 대기와 기본 임시 디렉터리, 실패 browser.log의 미보존 경로는 남아 있다. Chrome155/156의 고정 환경 비교·원인 구분·진단 개선·수정 회귀라는 원래 완료 기준은 충족하지 않았다.

## 종료 결정과 재개 조건

현재 배포/플레이와 최신 정식 CI가 정상이며 메인테이너가 과거 단발성 CI 진단을 계속 수행할 필요성이 낮다는 권고를 수용했으므로 **Not planned / 추적 중단**으로 종료한다. 원래 완료 기준을 체크하거나 실패 실행을 성공으로 바꾸지 않는다. 기존 문서의 #142 OPEN 문구는 작성 당시 상태다.

현재 소스/CI에서 같은 결과 대기 타임아웃이 재발하거나 브라우저 환경 변경으로 검증/배포가 막히면 이슈를 다시 연다. 그때 정확한 제품 commit·브라우저 전체 버전·runner image/자원 정보와 검사 진행 단계/사례, 브라우저 종료 상태, browser.log·결과 POST 수신 여부를 실패 산출물로 먼저 보존한다. 같은 소스/작업량에서 환경을 고정해 브라우저·runner 부하·검사 도구·제품 오류를 구분한 뒤 필요한 변경을 정한다.

이번 작업은 종료 판단/기록만 수행한다. 게임·WASM·CI 코드/대기 제한/검사 항목/브라우저 버전·공개 릴리즈/태그/버전 노트는 변경하지 않는다. #113의 성능/구간 메모리 NOT READY와 고성능 호스트 이월은 별도다.
