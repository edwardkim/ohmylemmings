# #137 2D HD 리마스터 진행

이슈: https://github.com/edwardkim/ohmylemmings/issues/137

메인테이너의 리마스터 등록/처리 지시를 받아 #137 OPEN·번호/URL을 확인했다. 초기 개발 #1은 CLOSED, 성능/메모리 #113은 독립 OPEN으로 유지한다. 기준 main001671c에서 별도 worktree/branch로 시작했다.

첫 절편 #138 OPEN·번호/URL 확인 후 실제 Fun 01을 bounded100틱으로 캡처하고 지형 배치·캐릭터 크기를 확인했다. 원본 캡처·trace는 기존 ignored assets/local에 보존했다. 이 실행은 성능 측정이나 새로운 전체 해법 검증이 아니다.

built-in image_gen으로 부드러운 손그림풍2D의1672×941 시안을 제작하고 프로젝트 assets/remaster에 복사했다. 프롬프트·출처/해시·원본 대응 목록과 [HD 자산/표시 계약](../plans/issue-138_hd_art_contract.md)을 작성했다. 시안은 디자인 후보이며 분리된 런타임 자산·실제 HD 게임 모드는 아직 없다. 사용자 스타일 응답은 후속 제작에 우선 반영한다.

원본28애니메이션337프레임·5세트273지형/54객체·540프레임/미리보기54개·특수4장면·UI19묶음200개를 제작 범위로 기록했다. 모든 런타임 HD 항목은 not_started로 유지했다. 검증된 원본 데이터나 규칙을 그림으로 대체하지 않는다.

HD-01은 PR #139/merge5a543b3로 병합하고 #138을 완료했다. 메인테이너 다음 진행 승인으로 [HD-02 / #140](issue-140_hd_presentation.md)을 등록·확인하고 typed HD snapshot·좌표/입력 계약과 합성16개 테스트를 구현했다. 정확한 로컬/CI·병합은 [#140 결과](../reports/issue-140_hd_presentation.md)를 따른다. 실제 HD 자산/표시·한 레벨/전체 확장·지원 플랫폼/패키지는 [상위 계획](../plans/issue-137_2d_remaster.md)의 후속이다.

HD-02 #140은 PR #141 실제 병합·동일 tree와 최종 CI7개 SUCCESS로 완료했다. Web180초 대기 실패는 같은 head의 해당 job 재실행으로 통과했고 Native15분 취소/설치 지연·실패 이력은 #140 보고서에 보존했다. 다음은 HD-03 분리 캐릭터/애니메이션·지형·시설/UI 자산 제작/로딩·검증이다. 실제 HD 플레이 화면·전체 자산은 미구현이므로 #137은 OPEN을 유지한다.

사용자 다음 단계 승인으로 #144 HD-03a를 등록·확인하고 걷기 좌우16프레임·암석 분리 PNG와 RGBA 팩 로더/검증을 구현한다. 실제 HD 게임과 다른 동작/시설/UI는 후속으로 유지한다. [진행](issue-144_hd_assets.md), [결과](../reports/issue-144_hd_assets.md).

HD-03a #144의 걷기16프레임·암석 후보와 공통 팩/검증은 PR #145 실제 병합·동일 tree/CI7개 SUCCESS로 완료했다. 통합된 HD 애니메이션은 여전히0개이며, 다음은 입출구·굴착/계단/폭발·UI 제작 후 실제 Fun01 HD 통합이다. #137은 OPEN으로 유지한다.

