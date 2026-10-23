# #160 전체 테마 HD 입출구와 출구 장식

이슈: https://github.com/edwardkim/ohmylemmings/issues/160 · 상위 [#137](../plans/issue-137_2d_remaster.md)

## 변경과 범위

기존 세트0 입출구11키에4테마44키를 추가해 전체5테마의 입구50·출구5프레임을 연결했다. 실제 출구에 별도로 놓인 장식도 테마별6프레임씩 총30키를 HD 쌍불꽃으로 교체했다. 원본의 문닫힘1→열기2~9→열림유지0과 장식의 `(iteration+2)%6` 위상, 원본 위치·그리기 순서·클리핑·trigger를 유지한다. 캐릭터337키·5지형 재질·틱·입력·사운드와 충돌 판정은 변경하지 않았다.

장식은 실제 레벨 배치와 GROUND 메타데이터의 세트0/id7, 세트1~3/id6, 세트4/id8이다. 모두 폭48px·effect0·6프레임이며 원본 높이는8/24/8/8/8px이다. 처음 식별한 id2/3은14프레임 일반 횃불로, 실제 Fun04/06 화면에 겹치던 출구 장식과 달랐다. 시각 검토로 대상을 바로잡고 시험84키는 배포 대상에서 제외했다. 화살표·함정·불 위험물을 숨기거나 새 불꽃으로 바꾸지 않았다.

built-in image_gen으로4입출구 PNG와5불꽃 PNG를 생성했다. 각각1536×1024이며 선택 PNG의 RGB/straight alpha 샘플을 수정하거나 재표본화하지 않았다. 실제 잉크 경계와3px 여백, 명시적 cutoff8, hatch top/center와 doorway floor/trigger center 기준점을 기록한다. 불꽃 atlas의16포즈 중 앞12포즈를 인접한2포즈씩 묶어6쌍으로 사용한다. 제작 좌표와 원본 PNG는 보존한다.

[자산·정확한 프롬프트 목록](../../assets/remaster/README.md), [출처·규격·원본 메타데이터·대응/검증](../../assets/manifests/hd_portals.json), [런타임 팩 manifest](../../assets/manifests/remaster_pack.json)를 따른다. [입출구 연락표](../../assets/remaster/previews/issue-160_portals.png)와 자체 제작 코스의 [실제 Web Canvas](../../assets/remaster/previews/issue-160_web_course.png), [실제 Native SDL](../../assets/remaster/previews/issue-160_native_course.png)을 제공한다. 원본 레벨 캡처와 전체 진단/맵은 ignored에만 보존한다.

팩은 기존 v1과128MiB/32atlas 한도에서 **32atlas/440자산·133,385,948바이트**다. 캐릭터337·문55·장식30·아이콘13·지형5를 포함한다. SHA-256은 `328097f22e3f6bbf1075cf0028dcdc3a8846f0cecbc9fc41440fd226a70ac20e`다. 기존 작은 v1 팩도 계속 읽는다. 현재 상한에 가까우므로 후속 자산 확장 때 atlas/팩 정리가 필요하다.

## 검증

- TDD:5세트55문 키와30장식 키, 잘못된 세트/객체/프레임·손상 팩·제작 누락/부분 구버전 호환을 검증했다. 실제 렌더러의 문 위상과6프레임 장식 루프에서 상태/사운드 비변경을 확인했다.
- 무손실: 기존366키와 새74키의31,939,921개 RGBA 샘플·표시 크기/기준점·cutoff를 대조했다. Native CRC/모든 행과 이전7atlas106자산44,046,160바이트 팩 호환 PASS.
- 자체5코스: 실제 SDL 원본/HD의947상태·사례당20개 진단/입력/최종 맵 파일 일치, 모두1마리 구출. 실제 WASM/Canvas에서도 동일947상태·120표시·문55/장식30 전 프레임 관측·모드 마스크/사운드 동일·5코스 성공·콘솔 오류0.
- 원본 카탈로그: Native/Web120초기 장면과 선택1,920틱, Canvas352표시·일반5테마/특수4배경/일방통행14장면, 문/출구 장식 원본 fallback0. Native/WASM 최종 상태와 원본/HD 상태·전체 마스크·사운드 일치.
- 실제 원본 SDL Fun01/02/04/05/06 각120틱, 총605상태·각20파일 일치. Fun04/06 출구 장식 겹침 해결을 실제 출력으로 확인했다. 실제 제품 Web Fun01 664틱/665상태·10구출·8직업/Pause/미니맵/재시작·레밍 클릭·전체화면 PASS.
- 전체 캐릭터 Native337복원·Canvas337표시, 문55/장식30 Canvas 표시·미리보기43그룹/86끝점·아이콘13/재질5/배경3 PASS. 복원 검사와 자연 플레이 검사는 구분한다.
- workspace Rust/doc와 변경 범위 최종 Rust, Python280·format/Clippy/WASM library PASS. 기존 Web33사례/66실행·11,960상태·6,877Canvas와 HD13사례·7,593상태·821표시도 최종 팩에서 PASS다. 소스 감사536개·문서289개 상대 파일 링크와 self-review PASS. 정확한 head CI/실제 병합은 최종 기록에 연결한다.

 검증은 원본120레벨 완주나 성능 인수를 뜻하지 않는다. #113 성능 이월/NOT READY·#142 진단·공개 v0.1.0/Pages와 Windows11 지원 정책은 유지한다. 특수4장면·나머지 함정/효과/글꼴/전체 UI·지원 HD 배포/HD-06은 후속이며 `full_remaster_ready=false`다.

