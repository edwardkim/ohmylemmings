# 버전·태그 정책과 첫 Releases 배포

담당 [#124](https://github.com/edwardkim/ohmylemmings/issues/124), 상위 [#115](https://github.com/edwardkim/ohmylemmings/issues/115).

사용자가 토이프로젝트를 GitHub Releases에서 다운로드해 플레이하도록 배포를 승인했고, 먼저 버전/태그 관리 지침 수립을 지시했다. 기준 main3f6eecf 전체7개 CI 성공을 확인했다. self-review로 정책·버전 일치 검사·패키지/CI 하드코딩 제거·검증 산출물의 첫 릴리즈 승격을 진행한다.

Cargo workspace version을 유일한 버전 기준으로 하고 SemVer 기반 정식 vX.Y.Z/시험판 vX.Y.Z-rc.N 태그를 사용한다. 첫 버전0.1.0을 유지한다. 게시 태그/asset은 덮어쓰지 않고 새 버전으로 수정한다. 검증된 동일 소스 트리의 CI 산출물만 릴리즈에 승격하며 태그/빌드 커밋·트리·체크섬을 기록한다. 4플랫폼 전체 게임과 실행 안내·출처/라이선스/대응 소스를 유지한다. #113 성능 이월과 Apple 미공증은 사용자 안내에 구분한다.

[SemVer](https://semver.org/), [GitHub Releases](https://docs.github.com/en/repositories/releasing-projects-on-github/managing-releases-in-a-repository)를 근거로 프로젝트 운영 규칙을 정한다. 토이프로젝트의 첫 플레이 릴리즈는 G11 성능 인증/전체 라이선스 부여를 뜻하지 않는다. 사용자 최신 배포 지시가 기존 공개 릴리즈 보류 문구를 대체한다.

구현/로컬 검사: version5·promotion/publish6·Mac preview1 red→green 포함 전체 Python236/정책/감사 PASS. CI·Makefile의0.1.0 경로를 workspace 계산으로 교체했고 태그 Release workflow는 검증 CI 산출물을 draft→다운로드 검증→publish→익명 다운로드 확인으로 승격한다. 정확한 source/CI/PR/태그·공개 주소는 #124에 이어서 기록한다.
