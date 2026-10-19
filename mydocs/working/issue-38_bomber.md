# G07-06 Bomber 진행 기록

이슈: [#38](https://github.com/edwardkim/ohmylemmings/issues/38), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 범위와 현재 상태

사용자가 Blocker 동작을 확인하고 다음 절차를 지시했습니다. main558e9e5·PR #37/이슈 #36 완료와 OPEN 작업을 확인한 뒤 #38의 번호·URL·OPEN을 조회했습니다. 기존 미추적 폴더를 보존하며 self-review·보이는 원본 검증을 이어갑니다.

Bomber 할당/수량·카운트다운·기존 행동 병행·OhNo/폭발·지형 삭제와 강철/경계·Blocker 영역 해제·ID 순서·SDL 입력/표시/재생을 구현합니다. 고정 원문을 실제 실행해 합성/원본 상태를 대조하고 DOS 두 실행 관측·전체 장면 대조를 수행합니다. Climber/Floater·Nuke·구출/승패·음향·전체 G07/DOS 인증은 후속입니다.

## 구현·검증 상태

고정43개 원문 함수와 대조해 구현했습니다. 코어9개/리소스2개/네이티브5개/Python3개 테스트를 추가했습니다. 최종 전체 검사는Rust388/문서27/Python62·SDL·WASM을 통과했습니다. 합성6736상태514캡처, 원본 네이티브1364상태111캡처, 기존 다섯 직업3410상태248캡처와F00 서명을 확인했습니다.

독립된 DOS proof-2/3의1365샘플·4761게임 프레임과130/146/162/195/196/260/300/340 전체320×160 장면×2회가 일치합니다. 숫자 앵커와 폭발 지형의 한 프레임 표시 지연을 실패 테스트 후 보정했습니다. self-review 결과와 제외한 준비/중간 실행은 [보고서](../reports/issue-38_bomber.md), 계약·재현은 [기술 기록](../tech_insights/issue-38_bomber.md), 출처/해시는 [manifest](../../assets/manifests/bomber.json)에 기록합니다.

## 통합 결과

[PR #39](https://github.com/edwardkim/ohmylemmings/pull/39)를 self-review 후 병합했습니다. 검증한 HEAD `56188a20115c78e16668cddeebae7ca3cc2d4124`와 병합 main `1438c74d9f0a2263dbe6d77f08d6618927ee7291`의 트리가 같음을 확인했습니다. push/PR/main CI 세 작업과 실제 로그·원격281파일·130소스/1587산출물 해시를 확인하고 #38을 종료했습니다. [최종 통합 기록](https://github.com/edwardkim/ohmylemmings/issues/38#issuecomment-6261524793)에 근거를 남겼습니다.

사용자가 E키로 Bomber가 되는 것을 직접 확인했습니다. 사용자 확인은 E키 선택/할당 범위로 기록합니다. 다음 계획 항목은 Climber이며 G07 전체는 진행 중입니다.
