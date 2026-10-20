# G09-05 브라우저 통합 완주

이슈: [#70](https://github.com/edwardkim/ohmylemmings/issues/70) · 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1)

main3b69590173ca1f4d549f9ca32586faa34a4d835e 동기화와 이슈 번호/URL 확인 후 작업했습니다. 사용자 승인 원격 HTTP8080과 방화벽 허용은 유지합니다.

- 계약/TDD: 선택적 제품 관측·모든 제어 경계/입력/맵/화면·실패 거절 검사를 구현했습니다. Rust1개/Python9개 추가, 공통 시뮬레이션/자산 규칙·의존성 변경 없음.
- 원본 없는 CI: 실제 DOM 네 시도656상태/57장면, 추가 WASM12재생1968상태/127장면과 독립 SDL12재생651픽셀 readback 통과.
- 사용자 CDP19222: 원본 Fun01 두 번10/10 구출, 중간 Nuke0/3 실패, 재시작/재도전/다음Fun02 새 ID0.1695전체 상태/137Canvas와 raw 네이티브 및 SDL12독립 재생·684픽셀 readback 통과. 숨김/복귀·CSS 크기/외부 클릭·결과 고정·JS 예외0.
- 전체 make check(Rust526/문서28/Python115) 종료0·self-review 미해결0·출처/해시를 고정했습니다. 원본 WASM12추가재생5085상태300RGB도 통과했습니다. 한글 PR/정확한 게시 CI·병합/마감 진행 중. 최종 결과는 [보고서](../reports/issue-70_web_playthrough.md)와 #70에 기록합니다.

원본 파생물은 assets/local에만 보관합니다. 기존 DOS #60 근거와 새 브라우저 증거를 구분합니다. 이번 게시/CI 마감 후 G09 전체를 완료하고 다음 G10 사운드 통합으로 넘깁니다.
