# #126 GitHub Pages 진행 상태

이슈: https://github.com/edwardkim/ohmylemmings/issues/126

- main 75a432f 기준 별도 worktree issue-126-github-pages에서 착수했다. Pages 미설정, 저장소 public/admin과 v0.1.0 공개 Web ZIP을 확인했다.
- 계획: [배포 계약](../plans/issue-126_github_pages.md). 도구7개 red→green, Python243·기계 감사와 실제 공개 v0.1.0 추출 /game/의120개 선택·Canvas·PCM·재시작/전환 PASS. 엔진/게임 payload 변경 없음.
- Pages build_type=workflow·HTTPS, github-pages 환경 main branch/v* tag 정책을 확인했다. PR #127을 실제 병합(3594bb0)하고 Pages run39766461582 SUCCESS.
- 공개 https://edwardkim.github.io/ohmylemmings/ 의47개 SHA·WASM MIME·원본120개 선택/Canvas·실제 클릭 PCM·재시작/전환·refresh PASS. [보고서](../reports/issue-126_github_pages.md)와 공개 증거 manifest에 기록했다.
- 사용자 요청 Pages 플레이 배지와 완료 기록을 후속 PR로 반영한다. 전체 플랫폼 CI의 최종 상태는 이슈 완료 기록에서 구분한다.
- 기존 v0.1.0 Web SHA256: 03597c78dbf0a46ef3f2cdfe37636f658031032ab22f402b483da2d83cd044f5. 이 바이트를 그대로 승격한다.
