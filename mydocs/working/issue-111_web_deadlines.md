# Web 표시 마감·메모리 보완

담당 [#111](https://github.com/edwardkim/ohmylemmings/issues/111), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1), 선행 [#110](../reports/issue-110_g11_closure.md). 기준 main `c4e935f5000cf79c3e4d8c0c404fa409d07f9cff`.

이슈 OPEN·번호/URL 확인, 원격 main 동기화, 사용자 다음 절차 승인에 따라 self-review로 진행한다.

- 기존 #106 원시·실행/소스/입력을 보존한다. 새 계측은 rAF timestamp·진입/완료와60Hz 계약 예산을 저장하며 물리 scanout이나 브라우저 보장 마감으로 부르지 않는다.
- 먼저 계측만 보완해 기존 게임의 비용/메모리를 관측한다. 틱·RGB 생성/Canvas·state/오디오·관측 비용과 미분류 host 비용을 구분한다. 필요할 때만 제품을 보정하고 전후60초3쌍·동일성을 검증한다.
- 메모리는 시간 표본 peak·WASM committed·관측 버퍼와 프로세스 범위를 구분한다. JS sampled peak는 정확한 연속 peak가 아니며 기존 끝값을 peak로 이름만 바꾸지 않는다.
- 구간마다 표시/오디오 ON·같은 Mayhem22 입력·숨김 없음, 실제 상태/맵/RGB/PCM·hidden/restart 정책 보존을 확인한다. 전체120개를 무조건 반복하지 않는다.
- 실패 테스트→구현, 원시 pins/범위·마감 판정·재현/보고서/색인/일지, self-review·한글 PR·정확한 CI/병합/main과 소스 전달을 기록한다. 요구 미충족은 남겨 G11/G12를 선행 완료하지 않는다.

## 결과·후속

[보고서](../reports/issue-111_web_deadlines.md)와 [기계 증거](../../assets/manifests/web_deadlines.json)에 메모리3회·단독 무조회1회·정식 무조회3회 및 모든 초과를 보존했다. 게임 실행 코드는 변경하지 않았다. 무조회 완료 초과2/0/1, 초기/추가 draw와 dispatch 원인 미확정은 #113 OPEN으로 추적한다. #111 계측 완료와 G11 합격을 구분하며 G11/G12는 미완료다. 정확한 CI/병합/main·소스 전달은 #111 최종 기록을 따른다.
