# #187 HD 패키징·최종 인수

이슈: https://github.com/edwardkim/ohmylemmings/issues/187 · 상위 #137

Windows11/Linux/macOS arm64·Intel/Web의 전체 HD 카탈로그·글꼴·출처/고지/소스와 기본 HD 실행, 원본 실행/전환을 구현한다. 추출 후 다른 CWD·공백/한글 경로, 누락/손상·오프라인 실행·테마/특수 전환, 원본 규칙/입력/사운드 동일성을 TDD·실제 설치/플랫폼 CI·self-review·실제 병합으로 검증한다. 기존 자산/규칙 증거와 현재 제작 목록을 정합화하고 최종 인수 범위를 명시한다. 공개 v0.1.0/Pages는 교체하지 않는다. #113 NOT READY·Mac ad-hoc/미공증·Windows11 지원을 유지한다. 사용자19222/8080/preview-172와 파일은 보존한다. BGM #165는 #137 마감 이후다.

## 완료

PR #188 실제 merge `6a7c68dc61b063f915aae15476dc3c61bd75a51b`, head `6243132b5fd094d4d49804e075bd9443e024d18a`의 [CI39976023448](https://github.com/edwardkim/ohmylemmings/actions/runs/39976023448)에서7개 모두 SUCCESS. CI checkout `96f74ff2e6fce579952223e6ddbe6abb1d2bd263`과 실제 병합 tree `ef01904c2cc8d3884303499d79dc9b3e5479ecfc`가 같다. [검증·한계](../reports/issue-187_hd_release.md). HD-06과 상위 #137의 현재 게임 리마스터 범위를 완료했고, 이후 BGM #165를 처리한다. 새 공개 릴리즈/Pages·60fps·Mac 공증 판정은 포함하지 않는다.
