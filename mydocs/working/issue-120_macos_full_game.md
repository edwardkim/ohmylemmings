# macOS 전체 게임 패키지

담당 [#120](https://github.com/edwardkim/ohmylemmings/issues/120), 상위 [#115](https://github.com/edwardkim/ohmylemmings/issues/115).

사용자가 Windows 실제 실행을 확인했다. main ef72267의 전체5개 CI 성공을 확인하고 번호/URL 검증 뒤 별도 worktree에서 Mac 준비에 착수했다. self-review를 유지한다. 개인 미추적 항목과 기존 원본/성능 증거를 보존한다.

Apple Silicon/Intel universal2 앱, 원본28개·SDL2 dylib와 앱 내부 자산 경로를 구현한다. macOS15 ARM/Intel 실제 runner를 사용하며 Linux 교차 빌드로 실행 검증을 대신하지 않는다. SDL2 2.32.10 공식 tar.gz의 GitHub release SHA/크기를 확인했다. GitHub secret 이름 목록은 비어 있으며 자격을 발급/구매하거나 Developer ID/공증 완료를 임의로 선언하지 않는다. ad-hoc 서명 시험 패키지와 서명/공증 절차를 준비한다. #113 성능 이월을 유지한다.

앱 자산 경로1개/패키징5개 red→green을 기록했다. universal2 실행 파일·동봉 SDL·Info.plist/권한·원본/source/hash 검사와 ad-hoc 서명 ZIP을 구현했다. 실제 ARM 빌드/LaunchServices/Cocoa/120개/PCM/Restart/Continue와 동일 ZIP Intel 검사를 CI에 연결하며 Gatekeeper 거부와 공증 잔여 조건을 분리한다.
