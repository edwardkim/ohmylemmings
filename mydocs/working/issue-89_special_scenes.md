# G11-03 특수 장면 연결 진행

이슈: [#89](https://github.com/edwardkim/ohmylemmings/issues/89) · 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1) · [G11 계획](../plans/issue-85_compatibility_validation.md).

기준 main `facbc46ea2d3644574520f00fca6c43d3952c3b2`. 이슈 등록·번호 확인 후 착수했다.

## 목표와 설계

- Fun22→VGASPEC3, Tricky14→VGASPEC1, Taxing15→VGASPEC0, Mayhem22→VGASPEC2. 기존 자원 계층의 검증된 VGASPEC/terrain/object API를 재사용한다.
- 초기 충돌 마스크와 매 렌더 지형 재합성에 동일한 특수 소스를 사용한다. 배경은 (304,0), 960×160이며 색0 투명도와 RGB를 구분한다.
- 특수 배경과 객체의 팔레트가 다르므로 다리 픽셀은 객체가 가리지 않을 때 게임 팔레트7을 사용한다. 지형 삭제/재생성·객체 가림 계약을 합성 테스트로 고정한다.
- Web 파일 허용 목록에 VGASPEC0..3만 추가하고 실패한 교체는 세션·화면·시계·음원 세대를 보존한다.
- 특수4개 실제 SDL/WASM 진입·정적 독립 증거 비교를 남긴다. 전체120개 초기340경계/완주/성능은 G11-04 이후다.

## 진행

- [x] #89 등록·URL 확인, 기존 디코더/지형/객체/호스트 검토
- [x] 실패하는 합성 테스트와 구현
- [x] 실제 특수4개 SDL/WASM·UI·재생/사운드 회귀
- [ ] 자체 검토·보고·한글 PR·병합/CI·상위 갱신

로컬606/29/164·WASM, 원본 포함 실제 Web43사례86회14,980상태/8,617Canvas, SDL648상태80readback, 네 정적 위상 정렬 RGBA 해시 일치. [결과](../reports/issue-89_special_scenes.md)·[계약](../tech_insights/issue-89_special_scenes.md). 다음 게시/병합 CI4작업과 이슈 마감.
