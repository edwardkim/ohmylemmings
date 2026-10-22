# #126 GitHub Pages 플레이 배포

이슈: https://github.com/edwardkim/ohmylemmings/issues/126

## 변경

공개 정식 Release의 Web ZIP을 그대로 승격한다. tools/pages_package.py가 GitHub asset digest·RELEASE-MANIFEST·sidecar·내부 파일·원본28개·대응 소스/라이선스를 검증하며 초안/시험판/이전 버전은 거부한다. deployment.json은 버전/릴리즈 소스/ZIP SHA를 제공한다. Actions 기반 게시이므로 Jekyll 변환은 없다(.nojekyll은 staging 표식이며 업로드 action의 숨김 파일 제외 대상이다).

pages.yml은 최신 정식 태그를 수동 배포하거나 Release 성공 후 직접 호출한다. GITHUB_TOKEN의 release 이벤트 비전파에 의존하지 않으며, 게시 직전 latest를 다시 확인한다. 전용 concurrency가 배포를 직렬화한다. Pages 환경은 main과 v* 태그만 허용하고 HTTPS를 강제한다.

게임/자산 payload46개는 v0.1.0과 바이트 동일하다. 호스팅 코드 변경 때문에 플레이 버전이나 기존 태그/asset을 바꾸지 않는다. #113 성능 이월과 기존 G11 미완료 판정은 유지한다.

## 검증

- 새 Pages 도구7개 합성 TDD: 구현 전 import 실패 → 정상·불변 승격/초안·시험판·이전 버전·변조·원본/소스 누락·중복/다른 출처·덮어쓰기 거부 PASS.
- make check-python PASS: Python243개와 source/evidence 감사·기존 경계 재검사. 새 JS/CDP checker의 node --check PASS. 게임 Rust 소스를 바꾸지 않아 로컬의 장시간 빌드/성능 반복은 추가하지 않았다. 전체 CI는 PR에서 확인한다.
- actionlint1.7.12로 Pages/Release workflow 구문·context 검사 PASS. 공식 배포 digest와 고정 action commit은 assets/manifests/pages_tools.json에 기록했다.
- 실제 공개 v0.1.0(4,919,566bytes) 다운로드/검증·추출 PASS. ZIP SHA256 03597c78dbf0a46ef3f2cdfe37636f658031032ab22f402b483da2d83cd044f5.
- 새 headless 로컬 서버 /game/ 검사 PASS: 원본120개 선택, RGB 1539ca2afdcc9cfbe4fa5de51036c45b881014ec14a0ab030c191ca41f988c4d, 실제 Canvas·PCM3072샘플/비영3067, 저장 Fun01 입력 성공→Restart/Continue와 fresh 상태/화면 동일, refresh·원본 누락2개·loader 오류8개·소스/라이선스 HTTP. 자동 음향 허용 flag 사용한 기능 검사이며 물리 청취/성능 합격은 뜻하지 않는다.
- 공개 HTTPS는 병합·배포 후 tools/check_pages.cjs로 실제 브라우저 입력 음향·동일 payload/MIME·120개 선택·재시작/전환·refresh를 확인한다. 공개 run/결과는 이슈 완료 기록에 추가한다.

## Self-review

- ZIP 추출 전에 정확한 허용 목록·안전 경로/일반 파일·전체 바이트/소스/원본 해시를 기존 verifier로 검사한다. 외부 metadata의 tag/version/저장소/URL·digest를 교차 확인하고 배포 출력 덮어쓰기를 거부한다.
- 입력 태그는 env 변수로 shell에 전달한다. Pages 게시 권한은 전용 job에 한정하고 환경을 main/v*로 제한했다. 정식 Release 성공 이후에만 자동 호출한다. 모든 신규 외부 action은 commit 고정이다.
- Pages workflow는 숨김 파일을 업로드하지 않지만 payload46개와 공개 provenance에는 영향이 없다. Jekyll을 사용하는 branch 배포는 구성하지 않았다.
- 새 공개 기능 검사는 별도 탭에서 rAF를 통제하며, 종료 후 검사 hook을 제거하고 정상 페이지를 새로 열어 둔다. 전체120개 해법 재검증이나 성능 합격으로 확대 해석하지 않는다.
