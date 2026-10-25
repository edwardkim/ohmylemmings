# Oh My Lemmings 0.3.3 — 가벼운 다운로드 / Smaller downloads

담당 이슈: https://github.com/edwardkim/ohmylemmings/issues/230

## 변경 사항 / Changes

- 실행 패키지에서 중복 대응 소스를 분리하고, 공통 소스 다운로드 하나로 제공합니다 (#226). Web ZIP은 이전 약218MB에서 약98MB로 줄어듭니다. 첫 스테이지 로딩 용량 감소율을 의미하지 않습니다.
- 배포 소스에서 사용하지 않는 과거 미술 시안·미리보기·문서·측정 로그444개를 제외하고, 오래된 Web HD 생성 파일을 정리합니다. 현재 게임 그래픽·120레벨·BGM·로컬 MP3·효과음은 유지합니다.
- Runtime packages now share one separate corresponding-source download. The Web ZIP is approximately98MB instead of218MB; this is package size, not initial gameplay traffic. Unused historical assets, documentation and logs are excluded from source delivery. Gameplay assets and audio are preserved.

## 다운로드와 실행 / Download and play

[웹 플레이 / Play online](https://edwardkim.github.io/ohmylemmings/) · [다운로드 / Downloads](https://github.com/edwardkim/ohmylemmings/releases/tag/v0.3.3)

Windows11 x64: ZIP을 전부 풀고 OhMyLemmings.exe를 실행하세요. Windows10은 지원하지 않습니다. macOS13+ universal2: 앱을 Applications로 복사하세요. ad-hoc 서명/미공증이며 OS의 실행 허용이 필요할 수 있습니다. Linux x86_64: SDL2 runtime이 필요합니다. Web ZIP은 HTTP 서버로 제공하세요.

Windows11 x64: extract the full ZIP and launch OhMyLemmings.exe. Windows10 is unsupported. macOS13+ universal2: copy the app to Applications; the app is ad-hoc signed and not notarized. Linux x86_64 requires SDL2. Serve the Web ZIP through HTTP.

MP3 파일 선택·음량·단축키는 [0.3.1 실행 안내](https://github.com/edwardkim/ohmylemmings/blob/v0.3.1/mydocs/releases/issue-208_v0.3.1.md)를 따릅니다. 사용자 MP3는 서버에 업로드하거나 배포에 포함하지 않습니다. Local MP3 files remain on your device.

## 대응 소스와 검증 / Corresponding source and verification

게임 실행에는 별도 소스 다운로드가 필요하지 않습니다. 수정·재링크하려면 같은 릴리즈의 `ohmylemmings-0.3.3-source.tar.gz`를 받고 각 패키지 REBUILD.md를 따르세요. 소스 URL·크기·SHA-256은 패키지에 고정됩니다. SHA256SUMS와 RELEASE-MANIFEST.json은 게시 파일과 CI/소스 계보를 기록합니다.

The source archive is optional for playing and provided separately for modification/relinking. Download `ohmylemmings-0.3.3-source.tar.gz` from the same release and follow REBUILD.md. Packages include its URL, size and SHA-256; release checksums and provenance are published alongside them.

자체 코드는 MIT이며 제3자 라이선스·원본 게임 데이터 고지를 유지합니다. 원본 자산의 재배포 권한 확보를 주장하지 않습니다. #113 성능/메모리 잔여 검증은 계속합니다. Project code is MIT; third-party notices and original-data rights boundaries remain unchanged. Performance qualification remains pending under #113.
