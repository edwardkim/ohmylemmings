# v0.3.5 재질 개선 릴리즈·Pages

담당 [#252](https://github.com/edwardkim/ohmylemmings/issues/252), [버전·게시 정책](issue-124_release_policy.md), [사용자 노트](../releases/issue-252_v0.3.5.md).

메인테이너가 #249 개발 미리보기를 직접 확인하고 다음 절차를 승인했다. 기존 기능의 HD 품질 보완이므로0.3.5 patch로 배포한다. 기준 main은 `974062e3f002a87db36bac6f3454848ccbe60e6f`이며 제품 PR#250/후속main CI7개가 성공했다.

1. #249 시각 인수와 이번 게시 승인을 기록한다. workspace/lock9개를0.3.5로 맞추고 한영 변경/실행 안내를 준비한다.
2. 버전·소스 인덱스·상대 링크를 검사하고 self-review PR의 동일 tree에서 전체 플랫폼 CI7개를 확인한다. 이미 통과한 재질 검증은 역사 범위를 유지한다.
3. main 병합 tree와 CI 산출물·대응 소스를 대조하고 prepare-only를 통과시킨다. annotated `v0.3.5` 태그를 main에 만든다.
4. 기존 게시 workflow로 네 플랫폼·공통 소스·체크섬/계보12파일을 검증·게시하고 같은 Web ZIP을 Pages에 승격한다.
5. 공개 파일/소스 링크·런타임 파일 해시·실제 WebGPU/Canvas 게임을 확인한다. 기존7개 릴리즈/태그를 보존하고 결과/한영 README/인덱스를 마감한다.

기존 태그/asset은 덮어쓰지 않는다. 원화 PNG/다운로드 팩을 추가하지 않으며 새 FPS 인증·전체120레벨 재완주·Apple 공증을 주장하지 않는다. 결과는 [보고서](../reports/issue-252_release_v035.md)에 기록한다.
