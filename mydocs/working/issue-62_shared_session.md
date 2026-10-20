# G09-01 공유 게임 세션 분리

이슈: [#62](https://github.com/edwardkim/ohmylemmings/issues/62) · 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1)

main `93578a2` 동기화·이슈 번호/URL을 확인했습니다. Session/Pacing을 SDL 없는 공유 크레이트로 옮겨 데스크톱과 향후 웹이 같은 틱 구현을 사용하게 합니다. core/resources의 no_std 경계는 유지합니다.

G09 분할·호스트/세션·자산 바이트·입력/시간·오류/전환·검증 계약을 고정하고 공개 API 실패 검사부터 구현합니다. 추출 전후 합성/원본 전체 상태·지형 대조와 기존 회귀·독립 WASM 컴파일을 검증한 후 자체 검토·한글 PR/실제 CI로 게시합니다. 브라우저 화면/완주는 후속입니다.

추출 전6실행1912상태/413캡처와 추출 후 렌더1/7의12실행3824상태/826캡처를 확인했습니다. 모든 정규화 상태·순서 있는 지형/객체 이력, 초기/최종 bin과 모든 논리 RGB가 같습니다. Session 전체는 명령 함수 위치와 테스트 fixture 경로만 바뀌었고 Pacing·명령 변환 본문은 바이트 단위로 같습니다.
