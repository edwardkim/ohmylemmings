# G11-04 전체120레벨 초기 재생 결과 기록

이슈: [#91](https://github.com/edwardkim/ohmylemmings/issues/91) · 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1) · [계약/재현](../tech_insights/issue-91_initial_replay.md).

실제 SDL 실행 파일의 전체 상태·렌더 스트림/읽기 검증·가상 시간 전달과 실제 WASM/FramePump·Canvas를 대조하는 재현 도구를 추가했다. 검증에 필요한 진단 v1을 공유했으며 규칙·초기 자원·UI 그림·기본 실행은 유지했다. 기준 main은 `dfaa1fafb55769f64c585c49105186f58c7630e4`다.

이 문서는 코드 검토 시점의 기록이다. 창작 일반1/특수4개는 플랫폼별15회, 각각5,115probe·SDL2,230readback·Canvas2,095readback을 통과했다. 무입력 실제 구출로 조기 종료한 뒤340probe까지 상태 고정도 확인했다. 스트림/경계/누락·중복 ID/차이 보고와 독립 Python 마스크·플레인 합성 검사를 추가했다. 로컬 `make check-rust check-python check-wasm`와 추가 Python 검사는 Rust611/문서29/Python172·fmt/clippy·WASM을 통과했다. 전체 SDL/Web 완주/음향과 새 창작 회귀는 게시 CI에서 검증한다. 현재 소스와 창작 검증의 해시는 [initial_replay.json](../../assets/manifests/initial_replay.json)에 고정한다.

원본120개는 각 플랫폼3회×341probe(초기0+340), 플랫폼별122,760probe의 실행 계약이다. 120개 초기 헤더/재고·카메라/팔레트와 지형·강철/일방향·raw 객체 맵, 독립 시작 합성 상부320×160 전체와 HUD 포함 SDL/WASM320×200을 대조한다. 실제로 생성된 렌더 수·frozen 결과·실행 환경과 개별/종합 해시는 전체 실행이 끝난 뒤 [GitHub #91 최종 검증 기록](https://github.com/edwardkim/ohmylemmings/issues/91)에 게시한다. 검증이 끝나기 전에는 전수 통과로 표시하거나 병합하지 않는다.

초안 PR 동안 원본 전수 실행과 원본 없는 CI를 병행한다. 정확한 head·동일 병합 트리·push/PR/main 네 작업의 전체 로그를 확인한 뒤 #91을 완료 처리한다. 이는 초기 재생 범위의 인증이며 유효 해법 완주/COMP 전수·새 DOS 관측·성능 측정 인증은 아니다. 다음 범위는 G11-05다.
