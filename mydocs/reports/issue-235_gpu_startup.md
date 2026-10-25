# Native 시작 시 GPU 가속 선택 완료

담당 [#235](https://github.com/edwardkim/ohmylemmings/issues/235), [계획](../plans/issue-235_gpu_startup.md), [선행 조사](issue-233_linux_graphics_review.md).

원본 모드의 강제 software와 HD 모드의 암묵적 선택을 공통 시작 절차로 교체했다. SDL에 등록된 가속 렌더러를 순서대로 실제 창에서 초기화하며, 실패한 후보는 정리하고 다음 후보를 시도한다. 모두 실패하거나 후보가 없으면 명시적 software renderer를 만든다. software도 생성할 수 없으면 가속/소프트웨어 실패 사유를 포함해 종료한다. 실패한 consuming CanvasBuilder가 창을 파괴하지 않도록 Window의 공유 소유권을 유지한다.

`SDL_RENDER_DRIVER=software`는 가속 시도 없이 존중한다. 다른 renderer 선호 설정은 해당 가속 후보의 우선순위로 적용한다. 원본/HD 전환에도 같은 선택 렌더러를 사용한다. 시작 로그 `LEMMINGS_RENDERER`에 mode·video_driver·실제 renderer·flags·최대texture·선택 사유를 기록하며 기존 F2 피드백도 실제 backend를 제공한다.

가속 판단은 SDL ACCELERATED flag와 실제 renderer 생성 성공이다. SDL이 내부에서 소프트웨어 GL을 제공하는 환경에서 물리 GPU를 판별하는 기능은 아니다. 이 변경은 texture 표시 경로 선택이며 시뮬레이션/CPU HD 지형 생성의 compute 이행이나 새 효과·자산 제작을 포함하지 않는다. 새 의존성/unsafe 코드는 없다. #113의 남은 성능 인수와 기존 측정값은 별도로 유지한다.

## 검증

- 선택 정책7개: software가 먼저 열거되어도 가속 우선, 첫 가속 후보 실패 후 다음 후보 성공, 전체 가속 초기화 실패 후 software, 가속 후보 부재, 명시적software, 선호 후보 우선, software까지 전체 실패 시 진단 보존. TDD red/green을 기록했다.
- 실제 synthetic SDL4경로(dummy 자동fallback/명시software, offscreen OpenGL자동선택/명시software)의81상태가 동일하다. 원본 캡처32개의 정확한 정수 배율·픽셀을 검사했다. 실제 native summary의 backend와 startup log도 대조했다.
- 기존 desktop 재생4사례·1,124스냅샷·42정확한픽셀 캡처 통과. dummy 환경이 실제 software fallback과 시작 로그를 사용함을 확인했다.
- 실제 Fun01 HD에서 Linux offscreen OpenGL/software 각각81틱 실행·9표시를 확인하고 trace와 canonical 원본RGB가 동일함을 확인했다. 최종 HD framebuffer의 GPU/소프트웨어 완전 동일이나 전체120레벨 수동 플레이·FPS 인수 결과는 아니다.
- `tools/check_renderer.py`를 `make check-desktop`에 추가했다. 기본 CI는 dummy의 GPU부재/강제software를 검증하고 `--gpu-video-driver offscreen`은 가능한 환경에서 실제 가속 경로를 요구한다.

`make check-rust` 통과: workspace744테스트·format·Clippy·상태 재생·음악/효과음 검증. `make check-python` 통과:339테스트·자산/고지/기존 검증 증거 검사. 기존 NOT READY 결과는 보존했다.

[PR#236](https://github.com/edwardkim/ohmylemmings/pull/236)의 [CI40156627683](https://github.com/edwardkim/ohmylemmings/actions/runs/40156627683) 7개 항목이 모두 SUCCESS다. PR head `5f889cefaf85c453bba84615eb445b1ab892996b`를 검증했고 실제 checkout `e7c6f2a`의 tree는 병합 main `c0a57ab73b6d7720ee7c3b7a8b36aa36b333f97c`의 `6487db952d9afb22e85380ea8d2ea51405cdf3c9`와 동일하다. 중복 push CI만 취소했다.

이번 제품 병합은 skip-ci를 사용하지 않았으며 [main CI40157629542](https://github.com/edwardkim/ohmylemmings/actions/runs/40157629542)가 자동 시작됐다. 이 보고서를 작성할 때는 main 재검증이 진행 중이며 그 결과를 이미 성공한 것으로 주장하지 않는다. 완료된 플랫폼 인수 근거는 위 동일 tree PR CI다.

개인 로그·원본 유래 HD 캡처는 ignored `assets/local/issue-235-gpu/`에만 둔다. 시스템 GPU/드라이버 설정과 기존 공개v0.3.3/Pages는 변경하지 않았다. 새 배포는 별도 릴리즈 절차다.
