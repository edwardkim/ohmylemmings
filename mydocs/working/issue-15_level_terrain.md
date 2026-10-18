# G03-06 레벨 지형 재구성 진행 기록

작업 이슈: [GitHub #15](https://github.com/edwardkim/ohmylemmings/issues/15). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 범위와 순서

번호·URL·OPEN 상태를 먼저 확인했습니다. 고정 소스·원본 배치 분석 → 합성 픽셀 TDD → 무할당 초기 지형 API → 전체 120개 진입점 독립 비교 → 문서·검사·게시·실제 CI 순서로 진행합니다.

초기 지형과 팔레트만 합성하며 객체 애니메이션·충돌·동적 지형·MAIN.DAT 스프라이트·게임 화면은 후속 범위입니다.

## 현재 상태

초기 지형 API·호스트 예제·검증 매니페스트를 구현하고 로컬 검증·실제 CI·원격 게시 확인을 마쳤습니다. [기술 계약·재현 명령](../tech_insights/issue-15_level_terrain.md)과 [결과 보고서](../reports/issue-15_level_terrain.md)에 근거를 기록했습니다. 최종 문서 게시·CI와 #15 종료는 GitHub 이슈 기록에서 추적합니다.

## 수행과 근거

- 고정 Lemmix의 1584폭·no-overwrite 우선·특수 배경 (304,0) 대체 규칙을 적용했습니다.
- 합성 Red 14개 실패·기존 리소스 107개 통과를 확인한 뒤 Green 121개를 확인했습니다.
- `make check`: Rust 단위 123개·문서 예제 10개·Python 8개, 포맷·Clippy·빌드·WASM 성공.
- 80개 기본 배치와 120개 진입점의 30412800개 인덱스·점유 픽셀·팔레트를 독립 Python과 대조했습니다.
- 실제 TS Frame·ColorPalette·GroundRenderer도 실행했습니다. 행별 폭·플래그 우선순위·투명 RGBA를 정규화하면 전체 점유·RGB·팔레트가 일치합니다. TS 원래 우선순위의 12개 진입점·27966픽셀 차이는 별도 기록했습니다.
- CLI 오류 8개에서 실패와 경로 문맥·기존 출력 보존을 확인했습니다. 일반 지형과 네 특수 배경 미리보기를 확인했습니다.
- 문서 재현 명령·Markdown 60개·로컬 링크 390개·앵커 66개·이슈 문서명 48개·#15 일지·고정 입력 5개·기존 로컬 의존 하나를 확인했습니다.

## 게시와 다음 행동

구현 커밋 `a859043227a76f5a538da378bf03415c0483fce1`의 [실제 CI 세 작업](https://github.com/edwardkim/ohmylemmings/actions/runs/39118315817)이 성공했습니다. 실제 로그·clean-input 검사·원격 파일 97개의 모드·blob 해시와 깨끗한 작업 트리를 확인했습니다.

최종 문서 게시·CI 확인 후 상위 #1 갱신·#15 종료로 마칩니다. G03 전체는 진행 중이며 후속 G03-07 MAIN.DAT 레밍 스프라이트와 G04-01은 각각 별도 이슈를 먼저 등록·확인합니다.
