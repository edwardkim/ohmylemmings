# SDL 표시 주기 보정

담당 [#107](https://github.com/edwardkim/ohmylemmings/issues/107), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1), 선행 [#106](../reports/issue-106_performance.md).

기준 main `213d50c761db515f71aa51231f9a2357b70e7083`. #107 OPEN·번호/URL을 확인했다. 사용자 다음 절차 승인에 따라 self-review로 진행한다.

## 계획

- present 뒤16ms 대기가 처리 시간을 더하는 문제를 실패 테스트로 재현한다.
- 네이티브 호스트 전용60Hz 절대 주기를 추가한다. 강제 초기/결과/캡처 프레임과 fast 모드는 유지하며 늦어진 슬롯은 반복 렌더하지 않는다. 숨김/복귀·레벨 교체는 표시 시계만 재설정한다.
- 고정 simulation/리소스/렌더/PCM 엔진을 유지한다. 같은 머신·Mayhem22 입력·표시/오디오 ON60초3쌍을 baseline/candidate 순서로 실행한다.
- 상태/맵/RGB/PCM 동일성과 틱·프레임 지연/queue/underrun·메모리를 기록한다. 원시/환경/실행 파일을 ignored에 보존한다.
- self-review·한글 PR·정확한 CI·병합/main·소스 배포·문서/상위를 갱신한다. G11 전체/G12 마감은 후속이다.

## 로컬 결과

- 절대 주기4테스트/SDL38개·fmt/clippy 통과. 실제3쌍에서 기존52.645~52.678fps→수정60.010~60.011fps, gap p95약22.5→17.4~17.5ms.
- 모든6회841틱/동일 상태·underrun0, 수정3회 scheduled 마감 초과/슬롯 생략0. 원본/합성451파일·40PCM 동일성 통과.
- 공개 display_cadence.json·보고서/합성 readback으로 고정한다. 정확한 게시/CI/병합/main·상위 마감은 #107 최종 기록을 따른다.
