# #157 HD-05b 일반 지형 재질

이슈: https://github.com/edwardkim/ohmylemmings/issues/157 · 상위 [#137](../plans/issue-137_2d_remaster.md) · [계획](../plans/issue-157_hd_terrain.md)

## 구현과 범위

일반 지형 세트0~4에 흙/암석·화산암·대리석·사암·결정 재질을 연결했다. 기존 지형 형상을 그대로 사용하며 강철/계단 벽돌을 구분한다. 일방통행은 기존 객체 화살표의 방향·위상·지형 클리핑·트리거를 유지한다. 특수 배경4종과 팩에 없는 재질은 원본 그림을 사용한다. 충돌·틱·입력·사운드는 변경하지 않았다.

[5재질 반복 미리보기](../../assets/remaster/previews/issue-157_materials.png)는 실제 브라우저의 자산 검토 Canvas 캡처다. 실제 원본 레벨 캡처는 ignored `assets/local/issue-157-review`에만 보존한다. 재질은 원본273개 지형 조각을 개별 재디자인한 결과가 아니다. 시설/효과·글꼴/전체 UI·특수 장면의 HD 제작과 지원 패키징/인수는 후속이다. `full_remaster_ready=false`를 유지한다.

## 생성 자산과 팩

built-in image_gen을 사용했다. 새 PNG4장은 각각1254×1254/완전 불투명이며 원본 샘플을 수정·재표본화하지 않았다. [자산/프롬프트 목록](../../assets/remaster/README.md), [규격·출처·SHA-256·검증 근거](../../assets/manifests/hd_terrain.json), [팩 대응](../../assets/manifests/remaster_pack.json)을 따른다.

기존 v1 구조와128MiB/32atlas 한계를 유지한다. `packing: rectangles-v1`은 사용한 사각형을 결정론적으로 재배치하고 경계 샘플을 복사한1픽셀 여백을 둔다. 동일 사각형은 공유하며, 커지는 경우 원래 atlas를 유지한다. 원본 PNG/사각형 대응은 제작 manifest에 보존하고 런타임 팩 좌표만 바꾼다. 표시 크기·기준점·cutoff도 유지한다. 컴파일러는 source PNG를 한 장씩 해독한다.

새4재질을 포함해23atlas/366자산, 캐릭터28/337·시설11·아이콘13·재질5다.119,548,732→109,503,312바이트로10,045,420바이트 감소했다. 최종 팩 SHA-256: `6f1561f12318b4fe62e3cb53088146ac97822b3fb2d86df7197310512cb7d744`. 이전362자산과 새4재질의26,313,649개 RGBA 샘플을 바이트 단위로 대조했다. Native CRC/모든 행·이전44,046,160바이트7atlas106자산 팩 읽기도 PASS다. `source-v1` 제작 정책도 유지한다.

## 검증

- TDD: 새 재질 키/범위·구버전 인코딩, 결정론 팩/중복 사각형·원본 RGBA/straight alpha·복사 여백, 알 수 없는 정책 거부. Rust는5재질·빈 셀/삭제·강철/벽돌·누락 재질/특수 경로와 청크 경계·스크롤 캐시를 검사한다.
- Native/Web: 실제120레벨 초기 장면, 일반5테마·특수4배경·일방통행14레벨의 경로를 검사했다. 선택16사례×120틱=1,920틱의 원본/HD 상태·전체 지형/벽돌/객체 마스크·사운드가 동일하다. Web Canvas352표시, 모든120사례의 Native/WASM 최종 상태 일치·콘솔 오류0. 이는120레벨 완주나 성능 판정이 아니다.
- 실제 SDL dummy/software: Fun01/02/04/05/06/11/24의 원본/HD 각120틱. 총847상태, 사례당20개 진단/입력/최종 맵 파일 일치. 재질5종과 좌우 화살표를 실제 화면에서 검토했다. GPU/60fps 검증은 아니다.
- 전체 캐릭터: Native 복원337·Canvas337프레임, 미리보기30그룹/60끝점·13아이콘·5재질 PASS. 복원 프레임 검사와 자연 플레이 검사를 구분한다.
- 기존 Web 회귀:33사례/66실행·11,960상태·6,877Canvas, HD 자연13사례/7,593상태/821표시 PASS. 실제 제품 Fun01 664틱/665상태·10구출,8직업/Pause/미니맵/재시작·레밍 클릭·전체화면·모드 전환 PASS.
- 로컬 workspace Rust/doc·추가 지형 테스트, Python275·format/Clippy/WASM library PASS. 현재 소스 감사·문서 검사·정확한 PR head CI/실제 병합은 아래 마감 기록과 이슈에서 확인한다.

원본 캡처·전체 진단/맵·ZIP은 ignored로 보존하고, 공개 manifest에는 경로·해시·요약만 기록한다. #113 성능 이월/NOT READY·#142 진단·공개 v0.1.0/Pages는 기존 상태를 유지한다.

## 재현

기존 원본 확보와 Rust/SDL2/WASM prerequisites 후 실행한다. 원본 경로는 로컬 준비 경로로 바꾼다.

```sh
make hd-assets
cargo run -p lemmings-game --example hd_asset_probe --release --locked --offline -- assets/local/hd-runtime/assets.hdp --all-actors
cargo run -p lemmings-game --example hd_character_probe --release --locked --offline -- assets/local/hd-runtime/assets.hdp
cargo run -p lemmings-game --example hd_terrain_probe --release --locked --offline -- assets/local/hd-runtime/assets.hdp assets/local/dos-original > assets/local/catalog-hd.jsonl
```

전체 게임 Web 패키지를 로컬 HTTP로 제공하고 **별도로 시작한 빈 단일 탭 Chromium**에 임시 CDP 포트를 지정한다. 사용자 CDP19222는 사용하지 않는다. 증거 디렉터리는 새 경로여야 하며 도구는 자기 QA 브라우저를 닫는다.

```sh
node tools/web/hd_terrain.cjs http://127.0.0.1:8080/ http://127.0.0.1:19257 assets/local/catalog-web-review assets/local/catalog-hd.jsonl
```

## 마감

로컬 검증·self-review를 마치고 코드 PR의 정확한 head CI와 실제 merge를 진행한다. 최종 PR/CI·merge·main 개발 준비는 이슈와 완료 기록에 동기화한다.
