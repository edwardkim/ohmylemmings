# GPU 시작 선택 진행

담당 [#235](https://github.com/edwardkim/ohmylemmings/issues/235). [계획](../plans/issue-235_gpu_startup.md).

원본 모드의 강제 software와 HD의 암묵적 기본 선택을 공통 시작 절차로 바꾼다. 테스트 및 실제 SDL 경로 검증 후 결과를 기록한다.

선택 정책7개 테스트 통과(TDD red/green). 실제 dummy 자동fallback·명시software·offscreen 자동OpenGL·offscreen 명시software의81상태 일치와32개 원본 캡처의 정확한 배율/픽셀 검사 통과. 실제 Fun01 HD에서도 OpenGL/software 각각81틱 실행과 상태/원본RGB 동일성을 확인했다. 전체 Rust/Python 검사는 진행 중이며 원본 유래 캡처/로그는 ignored `assets/local/issue-235-gpu/`에 둔다.

로컬 Rust744/Python339·실제 GPU/software·PR CI7개 통과 후 PR#236을main에 병합했다. main CI40157629542도 자동 시작했다. [완료 보고서](../reports/issue-235_gpu_startup.md)를 따른다.
