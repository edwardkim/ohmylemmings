# v0.3.3 정식 릴리즈·Pages 완료

담당 [#230](https://github.com/edwardkim/ohmylemmings/issues/230). [릴리즈 노트](../releases/issue-230_v0.3.3.md), [배포 최적화](issue-226_package_size.md).

[GitHub Release](https://github.com/edwardkim/ohmylemmings/releases/tag/v0.3.3)와 [공개 게임](https://edwardkim.github.io/ohmylemmings/) 게시를 완료했다. #226의 중복 소스 분리·미사용 역사 자산 제외·오래된 Web HD 생성 파일 정리를 포함한다. 원본120레벨, HD 그래픽, BGM/로컬 MP3 및 기존 플레이 기능은 유지한다.

## 소스와 검증 계보

| 항목 | 결과 |
|---|---|
| 준비 PR | [#231](https://github.com/edwardkim/ohmylemmings/pull/231) |
| PR head | `3914604ebc02566540d7f5e9cd2e8f75431c1ef5` |
| 실제 CI checkout | `8ee1c0fe78c79be19d8383787a74025ba1c81ac5` |
| main/tag commit | `f91373ceaff61aa29427592b84e1042f960992a7` |
| 공통 source tree | `d851c456f3fa5a4a0636da9a3d1b87aff9a2e34c` |
| annotated tag object | `39533bd74b762f4c9a3806fba7a8b0a1d78cd0c8` |
| 플랫폼 CI7개 | [40146401417](https://github.com/edwardkim/ohmylemmings/actions/runs/40146401417), 모두 SUCCESS |
| 태그 전 사전 검증 | [40147448393](https://github.com/edwardkim/ohmylemmings/actions/runs/40147448393), SUCCESS |
| 정식 게시·Pages | [40147862502](https://github.com/edwardkim/ohmylemmings/actions/runs/40147862502), publish/Pages SUCCESS |

버전/lock 검사와 Python339개 테스트 통과. 동일 tree CI의 Windows·Linux·macOS Apple Silicon/Intel·Web 실제 패키지 설치/실행 증거를 사용했다. 중복 CI만 취소하고 PR CI7개 전체 결과를 검증했다. 병합 커밋 skip-ci를 사용했으며 annotated tag 게시 후 해당 태그에서 Release workflow를 명시적으로 dispatch했다. 사전 검증과 정식 게시 RELEASE-MANIFEST 내용은 동일하다.

## 게시 파일

네 실행 패키지, 공통 대응 소스, PLAY.md, 개별/전체 체크섬과 RELEASE-MANIFEST의12파일을 공개했다. 대응 소스836파일의 내용과 각 패키지의 외부 소스 URL/크기/SHA를 검증했다. 게임 실행에는 소스 다운로드가 필요하지 않다.

| 파일 | 바이트 | SHA-256 |
|---|---:|---|
| `ohmylemmings-0.3.3-linux-x86_64.tar.gz` | 98,428,349 | `966dd3aa31e28aa3ac87828b9611cf8f2c43248e195f452444294f81b30c6e0d` |
| `ohmylemmings-0.3.3-macos-universal2.zip` | 100,797,390 | `6dc0ef44a9946d95519381a542e1d9f34678857c71e1d51f79acb879e310bc3c` |
| `ohmylemmings-0.3.3-source.tar.gz` | 85,992,629 | `a12839c8c66ab50dcd91e0a909e4111148480c5672470bbca17334c3bbae521e` |
| `ohmylemmings-0.3.3-web.zip` | 97,565,086 | `2d372e77321232fc0258801c4053636fee43cfb4ec95b296ec6c6ff6f2d1624e` |
| `ohmylemmings-0.3.3-windows-x86_64.zip` | 98,869,209 | `0e3103a01d85dc2a106dcce18d8d901fbb2889102e8b55a97f191f245bb9cd3e` |

Web ZIP은 공개v0.3.2의218.03MB에서97.57MB로55.3% 감소했다. 공통 소스는120.50MB에서85.99MB로 줄었다. 소스는 과거에도 자동 플레이 다운로드가 아니므로 첫 스테이지 로딩 용량이 같은 비율로 줄었다는 의미는 아니다.

## 공개 인수

- Release workflow가 초안 업로드 파일을 재다운로드해 검증하고, 공개 후12파일을 인증 없는 다운로드 URL로 읽어 내용 일치를 확인했다.
- Pages deployment.json의0.3.3 버전·태그·commit/tree·Web ZIP SHA가 공개 RELEASE-MANIFEST와 일치한다. 공개 package manifest의107개 실행 파일 전체를 읽어 크기/해시와 WASM MIME을 검증했다.
- 대응 소스는 Pages에 중복 포함되지 않는다. audio-source.json·라이선스 페이지가 공개0.3.3 소스 다운로드를 가리키며, URL·크기·해시 및 공개 접근을 확인했다.
- 공개 Fun01에서 기술8개·배정·일시정지·미니맵·재시작·전체화면 검사와664틱/665상태·10구출 Won을 통과했다. HD/원본 모드 시뮬레이션 동일, console 오류0. 이는 전체레벨 수동 플레이나 FPS 성능 인증이 아니다.
- 표시형 QA 브라우저는 X 서버 연결 실패로 시작하지 못했다. 별도의 headless Chromium으로 동일 검사를 수행해 통과했고 해당 QA 브라우저는 종료했다. 사용자19222/8080과 기존 브라우저/음악은 변경하지 않았다.
- 기존5개 릴리즈의 본문·게시 시각·asset ID/이름/크기/digest/시각과 기존 태그를 대조해 보존을 확인했다. 다운로드 횟수 같은 변동성 통계는 비교에서 제외했다.

개인 증거는 ignored `assets/local/issue-230-release/`에 보관한다. 공개 CI/Release workflow와 `release-publication` artifact에 게시 계보가 남는다. #113의 남은 성능 인수 및 원본 자산 권리 경계는 유지한다. #230의 릴리즈·Pages 요구를 완료했다.
