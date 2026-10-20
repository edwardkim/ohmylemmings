# G10-03 OPL 합성기·결정론 PCM

이슈: [#76](https://github.com/edwardkim/ohmylemmings/issues/76) · 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1)

main f63465963266a817491578a982418b518aaad2a6 동기화와 이슈 번호/URL 확인 후 issue-76-opl-pcm에서 착수했습니다. 무관한 미추적 디렉터리·HTTP8080/방화벽·CDP19222 loopback은 유지합니다.

- 초기 oplon 후보의 원본 음악 음량 차이를 독립 C 기준에서 발견해 Nuked-OPL3 Rust0.1.0 코어로 교체했습니다. LGPL 전체 고지·소스·수정 patch·재링크 안내를 포함합니다.
- no_std/무할당 정수 OPL2와 PCM: 유리수 시계·mono left i16LE·청크/초기화·전체 요청 오류 원자성·호출 경계 제어를 구현했습니다. Rust/Python 각13개 신규 검사 통과.
- 합성24개와 원본21곡/18효과음의 native+actual WASM63개/3,805,730프레임, 독립 C63개/6,942,328프레임 일치. 원본50파일 해시 전후 동일.
- 원본 IRQ dispatch512회×5모드로 매IRQ AdLib호출 확인. 실제 DOSBox AdLib선택FF·기본VGA19.889초/1390IRQ≈69.887511Hz와 캡처2190197/31250Hz 확인. 잘못 사용한 게임 budget counter를 실제 누적 DS:001c로 수정하고 wrap/reset 회귀를 추가했습니다.
- 전체 make check(Rust563/문서29/Python141) 종료0·self-review 미해결0·출처/해시 고정. 한글 PR·정확한 push/PR/main 네 작업/전체 로그·동일 병합 트리·이슈 마감 진행 중.

[계약](../tech_insights/issue-76_opl_pcm.md) · [결과](../reports/issue-76_opl_pcm.md). 다음 G10-04 게임 이벤트/DOS ID, 이어05장치 출력·06통합 청취이며 G10 전체는 진행 중입니다.
