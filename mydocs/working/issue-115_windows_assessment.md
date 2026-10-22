# Windows 패키징 가능성 검토

담당 [#115](https://github.com/edwardkim/ohmylemmings/issues/115), 기존 [배포 계약](../plans/issue-115_release_preparation.md).

2026-10-22 사용자가 Windows 패키징 가능성 검토를 요청했다. 기존 #115 OPEN·번호/URL과 main `41ec527` 동기화를 확인한 뒤 별도 검토 branch에서 실제 Windows MSVC 빌드·추출 ZIP 실행을 시험한다. 제품 Rust/패키징 명령과 main은 변경하지 않는다. 현재 호스트의60fps 조정은 재개하지 않는다.

- 공통 코어/자원/세션과 SDL2 데스크톱에 Windows를 막는 Unix 전용 API 사용은 찾지 못했다. 고정 sdl2 0.38.0은 MSVC import library와 exe 옆 SDL2.dll 배치를 지원한다.
- 공식 SDL2 2.32.10 VC archive SHA256 `af347939395a58b365846aaea27391e69f9ec9d4dd650d6ac40802159b418a6e`와 x64 library/DLL·라이선스 입력을 재사용한다. 전역 Rust/sysroot를 바꾸지 않고 CI process의 LIB로 링크한다.
- windows-2022 runner에서 고정 Rust로 기존 desktop 코드를 그대로 빌드한다. 실행 파일/SDL DLL/두 라이선스/전체 대응 소스/해시/안내의 시험 ZIP을 만들고 별도 공백·한글 경로에서 help, Won/Restart/Continue/Lost와 SDL dummy 오디오를 확인한다. 실제 PE 의존 DLL도 기록한다.
- 이 실험은 가능성 검토다. 정식 `release_package.py windows` 구현·사용자 PC 설치/화면/청취·서명/배포·성능 인증은 별도이며 #115/#113 잔여로 유지한다. 실제 결과는 검토 보고서와 #115에 기록한다.

- 기본 MSVC 빌드의 [run39726923791](https://github.com/edwardkim/ohmylemmings/actions/runs/39726923791)이 통과했다. 기존 코드를 바꾸지 않고 help·한글/공백 경로·Won/Restart/Continue/Lost·비영 PCM/SDL dummy 오디오를 확인했다. 실제 exe에는 VCRUNTIME140.dll과 UCRT import가 있어 정적 CRT 후보를 별도로 시험한다. 이 기본 후보와 원시 결과는 보존한다.

- 정적 CRT의 [run39727233075](https://github.com/edwardkim/ohmylemmings/actions/runs/39727233075)도 동일 기능을 통과했다. exe1,000,448bytes, VCRUNTIME140/MSVCP140/UCRT API-set 직접 import 없음. 실제 checkout/log·artifact digest·내부8개 파일과704개 소스 바이트를 Git blob에 대조했다.
- [검토 보고서](../reports/issue-115_windows_packaging_review.md): Windows x64 ZIP은 가능하다. 현재 기본 실행은 상대 DOS 자산 경로를 요구하므로 사용자용 런처/자산 폴더 안내, 정식 packager/CI 통합과 실제 Windows PC 화면/입력/청취가 후속이다. 검토 branch는 main에 병합하지 않으며 정식 Windows 패키지 완료로 표시하지 않는다.
