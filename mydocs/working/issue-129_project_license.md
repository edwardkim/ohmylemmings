# #129 자체 코드 라이선스 정책

이슈: https://github.com/edwardkim/ohmylemmings/issues/129

#115 종료 정리에서 이관한 후속이다. 등록 당시에는 정책이 미정이었고, 아래 착수 중 소유자가 MIT를 명시적으로 선택했다.

2026-10-22 사용자의 #129 처리 시작 지시를 받고 main f9303ef 기준 issue-129-license-policy에서 착수했다. 루트 license 미설정, OPL/C oracle LGPL, Lemmix 독자 고지, Lemmings.ts MIT+OPL 부록, SDL2 Zlib와 Cargo metadata의 외부18개 의존성을 확인했다. 소유자가 “mit 로 진행하세요”라고 명시하여 [적용안](../plans/issue-129_project_license.md)에 따라 MIT·metadata·고지/모든 패키지와 웹 안내를 구현했다. [검증 보고서](../reports/issue-129_project_license.md)와 최종 GitHub 기록에서 결과/병합을 추적한다.

루트 LICENSE/워크스페이스와8개 crate에 MIT를 적용하고 OPL LGPL은 유지했다. notice38개·외부 Cargo18개를 기록했고 패키지/root 고지 검증을 추가했다. 새 원본/수동 PC·성능 판정과 구분하며 기존 OPL/SDL 고지·대응 소스/재링크는 유지한다.

구현 PR #132 실제 병합598de80·검토 tree 일치와 PR CI39772657853의7개 SUCCESS를 확인했다. 네 실제 CI 패키지의 고지/원본/전체 해시 및 source793개씩 Git blob 동일성 PASS. GitHub도 MIT로 표시한다. 코드 변경 없는 최종 문서 정리/상위 추적·이슈 종료 결과는 GitHub #129에 기록한다.

완료 조건과 작업 목록, 정확한 CI/PR/병합 결과는 이슈 최종 기록을 따른다. 기존 v0.1.0 태그/asset은 불변이며 배포물 변경이 필요하면 #124 정책의 이후 버전으로 반영한다. 이번 MIT 선택은 소유자의 별도 명시 지시에 근거한다.
