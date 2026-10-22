# macOS 전체 게임 앱 패키지

담당 [#120](https://github.com/edwardkim/ohmylemmings/issues/120), 상위 [#115](https://github.com/edwardkim/ohmylemmings/issues/115).

Windows 실행 확인 후 Mac 배포 준비 요청에 따라 원본28개(그래픽·120개 레벨·음원)를 포함하는 `OhMyLemmings.app` universal2 ZIP을 준비한다. Apple Silicon과 Intel용 실행 파일·SDL2 dylib를 각각 두 Mach-O slice로 구성하며 앱 내부 Resources/data와 Frameworks만으로 원본 Fun01을 시작한다. Homebrew/SDL/원본 별도 설치를 요구하지 않는다.

## 사용자 실행과 서명 상태

`ohmylemmings-0.1.0-macos-universal2.zip`을 모두 풀고 `OhMyLemmings.app`을 Applications에 복사해 연다. 앱 전체를 함께 유지한다. 기본 시작은 원본 Fun01이며 결과 Enter는 다음, R은 재시작이다. 앱 안 Contents/Resources에 SDL/OPL 라이선스·게임 출처/해시, ZIP 옆 source.tar.gz/REBUILD.md·내외부 체크섬을 제공한다. 시작 오류는 `~/Library/Logs/OhMyLemmings/last-error.log`와 메시지로 남는다.

이 산출물은 **ad-hoc 서명 시험 패키지**이며 Apple Developer ID/공증 완료 배포가 아니다. 최초 인터넷 다운로드는 Gatekeeper에서 차단될 수 있다. 앱을 열어 본 뒤 시스템 설정 > 개인정보 보호 및 보안 > 그래도 열기 절차는 [Apple 공식 안내](https://support.apple.com/102445)를 따른다. Gatekeeper를 전역 해제하거나 다운로드 격리 속성을 지우도록 요구하지 않는다. 일반 배포용 서명/공증은 Developer ID Application 인증서·Apple 공증 자격을 확보한 뒤 수행한다.

정식 배포 절차: dylib를 먼저 Developer ID로 서명하고 앱을 hardened runtime/타임스탬프로 서명한다. `ditto -c -k --keepParent OhMyLemmings.app upload.zip` 후 `xcrun notarytool submit upload.zip --keychain-profile PROFILE --wait`로 제출한다. Accepted 로그를 확인하고 `xcrun stapler staple/validate OhMyLemmings.app`, `codesign --verify --deep --strict`, `spctl --assess --type execute`를 확인한다. 최종 ZIP/체크섬과 notarized 메타데이터를 별도로 생성해 다운로드 격리 상태의 깨끗한 Mac에서 첫 실행을 검증한다. 현재 도구의 ad-hoc ZIP을 공증된 배포로 표시하지 않는다. [Apple Developer ID](https://developer.apple.com/developer-id/), [공증](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution).

## 빌드와 검증

Rust1.93.1, Xcode SDK·CMake·Python3.12+를 사용한다. `python3 tools/macos_package.py prepare-sdl`은 고정 SDL2 2.32.10 source tar의 SHA256/크기를 검증하고 macOS13 deployment target의 universal dylib를 빌드한다. `.github/workflows/ci.yml`의 Mac 빌드 명령으로 두 Rust target을 빌드하고 lipo로 결합한다. `stage` 후 dylib와 앱을 차례로 ad-hoc 서명하고 `create`로 ZIP을 생성한다. `verify`는 Mach-O 두 slice/종류·앱 Info.plist·파일 목록/실행 권한·원본28개·소스/라이선스/체크섬을 검증한다. 기존 산출물은 덮어쓰지 않는다.

패키징5개와 앱 내부 자산 경로1개는 합성 offline fixture red→green으로 검증한다. CI의 macOS15 Apple Silicon은 깨끗한 source/locked Cargo/고정 SDL source와 원본 입력을 사용하며 실제 빌드/앱 LaunchServices 무인수 표시 창·Cocoa 원본 픽셀 readback·120개 선택·ADLIB 비영 PCM·Fun01 저장 해법 성공→Restart/Continue·독립 새 시도와 누락 자산 로그를 검사한다. 다음 macOS15 Intel job은 빌드한 동일 ZIP을 다운로드해 native x86_64로 같은 검사를 수행한다. DYLD 외부 환경을 지우고 otool의 OS/@rpath 의존·codesign과 격리된 테스트 복사본의 Gatekeeper 거부를 확인한다.

`development-macos-universal2`와 `macos-intel-validation` artifact는 ZIP/외부 sha256·숫자 JSON/로그·원본 Fun01 readback 화면을14일 보존한다. 정확한 실행 source SHA·run·artifact digest·다운로드 Git blob 대조·PR/병합 상태는 [#120](https://github.com/edwardkim/ohmylemmings/issues/120)에 기록한다. 원본/파생 바이너리는 ignored assets/local과 artifact에만 둔다.

배포 target은 macOS13.0+이며 실제 runner 검증 OS는 macOS15로 구분한다. hosted Mac/SDL dummy PCM 검증은 사용자 실제 스피커·구형 OS·고성능 호스트의60fps 인증과 다르다. #113 성능 이월, 기존120해법/240사례·DOS/PCM 자료를 보존한다. Linux/Web/Windows의 동작 계약을 바꾸지 않는다.
