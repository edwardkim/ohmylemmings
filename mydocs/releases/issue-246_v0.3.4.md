# Oh My Lemmings 0.3.4 — WebGPU 렌더링 / WebGPU rendering

담당 이슈: https://github.com/edwardkim/ohmylemmings/issues/246

## 변경 사항 / Changes

- 웹에서 WebGPU가 사용 가능하면 원본/HD 화면을 GPU 렌더러로 표시합니다. 미지원·초기화 실패·장치 손실에는 현재 게임을 유지하며 Canvas2D로 전환합니다. FPS 버튼 옆 표시와 F2 피드백에서 실제 렌더러를 확인할 수 있습니다.
- WebGPU에는 HTTPS 또는 localhost 접속과 지원 브라우저/드라이버가 필요합니다. 일반 LAN HTTP에서는 Canvas2D로 실행합니다. 비교/진단은 `?renderer=canvas2d`로 할 수 있습니다. GPU 사용이 게임 속도나 일정 FPS 달성을 보장하지는 않습니다.
- Native는 시작 시 SDL 가속 렌더러를 선택하고 실패하면 software로 실행합니다 (#235). 기본 표시 목표는40FPS이며 게임 논리·음향 속도를 유지합니다 (#240). #113 성능 추적은 메인테이너 결정으로 종료했습니다.
- Web now renders original and HD graphics with WebGPU when available. Unsupported browsers, initialization failures and device loss fall back to Canvas2D without restarting the game. The active renderer appears beside the FPS controls and in F2 feedback.
- WebGPU requires HTTPS or localhost and browser/driver support. LAN HTTP uses Canvas2D. Add `?renderer=canvas2d` for comparison. Native now selects an SDL accelerated renderer with software fallback and targets40FPS. Gameplay timing is unchanged; a fixed achieved FPS is not promised.

## 다운로드와 실행 / Download and play

[웹 플레이 / Play online](https://edwardkim.github.io/ohmylemmings/) · [다운로드 / Downloads](https://github.com/edwardkim/ohmylemmings/releases/tag/v0.3.4)

Windows11 x64: ZIP을 전부 풀고 OhMyLemmings.exe를 실행하세요. Windows10은 지원하지 않습니다. macOS13+ universal2: 앱을 Applications로 복사하세요. ad-hoc 서명/미공증입니다. Linux x86_64: SDL2 runtime이 필요합니다. Web ZIP은 HTTP 서버로 제공하세요. 원본120레벨·HD·효과음/BGM·사용자 MP3·점진적 다운로드를 제공합니다.

Windows11 x64: extract the full ZIP and launch OhMyLemmings.exe. Windows10 is unsupported. macOS13+ universal2: copy the app to Applications; it is ad-hoc signed and not notarized. Linux x86_64 requires SDL2. Serve the Web ZIP over HTTP. Local MP3 files remain on your device.

## 대응 소스 / Corresponding source

게임 실행에 소스 다운로드는 필요하지 않습니다. 수정·재링크하려면 같은 릴리즈의 `ohmylemmings-0.3.4-source.tar.gz`와 패키지 REBUILD.md를 사용하세요. 소스 URL/크기/SHA-256을 각 패키지에 고정하며 SHA256SUMS와 RELEASE-MANIFEST.json을 함께 제공합니다.

The separate source archive is optional for playing and supports modification/relinking. Packages pin its URL, size and SHA-256. Checksums and CI/source provenance accompany the downloads.

자체 코드는 MIT입니다. 제3자 고지·원본 게임 데이터 권리 경계를 유지하며 원본 자산 재배포 권한 확보를 주장하지 않습니다. Project code is MIT; third-party notices and original-data rights boundaries remain unchanged.
