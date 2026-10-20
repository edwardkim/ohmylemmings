# G09-02 자산 바이트·공통 장면 분리

이슈: [#64](https://github.com/edwardkim/ohmylemmings/issues/64) · 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1)

main `eb442214d7edaca99bf49948037414a112fe4790` 동기화·이슈 번호/URL을 확인하고 `issue-64-byte-assets-scene`에서 진행합니다. `lemmings-game`에 Assets/CPU 화면·패널·포인터 표현·LevelRun을 추출하고 desktop에 파일 읽기/경로와 SDL을 남겼습니다. core/resources no_std와 session 소스는 유지합니다.

공개 API 미구현 실패→정상 바이트/누락·손상·공급 실패·ODD·선택 EXE·전환 보존7개, 기존 공유24/desktop27 테스트와 전체 Rust513/문서28/Python101·Clippy·독립 WASM 빌드를 확인했습니다. 추출 전6실행1912상태/413캡처와 추출 후12실행3824상태/826캡처의 전체 상태/이력·bin·RGB가 같습니다. 원본29개 표준 그래픽의 파일/메모리 공급 및1929상태/프레임 비교도 일치합니다.

[구현 계약](../tech_insights/issue-64_shared_game.md)·[보고서](../reports/issue-64_shared_game.md)·[증거 해시](../../assets/manifests/shared_game.json)에 결과를 기록합니다. 자체 검토와 최종 make check·정확한 push/PR/main CI/실제 로그·병합/이슈 종료 결과는 #64에 누적합니다. 다음은 G09-03 JS/WASM·합성 브라우저이며 CDP19222를 사용합니다. G09 전체는 진행 중입니다.
