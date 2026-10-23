# #184 HD 글꼴·전체 게임 UI

이슈: https://github.com/edwardkim/ohmylemmings/issues/184 · 상위 #137

현재 HD 게임의 상태/기술/미니맵 패널과 Pause·카운트다운·성공/실패/다음 안내를 완성한다. 공통 글꼴은 원본5×7 확대에서 고정 출처·라이선스의 AA atlas로 교체하고 Native/Web에 같은 RGBA를 제공한다.14개 조작 영역·원본 모드·게임/입력/사운드 규칙은 유지한다. 새 DOS 메뉴는 추가하지 않는다. TDD·대표 Native/Web 입력/상태/전환·패키지 내 글꼴/고지·픽셀/예산·self-review·정확한 head CI·실제 병합으로 검증한다. 전체 HD 배포/인수는 후속, BGM #165는 리마스터 이후다. 사용자19222/8080·preview-172와 파일은 유지한다.

[결과·출처·재현·한계](../reports/issue-184_hd_ui.md)에 증거와 마감 상태를 기록한다. 현재 제품 UI를 범위로 하며 원본 DOS 메뉴200개 그림의 재제작을 완료했다고 세지 않는다.

완료: PR #185 head `58778877471c1c019b9aeb5844e7c09c5bd023b8` / CI39964641509 최종7개 SUCCESS, 실제 merge `b32d01c6ea76e3feca1ade10a241b078a7ffbb8e`와 검토 head 전체 tree 일치를 확인했다. [검증/재현/범위](../reports/issue-184_hd_ui.md).
