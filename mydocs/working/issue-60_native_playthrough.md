# G08-07 네이티브 통합 완주

이슈: [#60](https://github.com/edwardkim/ohmylemmings/issues/60) · 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1)

## 목표와 상태

main `091cf2a` 동기화·이슈 번호/URL을 확인하고 분기했습니다. #48~#58의 객체/승패/결과/패널/포인터를 기준 Fun01의 실제 성공·실패·재시작·다음 레벨로 통합 검증합니다. 원본 없는 CI, 고정 원문·DOS 관측과 독립 입력 재생을 포함합니다.

## 검증 상태와 다음 행동

실패 테스트부터 합성 통합 Rust1개·Python8개를 추가했습니다. 실제 SDL 합성663/원본1699상태와 각각12독립 재생, 원본 네 시도의 고정56함수/표 직접 실행이 일치했습니다. 새 DOS 성공 세 번·실패 한 번의 결과와 성공 결과에서의 레벨 인덱스 선행 증가를 확인했습니다.

[계약](../tech_insights/issue-60_native_playthrough.md)·[보고서](../reports/issue-60_native_playthrough.md)·[증거](../../assets/manifests/native_playthrough.json)를 동기화하고 전체 검사·자체 검토·게시를 진행합니다. 정확한 head의 push/PR CI와 main 병합 CI·전체 로그·트리를 확인한 뒤 #60 종료와 상위 #1 G08 완료를 반영합니다. 게시 결과는 이슈에서 추적합니다.

다음은 G09 웹의 동일 코어 경계를 별도 이슈로 고정하는 작업입니다. 전체 DOS 인증·음향·원본 메뉴/결과 아트워크는 후속입니다.
