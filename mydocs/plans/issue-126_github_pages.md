# #126 GitHub Pages 플레이 배포

이슈: https://github.com/edwardkim/ohmylemmings/issues/126

사용자가 Pages 공개 게시를 승인했다. 정식 Release의 Web ZIP을 재빌드하거나 수정하지 않고 검증 후 배포한다. 첫 대상은 v0.1.0이며 주소는 https://edwardkim.github.io/ohmylemmings/ 이다.

- Actions 기반 Pages를 설정하고 수동 태그 지정과 Release 성공 후 재사용 호출을 지원한다. GITHUB_TOKEN으로 게시한 release 이벤트에 의존하지 않는다.
- 최신 공개 정식 릴리즈만 허용한다. GitHub asset digest/크기, RELEASE-MANIFEST, sidecar, 내부 원본28개·대응 소스·라이선스·전체 파일을 검증한다. 배포 출처를 deployment.json에 남긴다.
- Python 합성 TDD, 실제 릴리즈 압축 해제 브라우저 검사, 공개 HTTPS/WASM MIME·전체 파일 해시와 실제 DOM/Canvas/120개 선택·PCM·재시작/전환을 확인한다.
- self-review·CI·PR 병합 후 초기 수동 배포를 실행한다. 완료 시 실제 run/주소/검증 결과를 보고서·이슈에 기록한다.

엔진·자산·성능 정책은 변경하지 않는다. #113의 고성능 호스트 검증은 이월 상태를 유지한다. 공개 태그/asset은 불변이며 원본/생성물을 Git에 추가하지 않는다.
