# #191 리마스터 v0.2.0 버전 적용

이슈: https://github.com/edwardkim/ohmylemmings/issues/191

2026-10-24 메인테이너가 리마스터 버전을0.2.0으로 지정했다. #137/#187의 HD 리마스터와 진행 중인 #165 BGM 수정을 포함하는 다음 버전으로 workspace 및9개 로컬 Cargo.lock 항목을0.2.0으로 맞췄다. Cargo가 lock을 갱신한 뒤 외부 dependency의 전체 기록이 기존과 동일함을 확인했다. 게임/자산/음원 코드의 추가 수정은 없다.

[0.2.0 버전별 노트](../releases/issue-191_v0.2.0.md)를 작성했고 `python3 tools/release_version.py check --tag v0.2.0`으로 workspace·lock·태그명·노트의 일치를 검증했다. Linux/Windows/Web 아카이브 이름과 Mac 숫자 bundle은 공통 버전 함수에서 파생되므로 실제0.2.0 CI 패키지의 새 설치/소스/manifest 검사까지 확인한다. BGM PR190을 먼저 병합하고, 같은 수정이 포함된 이 버전 PR을 후속으로 병합한다.

기존0.1.0의 릴리즈/태그/파일/출시 노트와 현재 공개 Pages는 변경하지 않는다. 새 릴리즈 노트는 **후보 준비 중/미게시**로 표시하고 아직 없는 공개 다운로드 링크를 만들지 않았다.0.2.0 태그 생성과 공개 승격은 버전 적용/후보 검증과 별도 절차다. #113 NOT READY·Mac ad-hoc/미공증·Windows11 지원을 유지한다.

CI7개·실제 병합과0.2.0 전체 패키지 후보 검증을 완료했다. 이번 버전 변경을 새 스테이지 완주/기존 성능 수치/실제 모든 지원 OS의 청취 판정으로 확대하지 않는다.

## 완료

버전 [PR #192](https://github.com/edwardkim/ohmylemmings/pull/192) 실제 merge `c7da0071cd8d056bea4bad83fa912b70be407e60`. head `232019603a36d487149c1cd9b8f93732c3111b15`의 [CI39984622798](https://github.com/edwardkim/ohmylemmings/actions/runs/39984622798)에서7개 모두 SUCCESS. CI checkout `d405a53a140de2c6ded78ea5c1c1a0be8a92a231`/실제 merge tree `daac94826be9b8fd15c084eb364dd06d00635c4e`가 같다.

BGM PR190을 먼저 실제 병합한 뒤 이 PR을 병합했다. workspace/9개 lock 항목·버전별 노트·Linux/Windows/macOS/Web archive/manifest·Mac 숫자 bundle은0.2.0이다.0.2.0의 각 CI job이 새 압축본의 설치·출처/소스·해시 검사를 통과했다. 로컬30버전/패키지 검사·321문서 링크·651소스 연결/라이선스 감사를 확인했다. 외부 dependency/게임/HD/음원 코드의 추가 변경은 없다.

#191의 버전 적용/시험 후보 준비를 완료한다. 공개용 노트 최종화·annotated tag·Releases 승격/공개 다운로드·Pages 전환은 별도 절차이며 아직 수행하지 않았다. 기존v0.1.0 태그/asset/노트/Pages는 그대로다. 메인테이너의 실제 청취 응답은 아직 없다. 디지털/장치 검증 및 제품 수정은 완료했으며 #165는 청취 확인 대기로 OPEN을 유지한다. 버전0.2.0 선택을 청취 정상 판정으로 간주하지 않았다.
