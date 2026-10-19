# G07-09 Nuke 진행 기록

이슈: [#44](https://github.com/edwardkim/ohmylemmings/issues/44), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

사용자가 다음 단계 진행을 승인했습니다. 선행 Floater PR #43/main feab3e8851a30fdf6d32845953f3ac903a3d648e의 세 CI·실제 로그·#42 종료/상위 상태를 확인했습니다. #44의 번호·URL·OPEN 확인 후 issue-44-nuke 브랜치를 만들었으며 관계없는 미추적 폴더를 보존합니다. 준비 이슈를 만들지 않습니다.

고정 BtnNuke/CheckSpawnLemming/CheckLemmings/CheckUpdateNuking을 읽었습니다. 입력 즉시 이후 생성을 멈추고, 각 논리 틱의 기존 개체 이동/카운트다운 이후 타이머가 없는 제거 개체를 건너뛰어 다음 개체 하나를 처리합니다. DOS EXE213a가 제거 검사2140보다 먼저 타이머를 검사하므로 제거됐어도 타이머가 남은 개체는 한 순서를 소비합니다. 이 경우 Lemmix의 무조건 제거 건너뛰기와 다르며 DOS 원문과 전용 테스트를 기준으로 합니다. 이미 타이머가 있거나 Splatting/Exploding이면 재설정하지 않지만 해당 순서는 소비합니다. 수량을 쓰지 않고79를 설정하며 Blocker/영구 능력은 기존 Bomber 전환에서 처리합니다. 마지막 개체 처리 후 다음 틱에 진행 플래그가 꺼집니다.

일반 DOS 조작의 Pause 중 Nuke 금지와 원문 BtnNuke 자체의 별도 무방비 호출을 구분합니다. 기본 입력 어댑터는 Pause 중 거부하며 Resume 뒤 같은 경계의 Nuke는 순서대로 처리합니다. NukeGlitch는 원래 최대 인원→이미 생성된 인원 상태 변경까지만 다루며 구출률/승패 계산은 후속입니다. no_std 코어·N키/재생/추적·원문 실제 실행/합성/원본 네이티브/DOS 두 실행·8직업 회귀/전체 검사·self-review/PR/CI를 진행합니다. G07 전체를 완료 처리하지 않습니다.

코어 새11개(통합10/시작 오버플로1)·네이티브 새2개와 기존 Rust/Clippy를 확인했습니다. 고정 원문50함수+표·합성10사례20재생8420상태1278캡처가 일치합니다. 위 DOS 제거/타이머 예외는 별도 TDD로 검증하며 원문 대조 사례가 이 예외까지 같다고 확대하지 않습니다. reference-1은 호스트 enum 치환의 중복 탐지로 빌드하지 못했으며 수정한 reference-2를 최종 기준으로 사용합니다.

## 최종 검증과 통합

최종 바이너리79837fe0·reference-2/10ece6fd로 합성10사례20재생8420상태1278캡처, 원본 Fun06 네이티브4실행1364상태171캡처를 검증했습니다. 고정 세 입력은120, 실제 N키는121이며 각 실제 입력으로 원문과 일치합니다. DOS proof-4/5의802샘플·검사 게임영상2646프레임·전체 여덟 장면×두 실행이 일치합니다. 이후436개의640×400 프레임은 범위에서 제외하고 전체4876디코드 프레임을 보존합니다.

기존8직업16재생5376상태470캡처/F00·전체 `make check`358.801초·Rust425/문서27/Python73·SDL·WASM을 통과했습니다. 도구의 직접 창 키 전달/녹화 자식 종료·마지막 앵커 이후 제외 보완은 최종 Python73개/실제 DOS 두 실행으로 검증했습니다. 원본/소스/사본·157소스/2684산출물 해시와 [보고서](../reports/issue-44_nuke.md)·[계약](../tech_insights/issue-44_nuke.md)·[manifest](../../assets/manifests/nuke.json)를 고정합니다.

reference-1 호스트 치환 중복, native-proof-1 포커스 유실, DOS proof-1 preview Space·proof-2 종료 keyup·proof-3 종료 미수신은 최종 횟수에서 제외합니다. 비활성 spawn 카운터/논리 Nuke 플래그와 원시 바이트의 표현 차이, 별도 DOS 타이머 우선 경계를 명시합니다. self-review 후 정확한 HEAD의 push/PR CI·원격 파일과 main 병합 트리/CI·#44 종료·상위 상태를 확인합니다. 후속은 전체 직업 전환/상호작용 마무리이며 하단 기술 패널은 G08 HUD 범위입니다.
