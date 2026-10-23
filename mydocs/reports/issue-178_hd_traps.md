# #178 기계 함정 HD

이슈: https://github.com/edwardkim/ohmylemmings/issues/178 · 상위 #137

원본 effect4 함정10종159프레임을 새9PNG로 연결했다. 낙석·추·대리석 압착기·케이블 장치·좌우 벽 가시·전기/광선 장치를 구분하고 포획 캐릭터도 HD로 제작했다. built-in `image_gen.imagegen`을 사용했으며 선택 PNG는 출력 바이트 그대로다. 전체 프롬프트·원본 GROUND 메타데이터/출처·해시와 사각형은 [대응 목록](../../assets/manifests/hd_traps.json), [팩](../../assets/manifests/remaster_traps.json), [자산 안내](../../assets/remaster/README.md)에 있다. 표시 cutoff127로 낮은 alpha 여백을 제외한다.

## 판정·표시

원본 first0·once·8/7/12/15/16/17/25/37 프레임 수와 trigger/sound ID를 보존했다. 새 키 범위/완전성 검사는159키의 누락·잘못된 프레임을 거부한다. 기존 렌더러의 실제 TrapStates 프레임 선택을 그대로 사용하며, HD/원본 함정 프레임 진단만 더했다. 원본 시뮬레이션·파서·오디오 합성 제품 소스는 변경하지 않았다. 잘린 source 여백은 프레임별 사각형과 공통 밀도/앵커로 대응하여 고정 베이스·벽의 정렬을 유지한다. 케이블 시트의 구분선은 선택 사각형 밖에 있다.

## 로컬 결과

- TDD: Rust/Python 키·누락 검사의 RED→GREEN, 전체159위상·실제 발동·재사용·Pause 정지·그리기의 상태/사운드 불변을 확인했다. Rust 전체703개, Python 자산/함정 fixture27개 검사를 통과했다.
- 실제 SDL10종 정상/Pause20코스·3,808상태·80회 Trapped. 원본/HD의 모든 상태·입력·마스크·사운드가 일치했다. 앞선 비함정1개 후 같은 함정2개를 두어 압축 인덱스1/2와 객체 ID가 다른 상태에서 각각 독립적으로 재생됨을 검사했다.
- 같은 코스의 실제 WASM/Canvas3,808상태·1,881표시.159위상 모두 HD, 원본 함정 fallback0, Native/WASM 매 상태·모드별 마스크/사운드 일치, console exception0.
- 기존39page 바이트 동일, 새6,123,337 RGBA 샘플이 생성 PNG와 동일하다. 전체48page/199,892,424바이트이며25scope의 최대24page/119,886,132바이트로128MiB/32page 안에 있다. 기존 기본v1 팩131,699,864바이트/SHA-256 `07e240a23f338fef77e7f9fbf6fc439ba3e0dd3972566f9e848bcbe106014ff4`도 보존했다.

새 catalog SHA-256: `c6c9b7db1dd1727c3cc3c02d04011af01cc33a149dd0e7393d4a00186b5fa3dc`.

[케이블 장치](../../assets/remaster/previews/issue-178_web_hoist.png)·[벽 가시](../../assets/remaster/previews/issue-178_web_spikes.png)·[전기 함정](../../assets/remaster/previews/issue-178_web_lightning.png)은 자체 코스의 실제 Web Canvas 캡처다. 원본 그래픽/레벨 bytes가 없는 fixture만 공개하며 원본 장면 캡처는 ignored에 보존한다.

실물 화면 검토에서 낙석의 표시 기준점을 보정해 포획 위치와 지면을 맞췄다. 긴 전극의 비율과 생성 그림의 여백도 재제작/선택 사각형으로 보정했다. 아래 최종 검증에는 수정한 사각형/앵커와 카탈로그를 사용한다.

## 재현과 남은 범위

```bash
python3 tools/web_build.py build
python3 tools/check_hd_traps.py --output assets/local/trap-courses
cargo run --release --offline -p lemmings-game --example hd_terrain_probe -- assets/local/web/hd-assets assets/local/dos-original
node tools/web/hd_traps.cjs PRODUCT_URL DISPOSABLE_CDP_URL assets/local/web-traps CASES_URL
node tools/web/hd_terrain.cjs PRODUCT_URL DISPOSABLE_CDP_URL assets/local/web-scenes NATIVE_JSONL --traps
```

원본120초기 장면/선택 틱·기존 위험 회귀·전체 Python/Web·소스 감사·self-review·게시 CI와 실제 병합 결과는 완료 기록에 추가한다. 렌더링 정합은 전체 레벨 해결/성능 인수와 다르다. 횃불·나머지 장식/효과·글꼴/전체 UI와 HD 배포/인수는 후속이다. #137 OPEN, BGM #165는 리마스터 이후이며 #113 NOT READY와 공개 v0.1.0/Pages는 기존 범위다.
