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

최종 카탈로그로 원본120초기 장면·선택1,920틱·Web352Canvas를 대조했고 상태/마스크/사운드·Native/WASM 상태가 일치했다. 원본 함정 fallback0·console exception0이었다. 원본 레벨에는9종이 실제 배치되며 미배치 전기 바늘4/7은 합성 코스로 검증했다. 기존14위험 사례/28재생/11,788상태/1,714픽셀 대조, 전체703Rust/302Python 및 최종615소스 감사·G11기록 무결성·307문서 링크·Web8개 검사 명령을 통과했다. 마지막 solution replay는 증거 보존 옵션으로 실행해 모든 명령 종료0을 확인했다. self-review에서 원본 압축 인덱스·기준점·RGBA 보존과 core/resources/audio 제품 소스 미변경을 검토했다. 렌더링 정합은 전체 레벨 해결/성능 인수와 다르다. 횃불·나머지 장식/효과·글꼴/전체 UI와 HD 배포/인수는 후속이다. #137 OPEN, BGM #165는 리마스터 이후이며 #113 NOT READY와 공개 v0.1.0/Pages는 기존 범위다.

## CI와 실제 병합

PR #179 head `4081c95be985ee47c3984562f9399b1c5e87d0cd` / CI39948229305의7개 SUCCESS, 실제 merge `b1609edbde498d55e55e181c22adc1664ad95483`와 검토 head 전체 tree 일치를 확인했다.

[PR #179](https://github.com/edwardkim/ohmylemmings/pull/179) · [CI39948229305](https://github.com/edwardkim/ohmylemmings/actions/runs/39948229305). Python·WASM·Rust native·Web·Windows·macOS Apple Silicon/Intel을 모두 통과했다. PR checkout `534afdbfc592e12ac37eefd6e78b8791302d0569` / tree `972a91962aa55c891216fccc18a4a3c726bee630`도 검토 head와 같다. 중복 push39948220334 취소는 PASS에 포함하지 않는다. 기존 플랫폼 패키징 CI와 전체 HD 배포/성능 인수를 구분한다. 완료 문서 병합·root Native/Web 준비·preview-172 보존과 종료 근거는 [#178](https://github.com/edwardkim/ohmylemmings/issues/178)에 기록한다.
