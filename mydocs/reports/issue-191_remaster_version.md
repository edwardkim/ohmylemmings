# #191 리마스터 v0.2.0 버전 적용

이슈: https://github.com/edwardkim/ohmylemmings/issues/191

2026-10-24 메인테이너가 리마스터 버전을0.2.0으로 지정했다. #137/#187의 HD 리마스터와 진행 중인 #165 BGM 수정을 포함하는 다음 버전으로 workspace 및9개 로컬 Cargo.lock 항목을0.2.0으로 맞췄다. Cargo가 lock을 갱신한 뒤 외부 dependency의 전체 기록이 기존과 동일함을 확인했다. 게임/자산/음원 코드의 추가 수정은 없다.

[0.2.0 버전별 노트](../releases/issue-191_v0.2.0.md)를 작성했고 `python3 tools/release_version.py check --tag v0.2.0`으로 workspace·lock·태그명·노트의 일치를 검증했다. Linux/Windows/Web 아카이브 이름과 Mac 숫자 bundle은 공통 버전 함수에서 파생되므로 실제0.2.0 CI 패키지의 새 설치/소스/manifest 검사까지 확인한다. BGM PR190을 먼저 병합하고, 같은 수정이 포함된 이 버전 PR을 후속으로 병합한다.

기존0.1.0의 릴리즈/태그/파일/출시 노트와 현재 공개 Pages는 변경하지 않는다. 새 릴리즈 노트는 **후보 준비 중/미게시**로 표시하고 아직 없는 공개 다운로드 링크를 만들지 않았다.0.2.0 태그 생성과 공개 승격은 버전 적용/후보 검증과 별도 절차다. #113 NOT READY·Mac ad-hoc/미공증·Windows11 지원을 유지한다.

CI/실제 병합·main 버전·후보 파일명 확인 결과는 검증 후 확정한다. 이번 버전 변경을 새 스테이지 완주/기존 성능 수치/실제 모든 지원 OS의 청취 판정으로 확대하지 않는다.
