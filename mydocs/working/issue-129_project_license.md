# #129 자체 코드 라이선스 정책

이슈: https://github.com/edwardkim/ohmylemmings/issues/129

#115 종료 정리에서 이관한 OPEN 후속이다. 사용자는 라이선스 결정을 별도로 추적하는 권고안을 수용했으며 이번 작업에서 특정 라이선스를 선택하지 않았다.

2026-10-22 사용자의 #129 처리 시작 지시를 받고 main f9303ef 기준 issue-129-license-policy에서 착수했다. 루트 license 미설정, OPL/C oracle LGPL, Lemmix 독자 고지, Lemmings.ts MIT+OPL 부록, SDL2 Zlib와 Cargo metadata의 외부18개 의존성을 확인했다. 소유자가 “mit 로 진행하세요”라고 명시하여 [적용안](../plans/issue-129_project_license.md)에 따라 MIT·metadata·고지/모든 패키지와 웹 안내를 구현했다. [검증 보고서](../reports/issue-129_project_license.md)와 최종 GitHub 기록에서 결과/병합을 추적한다.

루트 LICENSE/워크스페이스와8개 crate에 MIT를 적용하고 OPL LGPL은 유지했다. notice38개·외부 Cargo18개를 기록했고 패키지/root 고지 검증을 추가했다. 새 원본/수동 PC·성능 판정과 구분하며 기존 OPL/SDL 고지·대응 소스/재링크는 유지한다.

완료 조건과 작업 목록은 이슈 본문을 따른다. 기존 v0.1.0 태그/asset은 불변이며 배포물 변경이 필요하면 #124 정책의 이후 버전으로 반영한다. 플레이 공개 승인을 특정 라이선스 선택으로 해석하지 않는다.
