# macOS 전체 게임 앱 배포 계약

담당 [#120](https://github.com/edwardkim/ohmylemmings/issues/120), 상위 [#115](https://github.com/edwardkim/ohmylemmings/issues/115).

사용자가 Windows 패키지 실행을 확인한 뒤 Mac 배포 준비를 요청했다. 원본 그래픽·전체120개 레벨·음원28개를 포함한 `OhMyLemmings.app` ZIP을 제공한다. 앱을 열면 원본 Fun01을 시작하고 별도 게임 자산/SDL 설치를 요구하지 않는다. Apple Silicon/Intel universal2와 deployment target macOS13.0, 실제 macOS15 ARM64/Intel runner 검증을 준비한다. 이전 Windows와 같은 완전 게임 데이터 계약이며 Linux/Web 개발 패키지 계약을 바꾸지 않는다.

앱 구조는 Contents/MacOS/OhMyLemmings, Contents/Resources/data, Contents/Frameworks/libSDL2-2.0.0.dylib과 Info.plist를 사용한다. 고정 SDL2 2.32.10 소스를 검증하고 macOS SDK로 universal dylib를 빌드한다. 앱 내부 @rpath만으로 로드하며 Homebrew/개발 폴더 의존을 허용하지 않는다. 원본/SDL/OPL 출처·라이선스/대응 소스/해시와 재빌드 안내를 제공한다.

실제 Apple Silicon 빌드/실행 및 Intel 실행을 구분해 검증한다. 새 한글/공백 경로·다른 CWD에서 앱 open/원본 화면·120개 선택·음향 PCM·Fun01 성공/Restart/Continue와 독립 새 시도를 검사한다. Mach-O 두 slice, 동적 의존 경로·codesign 검증과 quarantine/Gatekeeper 판정을 기록한다. 테스트용 ad-hoc 서명은 Developer ID/Apple 공증으로 표시하지 않는다. 저장소에 서명 자격이 없으므로 정식 다운로드 신뢰에는 별도 Developer ID/공증 자격이 필요하다.

참고: [GitHub runner](https://docs.github.com/en/actions/reference/runners/github-hosted-runners), [Rust Darwin](https://doc.rust-lang.org/rustc/platform-support/apple-darwin.html), [SDL2 CMake](https://wiki.libsdl.org/SDL2/README-cmake), [Apple Developer ID](https://developer.apple.com/developer-id/), [공증](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution).
