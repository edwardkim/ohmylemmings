# Oh My Lemmings 0.3.5 — 지형·지물 재질 개선 / Scenery material polish

담당 이슈: https://github.com/edwardkim/ohmylemmings/issues/252

## 변경 사항 / Changes

- 단색 회색이던 강철에 금속판 이음새·리벳·브러시 결·마모와 테마별 색상을 적용했습니다. Builder가 놓는 계단에도 벽돌 질감과 입체 음영을 더했습니다.
- 입출구·위험물·함정·깃발·방향 표지와 특수 배경의 확대 보간을 개선했습니다. 결정 테마의 빨간 방향 표지는 투명한 내부를 유지하며 테두리에 광택과 음영을 더했습니다.
- Native·Canvas2D·WebGPU에 같은 개선을 적용하며 원본 모드, 지형/충돌/강철 판정, 게임·음향 속도와 애니메이션 주기는 유지합니다. 추가 그래픽 다운로드는 없습니다. 메인테이너가 개발 미리보기를 확인했습니다.
- Steel now has themed metal plates, seams, rivets, brushed detail and weathering. Builder stairs gain masonry texture and edge lighting.
- Smoother enlargement improves portals, hazards, traps, flags, direction markers and special backgrounds. Crystal-theme red markers gain a shaded enamel rim while keeping their transparent centers.
- Native, Canvas2D and WebGPU share these changes. Original presentation, collision and steel rules, simulation/audio timing and animation phases are preserved. No additional graphics downloads are required. The maintainer reviewed the development preview.

## 다운로드와 실행 / Download and play

[웹 플레이 / Play online](https://edwardkim.github.io/ohmylemmings/) · [다운로드 / Downloads](https://github.com/edwardkim/ohmylemmings/releases/tag/v0.3.5)

Windows11 x64: ZIP 전체를 풀고 OhMyLemmings.exe를 실행하세요. Windows10은 지원하지 않습니다. macOS13+ universal2: 앱을 Applications로 복사하세요. ad-hoc 서명/미공증입니다. Linux x86_64: SDL2 runtime이 필요합니다. Web ZIP은 HTTP 서버로 제공하세요. 원본120레벨·HD/원본 전환·효과음/BGM·로컬 MP3·점진적 다운로드를 제공합니다.

Windows11 x64: extract the full ZIP and launch OhMyLemmings.exe. Windows10 is unsupported. macOS13+ universal2: copy the app to Applications; it is ad-hoc signed and not notarized. Linux x86_64 requires SDL2. Serve the Web ZIP over HTTP.

지원 브라우저/드라이버와 HTTPS 또는 localhost에서는 WebGPU를 사용하고 실패/장치 손실에는 Canvas2D로 복구합니다. LAN HTTP는 Canvas2D로 실행합니다. 비교에는 `?renderer=canvas2d`를 사용하세요. Native는 SDL 가속 렌더러를 자동 선택하고 실패하면 software로 실행합니다. 표시 목표40FPS는 실제 달성 FPS 보장이 아닙니다.

Supported browsers/drivers use WebGPU on HTTPS or localhost, with Canvas2D fallback. LAN HTTP uses Canvas2D; `?renderer=canvas2d` forces it. Native selects SDL acceleration with software fallback and targets40FPS; this does not guarantee an achieved FPS.

Web의 MP3 선택과 Native F4/Ctrl+O(macOS Cmd+O), 파일 드롭/`--bgm FILE`을 사용할 수 있습니다. 사용자 음악은 업로드하거나 배포하지 않습니다. Local MP3 files remain on your device.

## 대응 소스와 검증 범위 / Source and validation scope

게임 실행에 소스 다운로드는 필요하지 않습니다. 수정·재링크에는 같은 릴리즈의 `ohmylemmings-0.3.5-source.tar.gz`와 패키지 REBUILD.md를 사용하세요. 각 패키지에 소스 URL/크기/SHA-256을 고정하고 SHA256SUMS·RELEASE-MANIFEST.json을 제공합니다.

The optional source archive supports modification/relinking. Packages pin its URL, size and SHA-256; checksums and CI/source provenance accompany the downloads.

#249에서54종540프레임·5재질·4특수 배경,120개 초기 장면과 선택 틱, 대표 저장 해법을 검증했습니다. 이번 배포는 새 FPS 성능 인수나 전체120레벨의 새 완주 인증이 아닙니다. Project code is MIT; third-party notices and original-data rights boundaries remain unchanged. Original game data is included; provenance and notice checks do not establish redistribution permission.
