# #165 효과음만 들리고 BGM이 들리지 않는 현상 조사

이슈: https://github.com/edwardkim/ohmylemmings/issues/165

메인테이너가 개발 Web, Native, 배포 패키지 모두에서 효과음은 들리지만 BGM이 들리지 않는다고 확인했다. #163과 별도 worktree에서 공통 음악 시작 이벤트→AdLib 음악 채널→PCM 합성→플랫폼 출력 순으로 조사한다. 원본 ADLIB 데이터는 ignored 경로만 사용하고 공개 문서에는 해시·측정치만 기록한다. 기존 비영 PCM 검사가 효과음만으로도 통과할 가능성을 구분하며, BGM 단독·장시간 지속 출력과 효과음 후 음악 지속을 확인한다.

1. 원본 실행 설정·음악 시작 이벤트와 음악 전용 PCM 재현.
2. 원인 확정 및 원본 드라이버 기준 대조. 필요한 수정은 실패 테스트부터 진행.
3. Native/Web 공통 회귀·문서/출처 감사·CI와 결과 보고.

상태: 공통 OPL 즉시 쓰기의 key-off/on 재시작 손실을 원본60초·원본 없는10음표·C buffered 대조로 확인했다. [원인/재현/검증 누락](../reports/issue-165_bgm.md)을 기록했다. #163은 병합/준비/종료 완료. #165 제품 수정은 아직 적용하지 않았으며 후속으로 OPEN 유지.

2026-10-23 우선순위 변경: 메인테이너 지시로2D 리마스터 #137 완료 후 제품 수정에 착수한다. 원인 조사는 완료 상태로 보존하고 #165는 OPEN/후속 대기로 유지한다. 현재 BGM 구현 작업은 진행하지 않는다.

## 제품 수정 재개 (2026-10-24)

#137은 #187/PR188·189와 CI7개로 마감했다. 메인테이너의 다음 진행 승인에 따라 기준 main `c1bbb3c4154766bf0f7aff884cfaa8b9f7849c67`에서 공통 OPL buffered delivery를 TDD로 연결한다. 반복 음표·버퍼 한계/복제·분할/continuation·고정 C buffered 대조, 원본 장기 음악/효과음 병행과 Native/WASM/장치 출력을 검증한다. 물리 청취 결과는 디지털 측정과 구분한다. 기존 조사 worktree/사용자19222·8080/preview-172·파일을 보존하며 새 공개 배포는 하지 않는다.

## 제품 수정 검증 마감

BGM [PR #190](https://github.com/edwardkim/ohmylemmings/pull/190) 실제 merge `9a0ab3bb4e1c4607ee3500445e1cc3ceedc911f7`. head `95c2e221acff5d62f530e312231b8847db22b484`의 [CI39982379823](https://github.com/edwardkim/ohmylemmings/actions/runs/39982379823)에서7개 모두 SUCCESS. CI checkout `6c571f10202c3897f09d965a4df6a26567ef5b62`/실제 merge tree `25a01ea4e47454f856ce457a1a9b4b921e8bffb0`가 같다. 버전 [PR #192](https://github.com/edwardkim/ohmylemmings/pull/192) 실제 merge `c7da0071cd8d056bea4bad83fa912b70be407e60`. head `232019603a36d487149c1cd9b8f93732c3111b15`의 [CI39984622798](https://github.com/edwardkim/ohmylemmings/actions/runs/39984622798)에서7개 모두 SUCCESS. CI checkout `d405a53a140de2c6ded78ea5c1c1a0be8a92a231`/실제 merge tree `daac94826be9b8fd15c084eb364dd06d00635c4e`가 같다.

메인테이너의 실제 청취 응답은 아직 없다. 디지털/장치 검증 및 제품 수정은 완료했으며 #165는 청취 확인 대기로 OPEN을 유지한다. 버전0.2.0 선택을 청취 정상 판정으로 간주하지 않았다. [결과·범위](../reports/issue-165_bgm.md). #113 NOT READY와 기존 공개 배포/OS 정책은 유지한다.
