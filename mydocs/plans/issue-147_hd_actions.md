# #147 HD-03b 직업·시설·패널 자산

이슈: https://github.com/edwardkim/ohmylemmings/issues/147 · 상위 [#137](issue-137_2d_remaster.md)

main b6ae82b과 이슈 번호/URL·OPEN을 확인한 뒤 별도 worktree에서 착수했다. [아트 계약](issue-138_hd_art_contract.md), [표시 계약](issue-140_hd_presentation.md), [기존 팩](issue-144_hd_assets.md)을 따른다.

1. built-in image_gen으로 굴착16·계단 좌우32·OhNo16/폭발1과 첫 테마 입출구·패널13아이콘을 제작한다. 실제 생성 크기·투명 여백·동작/방향·프레임 대응을 검토하고 원본 PNG를 수정 없이 보존한다.
2. 팩의 기존 actor/material 키와 호환되는 typed 시설/조작 키를 추가한다. 시설 범위는 원본 메타데이터로 제한하고 조작은 기존 HD Control과 연결한다. 합성 실패 테스트를 먼저 작성한다.
3. PNG 경계/coverage 검증·미리보기/Native probe를 확장한다. 실제 팩의 프레임/CRC·시설/아이콘 조회와 WASM library 컴파일을 검증한다.
4. 출처·프롬프트/PNG/팩 해시·후보 coverage·한계를 기록하고 로컬 검사·self-review·정확한 PR head의 CI 뒤 실제 병합한다.

조작13개는 속도감소/증가·8직업·Pause/Nuke/Restart다. 글자·숫자·상태 테두리·미니맵·status는 실제 UI 코드/상태가 담당하며 그림에 굽지 않는다. 실제 HD 게임/GPU·Fun01 입력 통합은 HD-04이며 이 절편을 플레이 완료로 표시하지 않는다. 전체337프레임/나머지 테마·효과는 HD-05 후속이다. #113 성능 이월·#142 Chrome156 진단·Windows11 정책·공개 v0.1.0/Pages를 유지한다.
