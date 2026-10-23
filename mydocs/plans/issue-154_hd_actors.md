# #154 HD-05a 캐릭터 전체 프레임

이슈: https://github.com/edwardkim/ohmylemmings/issues/154 · 상위 [#137](issue-137_2d_remaster.md)

main e7c2870·#154 OPEN/번호·URL을 확인했다. 기존7슬롯81프레임에 남은21슬롯256프레임을 더해28/337 캐릭터 대응을 완성한다. 테마/시설/효과/UI 확장은 HD-05 후속이며 전체 리마스터 완료와 구분한다.

1. built-in image_gen으로 이동/등반·굴착 도구/우산·Blocker/완료/사망의 분리 atlas를 생성하고 실제 투명 여백·방향·위상·발 기준점을 검토한다. 생성 PNG를 보존하고 프롬프트·출처/해시와 정확한 rect를 기록한다.
2. TDD로 모든28슬롯337프레임의 정확한 coverage·중복/누락/손상 검사를 추가하고 공통 팩/미리보기·기존 렌더러에 연결한다. 실제 atlas 크기를 측정해 bounded pack 용량과 초기 로딩 메모리를 검토한다.
3. Native/WASM에서 각 상태/방향·마지막 프레임/전환·HD 사용 여부와 원본/HD 상태·mask·사운드 비변경을 검증한다. 자체 장면만 공개하고 원본 캡처는 ignored로 유지한다.
4. 문서/감사/상위 계획·일지·self-review·정확한 CI와 실제 merge를 마친 뒤 종료한다. #113 성능 이월/NOT READY·#142 진단·공개 v0.1.0/Pages 유지.

완료: PR #155/merge `7d8712b4f3897c7811f8917a1af6e332b87e7243`, 최종 head `963fcea38f83cc3f2dcc6e413ef07a8a2d31e54b`의 CI39887504382 attempt17개 SUCCESS·tree 일치. [결과/남은 범위](../reports/issue-154_hd_actors.md)를 따른다.
