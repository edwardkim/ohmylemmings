# HD 지형·지물 공통 품질 개선

담당 [#249](https://github.com/edwardkim/ohmylemmings/issues/249), [계획](../plans/issue-249_hd_material_quality.md). 기준 main은 `618b2caa97b9a29b7881ce7f79b6d1ba34f584d9`이다.

## 발견 사항과 변경

Mayhem08의 강철을 계기로 활성 HD catalog 전체를 조사했다. 5개 지형 재질, 4개 특수 배경과 54종 지물/540개 프레임 키가 모두 있으며, 현재 쓰는 월드 원화는38PNG다. 입출구·출구 불꽃·물·불·기계 함정·깃발·방향 표시를 함께 확인했다. 누락된 키나 단순 도형으로 남은 원화는 없었다. first/middle/last contact sheet 전수 시각 점검과 각 프레임의32×32 이하 RGBA 표본 검사에서, 단색 표본만 있는 프레임도 없었다. 이 표본 검사는 미술 품질을 자동 판정하는 지표는 아니다.

| 공통 부족 부분 | 개선 결과 | 적용 범위 |
|---|---|---|
| 강철이 RGB132/150/166 단색으로 주변 그림과 불일치 | 브러시 결, 판 이음새, 리벳, 마모·모서리 음영 | 모든 강철, 테마별5팔레트 |
| Builder 계단이 두 색 줄무늬 | 광물 입자·벽돌 색 변화·줄눈·마모·입체 음영 | 모든 건설 계단 |
| HD 지물을 확대할 때 nearest 표본화로 계단짐 | 중앙 정렬 bilinear, premultiplied 색 보간, 각 atlas 프레임 내부로 표본 제한 | 입출구/불꽃/위험물/함정/깃발/화살표54종540프레임 |
| 특수 배경4개의 확대 픽셀이 거침 | 같은 bilinear 보간을 월드 위치에 고정 | 특수 배경4종 |
| 결정 테마 빨간 일방통행 표지가 평면 윤곽선 | 기존 붉은 잉크에 에나멜 테두리의 음영·광택 | 결정 테마 좌/우 표지14프레임 |

모든 처리는 공통 Rust HD renderer에서 이루어져 Native·Canvas2D·WebGPU에 함께 적용된다. 강철/계단의 결은 월드 좌표 기준이므로 스크롤과 청크 재생성 때 움직이지 않는다. 결정 화살표의 투명한 내부와 alpha는 보간한 원래 값 그대로다. 다른 색의 사용자 대체 그림은 재착색하지 않는다.

물리 지형·강철/일방통행/trigger mask, 레밍 상태·입력·직업 수·프레임 수/위상·음원은 유지한다. 기존 #215 지형/수직벽 수준을 유지하며 그 주변의 누락된 재질과 표본화 품질을 보완했다. 원본 모드·불완전한 구형 팩의 fallback, 기능상 단색인 미니맵은 유지한다.

## 실제 화면

아래는 `tools.decor_fixture.bank(theme, scenery=True)`로 만든 자체 코스를 실제 SDL로 실행한 캡처다. 원본 레벨/이미지 바이트를 사용하지 않았고 framebuffer를 PNG로 무손실 저장했다. 물리 경계는 전후가 같다. Native 캡처 크기는960×600이다.

| 코스 | 변경 전 | 변경 후 |
|---|---|---|
| 흙/암석 | [전](../../assets/remaster/previews/issue-249_theme-0_before.png) | [후](../../assets/remaster/previews/issue-249_theme-0_after.png) |
| 결정·빨간 표지 | [전](../../assets/remaster/previews/issue-249_theme-4_before.png) | [후](../../assets/remaster/previews/issue-249_theme-4_after.png) |

실제 Mayhem08 Native 전후 및 일반 Wayland Chromium WebGPU 화면도 확인했다. 회색 강철 덩어리가 금속판으로 바뀌며 주변 화산암과 구분된다. 원본 레벨에서 파생한 캡처는 `assets/local/issue-249/`에만 보존한다. 사용자 CDP19222와 기존8080 기본 게임은 교체하지 않았다.

개발 미리보기: 기존 HTTP 서버의 `/preview-249/` 경로. 이 후보는 `development #249`로 표시한다. 공개 릴리즈/Pages는 기존 v0.3.4 그대로이며 새 버전 게시를 뜻하지 않는다.

## 검증과 재현

먼저 실패하는 재질/보간/화살표/자산 감사 테스트를 작성한 뒤 구현했다. transparent RGB halo·인접 atlas 오염·좌우/상하 표본 위상·투명 내부·청크/스크롤·지형 구멍을 확인했다. 기존 모든 지물의 프레임/원본 판정 불변 테스트도 적용한다.

| 검사 | 결과/범위 |
|---|---|
| 로컬 Python | `make check-python`: 344개 테스트·자산/라이선스/소스 핀 감사 통과 |
| WASM | `make check-wasm` 통과 |
| 활성 원화 감사 | 54종/540프레임,5재질,4배경,38PNG. 모든 키·사각형/해시 정상, 단색 표본 프레임0 |
| Native 원본120레벨 | 모든 초기 장면과 선택1,920틱 HD/원본 상태·mask·오디오 대조 |
| 실제 WASM/Canvas120레벨 | 352번 화면, 선택1,920틱,5테마/4특수/일방통행14장면, Native/WASM 최종 상태 동일·예외0 |
| 실제 SDL 전후11쌍 | 5,110개 상태 및 모든 capture protocol의 상태/입력/논리 RGB/지형·직업 mask 바이트 동일 |
| 저장 해법 | Fun01 10명, Fun07 28명, Mayhem08 72명 구출 성공. Fun07/Mayhem08 계단65/264픽셀 확인 |
| 자체5테마 코스 | Native 전후10실행, 강철/양방향 표지/깃발/시설 표시 확인 |
| 실제 Intel WebGPU | 일반 blend/clip/revision, 굴착·건설·폭파·결과, Mayhem08와 자체5테마 비교 통과. 원본 픽셀 일치, 장치 손실 복구·입력/상태/오디오 불변 |
| 일반 Wayland 제품 화면 | 개발 미리보기의 실제 WebGPU Mayhem08 80틱 표시, 브라우저 예외0 |

GPU/Canvas는 backend 필터의 정수화 차이를 허용한다. HD 비교의 최대 채널 차이는15/255, 평균 차이는0.176 미만이다. 이 검사는 모든120레벨 완주나 새 FPS 성능 인수를 주장하지 않는다. 결정 테마의 일방통행 표지는 기본120레벨에 없어 자체 코스로 검증했다.

```bash
# 새 ignored 출력 폴더를 사용한다.
python3 tools/hd_scenery_audit.py --output assets/local/issue-249/scenery-audit-final
cargo run -p lemmings-game --example hd_terrain_probe --release --locked --offline -- assets/local/web/hd-assets assets/local/dos-original
python3 tools/check_hd_scenery.py --before assets/local/issue-249/baseline/lemmings-desktop --after target/release/lemmings-desktop --assets assets/local/dos-original --pack assets/local/web/hd-assets --output assets/local/issue-249/native-pairs
python3 tools/check_webgpu.py --runtime assets/local/issue-249/runtime --evidence assets/local/issue-249/gpu-final --browser-arg=--enable-unsafe-webgpu --browser-arg=--enable-features=Vulkan --browser-arg=--use-angle=vulkan --browser-arg=--disable-vulkan-surface
```

baseline 실행파일은 변경 전 빌드를 복사한 것이다. Canvas120 검사는 기존 `tools/web/hd_terrain.cjs`에 `--decor`를 주어 실행했다. WebGPU 감사 도구는 실제 번들 데이터와 `scenery-courses/FILES.json`이 제공될 때 추가 코스를 검사하고, 없으면 그 범위를 명시적으로 건너뛴다.

주요 ignored 증거는 `scenery-audit-final/audit.json`, `native-catalog.jsonl`, `web-catalog/terrain-check.json`, `native-pairs/summary.json`, `gpu-final/summary.json`, `invented-native/summary.json`, `headed-product/summary.json`이다. 원화 manifest 집합 SHA-256은 `539dc58e391ace60aee99d095a330a10c8d00898a1265a0c922f20b523be3c5e`다.

## 배포·검토 상태

원화 PNG/manifest/atlas 개수와 다운로드 파일은 늘리지 않았다. 런타임 재질 합성과 보간만 변경했으며 새 PNG는 위 검토용 캡처뿐이다. 소스 인덱스는 현재 파일 해시를 갱신하고 이전 핀·역사 측정·#113 종료 결정을 보존한다.

로컬 최종 검사와 PR self-review/플랫폼 CI를 진행 중이다. 완료 후 동일 소스 결과와 병합 정보를 여기에 기록한다.
