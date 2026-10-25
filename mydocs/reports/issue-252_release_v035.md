# v0.3.5 지형·지물 개선 배포 완료

담당 [#252](https://github.com/edwardkim/ohmylemmings/issues/252), [계획](../plans/issue-252_release_v035.md), [한영 릴리즈 노트](../releases/issue-252_v0.3.5.md).

[정식 릴리즈](https://github.com/edwardkim/ohmylemmings/releases/tag/v0.3.5)와 [Pages 웹 플레이](https://edwardkim.github.io/ohmylemmings/)를 게시했다. 릴리즈 게시 시각은2026-10-25 19:09:39 KST다. Windows/macOS/Linux/Web 전체 게임4패키지, 공통 대응 소스·PLAY.md·개별/전체 체크섬·계보의12파일을 제공한다.

## 인수와 준비

메인테이너가 [#249 개발 미리보기](issue-249_hd_material_quality.md)를 확인하고 다음 절차를 승인했다. 기존 기능의 재질 품질 보완이므로 #124 정책에 따라 patch0.3.5로 정했다. 기준 main은 `974062e3f002a87db36bac6f3454848ccbe60e6f`다.

강철5팔레트·건설 계단,54종540지물 프레임/특수4배경의 확대 보간과 결정 표지 조명을 포함한다. Native/Canvas2D/WebGPU가 공유한다. 제품 구현/외부 의존성/원화 PNG/팩은 #249 이후 바꾸지 않았으며 workspace/lock9개 버전만0.3.5로 맞췄다. 상세 재질 검증과 제한은 #249 보고서에 보존한다.

로컬 버전/태그 일치·COMP/G11 소스 인덱스·라이선스·릴리즈/소스 분리/Pages 관련25개 테스트와 문서 상대 링크/diff를 통과했다. 현재 소스 핀은 이전 핀과 역사 판정을 보존했다. 게시 전까지 README에 실제 공개0.3.4와0.3.5 준비 안내를 구분했고, 게시 성공 후 한영 README/문서 인덱스에0.3.5를 반영했다.

## 소스·CI·게시 계보

| 항목 | 값 |
|---|---|
| 준비 PR | [#253](https://github.com/edwardkim/ohmylemmings/pull/253), self-review/병합 완료 |
| PR head | `9fbc8aa436e13f8d56b93031abe7c5b8f0b48335` |
| 실제 CI checkout | `1b2589e9fc2f4d8ce2a738f558b7d6e5a5c177fa` |
| main/tag commit | `b00829113d31efcf09c61706acdc783de1d47ec6` |
| 공통 source tree | `db06e42a012af921e0251fc56c04997dd289c05c` |
| annotated tag object | `bdad659aa2071dda5b99a1093406872c0bc1dcc9` |
| 플랫폼 CI7개 | [40181601182](https://github.com/edwardkim/ohmylemmings/actions/runs/40181601182), 전부 SUCCESS |
| prepare-only | [40182817837](https://github.com/edwardkim/ohmylemmings/actions/runs/40182817837), SUCCESS |
| 정식 게시·Pages | [40183187279](https://github.com/edwardkim/ohmylemmings/actions/runs/40183187279), publish/Pages SUCCESS |

Windows 전체 게임, Apple Silicon/Intel 동일 universal 패키지 실행, Linux 전체 검사/설치, Web 전체 검사/패키지, Python, WASM의7개 CI가 모두 성공했다. 중복 feature push CI만 취소했다. 병합 후 [main CI40182798043](https://github.com/edwardkim/ohmylemmings/actions/runs/40182798043)도7개 작업 전부 SUCCESS였다.

CI checkout/head/main/tag의 tree 일치를 확인했다. 네 플랫폼 각각의 대응 소스849개를 같은 릴리즈 소스와 대조했다. 사전 검증과 정식 게시의 RELEASE-MANIFEST가 완전히 동일하다. 기존 태그/asset은 덮어쓰지 않았다.

## 공개 검증

- workflow에서 초안 업로드를 재다운로드하고 공개12파일을 인증 없이 받아 바이트를 대조했다.
- Pages0.3.5의 source commit/tree/Web ZIP SHA가 RELEASE-MANIFEST와 일치한다. 공개 실행 파일109개 전체 크기·SHA-256과 WASM MIME, 대응 소스 URL/크기/해시·공개 HEAD 접근을 검증했다. 배포 메타데이터를 포함하면111파일이다.
- 캐시 우회 query뿐 아니라 브라우저가 쓰는 기본 index/WASM URL도 같은 바이트인지 확인했다.
- 일반 Wayland Chromium의 공개 WebGPU에서 버전0.3.5·HD/원본 전환·8기술 선택 키·Pause/Resume·1920×1080 전체화면과 입력/표시 위치 일치를 확인했다. Fun01의 저장 해법으로660틱/10구출 Won, 다음 Fun02 이동, Mayhem08 80틱과 새 철판 표시를 확인했다.
- Mayhem08 Pause 상태에서 GPU device를 파괴해 상태 보존·Canvas2D 복구·이후 입력 복귀를 확인했다. 브라우저 예외0이었다.
- 공개 `?renderer=canvas2d` 시작에서도 같은 조작/전체화면·Fun01 10구출·다음 Fun02·Mayhem08 80틱/재질 표시를 통과했다. 브라우저 예외0이었다.
- 기존7개 릴리즈의 본문·게시 시각·asset ID/이름/크기/digest/시각과 기존 태그를 보존한 것을 확인했다. 다운로드 횟수 같은 변동성 통계는 비교하지 않았다.

이 플레이 검사는 저장 해법과 명시적 틱 진행을 사용했다. 새 FPS 성능 인수·전체120레벨의 새 완주 인증이나 새 Apple 공증을 뜻하지 않는다. GPU QA 옵션은 임시 브라우저 프로필에만 사용했으며 사용자19222/8080·개인 MP3 환경은 유지했다. 원본 레벨 파생 캡처와 원시 자료는 ignored `assets/local/issue-252/`에만 보존한다.

주요 증거: `ci-final.json`, `preflight/RELEASE-MANIFEST.json`, `publication/RELEASE-MANIFEST.json`, `publication/published.json`, `public-verification.json`, `public-webgpu/summary.json`, `public-webgpu/public-mayhem08.png`, `public-canvas2d/summary.json`. 공개 CI/Release workflow와 publication artifact에도 계보가 남는다.

## 게시 파일

| 파일 | 바이트 | SHA-256 |
|---|---:|---|
| `ohmylemmings-0.3.5-linux-x86_64.tar.gz` | 98,444,251 | `d90f9e265d804a76459cb6e85b31ee4f599b2fd5133154750b94d3a91a559424` |
| `ohmylemmings-0.3.5-macos-universal2.zip` | 100,825,601 | `51c557e7b0e57883d5d0f04c2832ccd28d47ab97e03b31ddcb99952951ff74df` |
| `ohmylemmings-0.3.5-source.tar.gz` | 86,023,910 | `839a63b8d363b46c89f221a05996071290f3d07a560177f3915ba2d056af7f30` |
| `ohmylemmings-0.3.5-web.zip` | 97,571,726 | `45b8f8374b4639e5733fe9355b74a24c0af71e728fa10483ffa63cd5ef8843bd` |
| `ohmylemmings-0.3.5-windows-x86_64.zip` | 98,883,171 | `11057fa81de6795ecfbb64139f86f1496d52ce21c394d241f7c50544c8a9ab52` |

Web ZIP은0.3.4의97,570,060바이트에서97,571,726바이트로1,666바이트 증가했다. 추가 그래픽 다운로드는 없으며 점진적 다운로드/다음 레벨 백그라운드 준비를 유지한다. 실행에 대응 소스 다운로드는 필요하지 않다.
