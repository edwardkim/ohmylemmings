# #175 물·불 위험 요소 HD

이슈: https://github.com/edwardkim/ohmylemmings/issues/175 · 상위 #137

물5객체40프레임과 effect6 장치4객체44프레임을 새8PNG로 연결했다. 청색 물은 원본 팔레트/형상이 같은 세트0/3에서 공유한다. 세트2/9는 회전 장치이며 화염 그림으로 대체하지 않았다. 생성 도구는 built-in `image_gen.imagegen`이고 선택 PNG는 출력 바이트 그대로다. 전체 프롬프트·원본 GROUND 메타데이터/해시·실제 사각형은 [대응 목록](../../assets/manifests/hd_hazards.json), [확장 팩](../../assets/manifests/remaster_hazards.json), [자산 안내](../../assets/remaster/README.md)에 있다. 표시 cutoff127로 낮은 alpha 배경 잔여를 제외한다.

키 범위는 원본8/10/16프레임에 제한한다. 완전한 위험 팩은9객체84키를 요구하며 누락/잘못된 키를 거부한다. 기존 Object 렌더러가 원본 프레임 주기·순서·뒤집기·클리핑을 적용하며 새 진단은 HD/원본 위험 키를 구분한다. 원본 trigger·충돌·사망·사운드 합성 소스는 바꾸지 않았다.

## 확인한 결과

- TDD: Python 키/누락 정책과 Rust 파서의 RED→GREEN, Rust 전체84위상2회 반복에서 상태/오디오 불변. HD 파서18·Python 관련31테스트 통과.
- 실제 SDL9개 합성 코스991상태: 익사5·vaporize4, 원본/HD 상태·입력·마스크·사운드 일치.
- 같은 코스의 실제 WASM/Canvas991상태·405표시:84위상 모두 HD, 원본 위험 fallback0, Native/WASM 최종뿐 아니라 매 상태 일치, console exception0.
- 기존 위험14사례/28재생/11,788상태/1,714픽셀 대조 통과. 새 기준 실행은 하지 않았으며 고정한 기존 기준 결과와 대조했다.
- 기존31page 바이트 동일. 새7,970,037 RGBA 샘플이 PNG와 동일. 전체39page/173,879,960바이트, 모든25scope가 최대24page/119,886,132바이트로128MiB/32page 안에 있다. 기존 캐릭터·입출구·특수 장면 페이지를 보존했다.
- [물](../../assets/remaster/previews/issue-175_web_water.png)·[분사기](../../assets/remaster/previews/issue-175_web_fire.png)·[회전 장치](../../assets/remaster/previews/issue-175_web_spinner.png)는 원본 바이너리가 없는 자체 코스의 실제 Web Canvas 캡처다. 원본 레벨 캡처는 ignored에 보존한다.

카탈로그 SHA-256: `41624a0b47a371e6daa2f0cd7b01e2209d09b823809e1c7b0be85d7995030c0a`. 기존 v1 기본 팩은131,699,864바이트/`07e240a23f338fef77e7f9fbf6fc439ba3e0dd3972566f9e848bcbe106014ff4` 그대로다.

## 재현

`make check-rust`, `make check-python`, `make check-web`와 아래 실물 검증을 실행한다. 출력 폴더는 새 ignored 경로를 사용한다.

```bash
python3 tools/web_build.py build
python3 tools/check_hd_hazards.py --pack assets/local/web/hd-assets --output assets/local/hazard-courses
cargo run --release --offline -p lemmings-game --example hd_terrain_probe -- assets/local/web/hd-assets assets/local/dos-original
node tools/web/hd_hazards.cjs PRODUCT_URL DISPOSABLE_CDP_URL assets/local/web-hazards CASES_URL
node tools/web/hd_terrain.cjs PRODUCT_URL DISPOSABLE_CDP_URL assets/local/web-scenes NATIVE_JSONL --hazards
```

검증 코스는 원본 trigger의4픽셀 높이에도 actor 발이 들어가도록 바닥/객체를 배치한다. 새 코스 배치의 초기 오류와 분사기 여백 사각형의 경계 오류를 검수 중 바로잡았다. 제품의 원본 위험 판정은 유지했다.

원본120장면/선택 틱·전체 로컬 검사·감사·self-review·게시 CI와 실제 병합 결과는 완료 기록에서 추가한다. 이는 전체 레벨 완주나 성능 인증이 아니다. 기계 함정·횃불/기타 효과·글꼴/전체 UI와 HD 배포/전체 인수는 후속이다. BGM #165는 #137 이후, #113 NOT READY·#142·공개 v0.1.0/Pages는 기존 범위다.
