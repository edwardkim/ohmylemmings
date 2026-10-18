# G06-01 네이티브 화면 진행 기록

작업 이슈: [GitHub #27](https://github.com/edwardkim/ohmylemmings/issues/27). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 범위와 순서

번호·URL·OPEN을 확인했습니다. SDL2 화면/키보드 어댑터·Fun01 원본 조립·카메라·17Hz 세션을 구현합니다. Pause/Resume·RR·종료 입력과 렌더 빈도 독립성을 검증합니다. 직업·객체 트리거·승패·음향과 DOS 대조는 후속 범위입니다.

## 현재 상태

SDL2 네이티브 화면/입력·원본 조립·실제 X11/키/화면·공통 검사·구현 게시/CI를 완료했습니다. 두 no_std 라이브러리와 단방향 의존을 유지합니다. G06 전체는 DOS 관측이 남아 진행 중입니다.

## 수행과 근거

- [기술 계약](../tech_insights/issue-27_native_view.md), [보고서](../reports/issue-27_native_view.md).
- 세션 새 5개 Red와 화면 새 4개 Red/기존 7개 통과에서 데스크톱 15개 Green을 확인했습니다.
- SDL x11/software 2.30.0에서 원본 F00을 20.180초·340제어 호출·870렌더로 실행했고 341개 상태가 기록된 Lemmix 실행과 같습니다. 57렌더의 재생도 같은 상태입니다.
- 원본 6실행·1786상태·63개 화면 전체 픽셀을 대조했습니다. X11 키로 실제 창에 입력한 5개 게임 명령을 정확한 정수 경계로 재생해 81개 상태가 같습니다.
- make check: Rust 단위/예제 297개·문서 27개·Python 18개·기존 두 참조/독립 모델·SDL 합성 네 실행/1124상태/40캡처·포맷·Clippy·네이티브/WASM 성공.
- 구현 게시·실제 CI·원격 tree를 확인했습니다. 최종 문서 게시·CI·상위 #1 갱신·#27 종료를 GitHub에서 추적합니다. G06 전체는 DOS 실행·동일 장면 대조가 남아 진행 중입니다.

- 문서의 X11 재현 sh 블록을 그대로 실행했습니다. 독립 실행의 정수 키 입력 경계도 기록/재생 결과가 같고 63개 전체 픽셀 검증을 통과했습니다. 문서 98개/링크 697개/앵커 114개/이슈명 84개와 standalone make check-desktop도 통과했습니다.

구현 `766cca58a9657e354fa2f30b66754dea6f3014a7`의 [실제 CI 세 작업](https://github.com/edwardkim/ohmylemmings/actions/runs/39152276025)·SDK/fetch 준비·297/27/18 테스트·SDL/두 모델 서명과 원격 파일 177개를 확인했습니다. 다음 후보는 별도 이슈의 G06-02 DOS 실행·동일 장면 대조입니다.