이전 #157 Web CI는9분46초로10분 제한에 근접했다. PNG9장과 팩이 늘어난 이번 변경에서는 Web job 제한을15분으로 늘렸다. 테스트 내용과 제품의 시간/성능 기준은 유지한다.

## 재현

기존 Rust/SDL2/WASM prerequisites와 원본 확보 후 실행한다. 자체 코스는 원본 그래픽이나 레벨을 사용하지 않는다. 출력은 매번 새로운 ignored 경로를 지정한다.

```sh
make hd-assets
cargo run -p lemmings-game --example hd_asset_probe --release --locked --offline -- assets/local/hd-runtime/assets.hdp --all-portals
cargo build -p lemmings-desktop --release --locked --offline
python3 tools/check_hd_portals.py --binary target/release/lemmings-desktop --pack assets/local/hd-runtime/assets.hdp --output assets/local/portal-courses
make web
cargo run -p lemmings-game --example hd_terrain_probe --release --locked --offline -- assets/local/hd-runtime/assets.hdp assets/local/dos-original > assets/local/catalog-hd.jsonl
```

전체 게임 Web을 HTTP로 제공하고 **별도로 시작한 빈 단일 탭 Chromium**의 임시 CDP를 사용한다. 예시 HTTP는 assets/local을 루트로 제공한다. 사용자 CDP19222는 사용하지 않는다. 도구는 자기 QA 브라우저를 종료한다.

```sh
node tools/web/hd_portals.cjs http://127.0.0.1:18360/web/ http://127.0.0.1:19360 assets/local/course-web-review http://127.0.0.1:18360/portal-courses/cases.json
node tools/web/hd_terrain.cjs http://127.0.0.1:18360/web/ http://127.0.0.1:19360 assets/local/catalog-web-review assets/local/catalog-hd.jsonl --portals
```

## 최종 CI·실제 병합

[PR #161](https://github.com/edwardkim/ohmylemmings/pull/161)의 최종 head `e5a6b302ee6aa6a52088b3eb81f2012cef8d8059`를 [CI39904818590](https://github.com/edwardkim/ohmylemmings/actions/runs/39904818590) attempt1에서 확인했다. Rust native·Python tooling·WASM libraries·Web browser·Windows full game·macOS Apple Silicon/Intel의7개 검사가 모두 SUCCESS다. 중복 push CI39904813609 취소는 PASS에 포함하지 않으며 최종 PR head의 실패/재시도는 없다. CI의 PR checkout `b62e150152906773d9a362582daae0f723f193ca`와 검토 head 전체 tree도 같다.

Chrome155.0.8059.39의 전체 Web 회귀·오디오/저장 입력 smoke와 개발용 전체 게임 ZIP 새 설치도 PASS다. Web ZIP133,442,156바이트/SHA-256 `38255df676fcc79701f9b5df2af59d838c919cae6cbc85881b6e6a4518f90fbc`이며 대응 소스931파일과 게임 데이터28개를 검증했다. Windows와 두 Mac에서도 기존 전체 게임 패키지의 새 설치를 확인했다. 이를 전체 HD 지원 패키징이나 성능 인수와 구분한다.

PR을 merge `40b4bdaadb2d1f8198d4cd1cb5c1617a4c76fbbd`로 실제 병합하고 검토 head와 main 전체 tree 일치를 확인했다. 완료 문서·상위 계획/진행·일지를 동기화한다. 문서 PR/merge와 root main·Native/HD 팩·기존8080 전체 게임 Web의 최종 준비는 [#160 완료 기록](https://github.com/edwardkim/ohmylemmings/issues/160)에 연결한다. #160 범위는 완료이며 #137·#113·#142는 OPEN, #1은 CLOSED다. 공개 v0.1.0/Pages는 유지한다.
