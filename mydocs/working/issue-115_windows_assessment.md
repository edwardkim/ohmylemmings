# Windows 패키징 가능성 검토

담당 [#115](https://github.com/edwardkim/ohmylemmings/issues/115), 기존 [배포 계약](../plans/issue-115_release_preparation.md).

2026-10-22 사용자가 Windows 패키징 가능성 검토를 요청했다. 기존 #115 OPEN·번호/URL과 main `41ec527` 동기화를 확인한 뒤 별도 검토 branch에서 실제 Windows MSVC 빌드·추출 ZIP 실행을 시험한다. 제품 Rust/패키징 명령과 main은 변경하지 않는다. 현재 호스트의60fps 조정은 재개하지 않는다.

- 공통 코어/자원/세션과 SDL2 데스크톱에 Windows를 막는 Unix 전용 API 사용은 찾지 못했다. 고정 sdl2 0.38.0은 MSVC import library와 exe 옆 SDL2.dll 배치를 지원한다.
- 공식 SDL2 2.32.10 VC archive SHA256 `af347939395a58b365846aaea27391e69f9ec9d4dd650d6ac40802159b418a6e`와 x64 library/DLL·라이선스 입력을 재사용한다. 전역 Rust/sysroot를 바꾸지 않고 CI process의 LIB로 링크한다.
- windows-2022 runner에서 고정 Rust로 기존 desktop 코드를 그대로 빌드한다. 실행 파일/SDL DLL/두 라이선스/전체 대응 소스/해시/안내의 시험 ZIP을 만들고 별도 공백·한글 경로에서 help, Won/Restart/Continue/Lost와 SDL dummy 오디오를 확인한다. 실제 PE 의존 DLL도 기록한다.
- 이 실험은 가능성 검토다. 정식 `release_package.py windows` 구현·사용자 PC 설치/화면/청취·서명/배포·성능 인증은 별도이며 #115/#113 잔여로 유지한다. 실제 결과는 검토 보고서와 #115에 기록한다.