HD-03b [#147](https://github.com/edwardkim/ohmylemmings/issues/147)의 직업65프레임·첫 테마 입출구11프레임·13조작 아이콘과 typed 팩 확장을 PR #149/merge `653762ff7383d87a5963fc2bc39c43d25983fdd2`로 실제 병합했다. 최종 head `3ff29f8df90f8bcad4c31eff84d9c11829f7bd25`/CI39866059128의7개 검사 SUCCESS·전체 tree 일치, 로컬 실제7atlas106자산/81캐릭터 프레임·브라우저 미리보기·기존 팩 호환성을 확인했다. [결과](../reports/issue-147_hd_actions.md)를 참고한다. 다음은 HD-04 실제 Fun01 Native/Web 합성·원본 모드 선택/입력 통합이며, 동작 크기/위상·출구 doorway/원본 trigger와 누락 상태 처리도 이 단계에서 검토한다. 전체 HD 게임/GPU·나머지21애니메이션256프레임/다른 테마·시설/효과·전체 UI·배포는 미완료, #137은 OPEN이다. #113 성능 이월/기존 NOT READY·#142 Chrome156 진단·#1 CLOSED/공개 v0.1.0/Pages를 유지한다.

HD-04 [#151](../reports/issue-151_hd_integration.md)의 실제 Native/Web HD 합성·원본 전환/전체화면·입력과 Fun01 동일성을 PR #152/merge `3147601e06b12b67289fc5c105ed19163bf09203`로 실제 병합했다. 최종 head `efadab1f241061f102d2a96cc0cb190fa1f4c920`/CI39876930568 최종 attempt2의7개 SUCCESS와 전체 tree 일치. 실제7애니메이션81프레임 통합/부분 runtime_ready=true이며 full_remaster_ready=false다. 다음 HD-05/06·#137 OPEN, #113 이월/NOT READY·#142 진단과 공개 v0.1.0/Pages 유지.

HD-05a [#154](../reports/issue-154_hd_actors.md)에서 새256포즈를 더해28/337 캐릭터 대응을 완성했다. PR #155 head `963fcea38f83cc3f2dcc6e413ef07a8a2d31e54b`의 CI39887504382 attempt17개 SUCCESS·merge `7d8712b4f3897c7811f8917a1af6e332b87e7243` 실제 병합/tree 일치. Native337복원/Canvas337, 실제13합성 시나리오7593상태821표시(18슬롯 자연 관측), 원본/HD Fun01/우산·실제 Web Fun01 동일성을 구분해 기록했다. 전체 테마/시설/효과/UI·지원 HD 배포는 후속이며 #137은 OPEN이다.

HD-05b [#157 일반5테마](../reports/issue-157_hd_terrain.md)를 PR #158/merge `08d309f1aac5edc194752f6004f96fcd8f92f9a0`로 실제 병합했다. head `c1ad3d114b7943d19cd1c6042cfb36d76da98fc0`와 전체 tree 일치, CI39895388097 attempt1의7개 SUCCESS를 확인했다. 새 PNG4장/프롬프트·원본273지형 형상/일방통행 화살표 유지, 정확한26,313,649 RGBA 샘플과 팩119.5→109.5MB를 검증했다.120초기 장면·16사례1,920틱/SDL7사례847상태·캐릭터337/기존Web 회귀와 실제Fun01구출을 구분해 기록한다. 특수4장면/시설/효과/전체 UI와 HD-06은 후속이다.

HD-05c [#160 전체 입출구·출구 장식](../reports/issue-160_hd_portals.md)을 PR #161/merge `40b4bdaadb2d1f8198d4cd1cb5c1617a4c76fbbd`로 실제 병합했다. head `e5a6b302ee6aa6a52088b3eb81f2012cef8d8059`와 전체 tree 일치, CI39904818590 attempt1의7개 SUCCESS다. 새9PNG/정확한 프롬프트·전체55문/30실제 출구 장식,32atlas440자산/133,385,948바이트·기존366/새74키의31,939,921 RGBA 샘플을 보존했다. Native/Web 자체5코스947상태·전55/30위상·5구출/120Canvas, 원본120초기/1,920틱/352Canvas·SDL5사례605상태·기존 Web/캐릭터337/실제 Fun01 UI와10구출을 확인했다. 일반14프레임 횃불과 실제6프레임 출구 장식의 식별을 시각 검토로 바로잡았으며 화살표/함정을 바꾸지 않는다. 특수4장면·나머지 함정/효과/글꼴/전체 UI와 지원 HD 배포/HD-06은 후속이다. 완료 문서·root 준비와 #160 종료 근거는 이슈를 따른다. #137·#113·#142/NOT READY·공개 배포는 유지한다.

HD-05d [#163 팩 정리](../reports/issue-163_hd_pack.md): 화면을 보존한27atlas/440키·131,699,864바이트 의 로컬 검증·PR #164 CI7개·실제 병합 완료.

## 메인테이너 작업 우선순위

2026-10-23 지시: **2D 리마스터 #137을 먼저 완료하고, 이후 BGM #165를 수정한다.** #165의 원인 조사 결과는 보존하지만 리마스터 진행 중 제품 수정을 시작하지 않는다.

남은 순서:
1. 완료 #169: 테마별 선택 로딩으로 활성 자산 예산 약32~38MiB 확보.
2. 완료 #172/#175/#178/#181: 특수4장면과 원본54객체/540디스크 키를 HD로 대응했다.
3. 완료 #184: 현재 제품의 HD 글꼴·상태/기술/미니맵·Pause/카운트다운/결과 화면을 완성했다. 새 DOS 메뉴/한국어 번역은 추가하지 않았다.
4. Native/Web 원본 규칙·입력·화면 회귀, 지원 HD 패키징·출처/고지·설치 인수를 마쳐 #137을 완료한다.
5. 그 다음 #165의 공통 OPL buffered delivery 구현과 지속 BGM/효과음 병행·플랫폼/청취 검증을 진행한다.

BGM 결함은 알려진 별도 후속으로 명시하며, HD 검증 통과를 BGM 정상 판정으로 확대하지 않는다. #113 성능 이월과 공개 배포의 버전/태그 정책은 유지한다.

[#169 테마별 로딩](../reports/issue-169_hd_loading.md): 다음 그래픽 제작용 활성 예산 약32~38MiB 확보. 로컬 Native/Web 동일성·전환/실패 복구와 PR #170 CI7개·실제 병합 완료. 이후 특수4장면/함정·효과/글꼴·전체 UI를 진행한다.

[#172 특수4장면](../reports/issue-172_hd_special.md):4PNG와 원본 mask 표시·선택 catalog를 연결했다. 로컬 검증·사용자 Fun22 시연·Web120장면·감사와 PR #173 CI7개·실제 병합을 완료했다. 다음은 나머지 함정/효과와 글꼴/전체 UI다.

[#175 물·불 위험 요소](../reports/issue-175_hd_hazards.md):8PNG·9객체84프레임을 연결했다. SDL/WASM9코스991상태·84위상, 원본120초기 장면/1,920틱/352Canvas·기존31page 보존과 감사·PR #176 CI7개·실제 병합을 완료했다. 기계 함정·횃불/기타 효과·글꼴/전체 UI와 HD 배포/인수는 후속이며 #137은 OPEN이다. BGM #165는 리마스터 이후다.

[#178 기계 함정](../reports/issue-178_hd_traps.md):9PNG·10종159프레임과 실제 TrapStates 위상·발동/재사용/Pause를 연결했다. SDL/WASM20코스3,808상태·80포획·1,881Canvas/159위상, 원본120장면·1,920틱·352Canvas, 기존39page와 RGBA 보존·감사·PR #179 CI7개·실제 병합을 완료했다. 다음은 횃불/기타 장식·글꼴/전체 UI와 HD 배포/인수다. #137 OPEN·BGM #165는 리마스터 이후다.

[#181 깃발·일방통행 표시](../reports/issue-181_hd_decor.md):2PNG·20종212키를 연결하여 원본54객체/540프레임의 HD 대응을 갖췄다. 원본에서 미배치된14종은 합성으로 검증했다. SDL/WASM5코스1,790상태·670Canvas/212위상과 지형 제거·Pause, 원본120장면/1,920틱/352Canvas·627소스 감사·PR #182 CI7개·실제 병합을 완료했다. 초기 횃불 분류는 회전 깃발로 정정한다. 다음은 글꼴/전체 UI·지원 HD 배포/전체 인수이며 #137 OPEN·BGM #165는 리마스터 이후다.

[#184 HD 글꼴·게임 UI](../reports/issue-184_hd_ui.md):95 AA glyph·3색 atlas와 코드 기반 skin, 상태/8기술/미니맵·Pause/카운트다운/성공/실패/마지막 레벨 안내를 완성했다.14조작 영역·원본 모드/규칙과51카탈로그 파일을 보존했다. SDL/WASM7코스826상태·실제 Fun01/6레벨 선택 RGBA·709Rust/305Python·PR #185 CI7개·실제 병합을 확인했다. 다음은 HD-06 지원 HD 패키징/전체 인수다. #137 OPEN·BGM #165는 리마스터 이후다.

## 최종 마감 #187

PR #188 실제 merge `6a7c68dc61b063f915aae15476dc3c61bd75a51b`, head `6243132b5fd094d4d49804e075bd9443e024d18a`의 [CI39976023448](https://github.com/edwardkim/ohmylemmings/actions/runs/39976023448)에서7개 모두 SUCCESS. CI checkout `96f74ff2e6fce579952223e6ddbe6abb1d2bd263`과 실제 병합 tree `ef01904c2cc8d3884303499d79dc9b3e5479ecfc`가 같다.

899HD 키·현재 전체 UI/95글꼴·지원 패키지/설치 인수를 마쳤다. [최종 보고서](../reports/issue-187_hd_release.md). 앞선 부분 preview/Open 기록은 당시 기록이다. #137 완료 후 BGM #165를 처리하며 #113 NOT READY·공개 v0.1.0/Pages·Mac 미공증은 유지한다.
