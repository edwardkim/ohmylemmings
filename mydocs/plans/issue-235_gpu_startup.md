# Native 시작 시 가속 렌더러 선택

담당 [#235](https://github.com/edwardkim/ohmylemmings/issues/235), 선행 [조사](../reports/issue-233_linux_graphics_review.md).

시작 시 SDL이 열거한 가속 렌더러를 실제 창에서 생성해 보고 성공한 경로를 원본/HD 모두에 사용한다. 실패하면 다음 가속 후보, 모두 실패하거나 후보가 없으면 software로 진행한다. SDL_RENDER_DRIVER=software 명시 설정은 존중한다. SDL 가속 flag와 생성 성공을 판단 기준으로 하고 실제 backend/선택 경로/실패 사유를 시작 로그에 남긴다. SDL의 GPU API 구현과 실제 물리 장치 식별은 구분한다. 새 의존성/unsafe/시뮬레이션 변경은 도입하지 않는다.

TDD로 가속 우선·후보별 초기화 실패·후보 부재·강제 software·software 실패를 검증한다. 실제 dummy/software와 Linux offscreen 가속에서 원본/HD 렌더링 및 상태 보존을 확인한 뒤 플랫폼 CI를 완료한다. #113의60fps 장기 인수는 별도이며 공개 릴리즈는 이번 범위가 아니다.
