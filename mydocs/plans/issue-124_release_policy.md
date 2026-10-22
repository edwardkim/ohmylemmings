# 버전·태그 정책과 첫 Releases 배포

담당 [#124](https://github.com/edwardkim/ohmylemmings/issues/124), 상위 [#115](https://github.com/edwardkim/ohmylemmings/issues/115).

사용자가 토이프로젝트를 GitHub Releases에서 다운로드해 플레이하도록 배포를 승인했고, 먼저 버전/태그 관리 지침 수립을 지시했다. 기준 main3f6eecf 전체7개 CI 성공을 확인했다. self-review로 정책·버전 일치 검사·패키지/CI 하드코딩 제거·검증 산출물의 첫 릴리즈 승격을 진행한다.

Cargo workspace version을 유일한 버전 기준으로 하고 SemVer 기반 정식 vX.Y.Z/시험판 vX.Y.Z-rc.N 태그를 사용한다. 첫 버전0.1.0을 유지한다. 게시 태그/asset은 덮어쓰지 않고 새 버전으로 수정한다. 검증된 동일 소스 트리의 CI 산출물만 릴리즈에 승격하며 태그/빌드 커밋·트리·체크섬을 기록한다. 4플랫폼 전체 게임과 실행 안내·출처/라이선스/대응 소스를 유지한다. #113 성능 이월과 Apple 미공증은 사용자 안내에 구분한다.

[SemVer](https://semver.org/), [GitHub Releases](https://docs.github.com/en/repositories/releasing-projects-on-github/managing-releases-in-a-repository)를 근거로 프로젝트 운영 규칙을 정한다. 토이프로젝트의 첫 플레이 릴리즈는 G11 성능 인증/전체 라이선스 부여를 뜻하지 않는다. 사용자 최신 배포 지시가 기존 공개 릴리즈 보류 문구를 대체한다.

## 버전 규칙

- 단일 기준은 루트 Cargo.toml의 `[workspace.package].version`. 모든9개 crate가 이를 상속하고 Cargo.lock의 로컬 package 버전도 일치해야 한다. 변경 시 해당 값과 lock을 같은 PR에서 갱신하고 `python3 tools/release_version.py check`를 통과시킨다.
- `X.Y.Z`는 정식 플레이 배포, `X.Y.Z-alpha.N`, `-beta.N`, `-rc.N`은 시험판이다. N은1부터 시작한다. 선행0/임의 접미사/빌드 메타데이터(+)는 파일명 혼동을 막기 위해 배포 버전에 사용하지 않는다.
- 호환 버그 수정/작은 배포 수정은 patch, 기능/레벨·플랫폼 추가는 minor. 0.x에서 호환성이 바뀌어도 minor를 올리고 릴리즈 노트에 적는다. 1.0 이후 비호환 변경은 major. 태그를 만들기 전까지 개발 커밋마다 버전을 올리지는 않는다. 첫 배포는 기존0.1.0이다.
- 예: 0.1.0 → 버그 수정0.1.1, 새 기능0.2.0. 시험판은0.2.0-rc.1 → rc.2 →0.2.0. 태그·파일명·PACKAGE-MANIFEST는 전체 버전 일치. Mac 숫자 bundle 필드는 X.Y.Z를 사용하고 전체 접미사는 CFBundleGetInfoString과 archive/manifest에 남긴다.

## 태그·게시 규칙

- main에 병합한 소스에 **annotated tag** `v<workspace-version>`만 사용한다. `git tag -a v0.1.0 <commit> -m 'Oh My Lemmings 0.1.0'` 후 `git push origin v0.1.0`. 태그 이름을 생략한 GitHub 자동 생성은 사용하지 않는다.
- 태그의 소스 트리와 동일한 source tree의 CI7개가 전부 성공해야 한다. feature/PR 검증 산출물도 태그와 완전히 같은 트리이고 각 대응 소스가 현재 태그 내용과 일치할 때 재빌드 없이 승격할 수 있다. RELEASE-MANIFEST에 릴리즈/빌드 commit과 tree/CI/artifact/hash를 모두 남긴다.
- CI/package 경로는 `tools/release_version.py`가 workspace에서 계산한다. 버전 불일치, stale lock, 실패/누락 CI, 다른 트리, 다른 원본/소스, 빠진 플랫폼, 변조된 업로드는 게시 전에 실패한다.
- 버전별 변경/실행 안내를 `mydocs/releases/issue-<번호>_v<version>.md`에 작성한다. 4플랫폼 전체 게임·개별/전체 체크섬·대응 소스·출처/라이선스·실행 안내를 모두 준비한다.
- tag push의 Release workflow가 성공한 같은 트리의 CI 산출물을 내려받아 검증하고 **draft**에 업로드한다. 업로드 바이트를 다시 내려받아 확인한 뒤 publish한다. 시험판은 pre-release이며 정식 버전만 latest다. 공개 URL도 인증 없이 다운로드해 검증한다. 웹 ZIP은 HTTP 서버용 정적 파일이며 Releases가 게임 페이지를 호스팅하는 것은 아니다.
- 게시된 태그를 이동/삭제/강제 push하거나 asset을 교체(`--clobber`)하지 않는다. 수정은 새 patch/다음 시험판. 누락/실패 draft는 공개하지 않고 이슈에 원인을 남긴다. 태그와 파일이 완성되기 전에는 publish하지 않는다.
- 사용자 승인은 이번 첫 릴리즈 게시까지 포함한다. 이후 릴리즈는 사용자가 배포를 지시하거나 승인한 범위에서 준비·게시한다. 태그 workflow는 그 결정의 실행 수단이며 모든 커밋을 자동 공개하는 정책은 아니다.

## 검증·운영 명령

`python3 tools/release_version.py check --tag v0.1.0`으로 정책을 검사한다. CI는 같은 검사를 항상 수행한다. 수동 복구/검토는 태그 checkout에서 `python3 tools/publish_release.py prepare --tag v0.1.0 --run <성공CI> --directory assets/local/release`로 준비하고, annotated tag를 원격에 올린 뒤 `publish --directory assets/local/release`로 수행한다. 이미 있는 릴리즈/asset을 덮어쓰지 않는다. 일반 절차는 main CI 또는 같은 트리 PR CI 성공 확인 → tag → Release workflow → 공개 파일 확인이다.

공식 근거: [Apple 숫자 버전](https://developer.apple.com/documentation/bundleresources/information-property-list/cfbundleshortversionstring). 최신 사용자 지시에 따라 릴리즈 기능을 플레이 배포로 사용하며 성능 인증/공증/전체 라이선스 결정을 새로 주장하지 않는다.

자동 게시가 실패해 draft가 남으면 실패 원인을 이슈에 기록하고 검증 완료 전에는 공개하지 않는다. 공개하지 않은 draft는 필요한 경우 삭제 후 새 작업 디렉터리로 다시 준비할 수 있으나 원격 태그를 재지정하거나 공개 asset을 덮어쓰지 않는다. 첫 tag push 전에도 prepare를 실행해 전체 산출물과 소스 일치 검사를 확인한다.
