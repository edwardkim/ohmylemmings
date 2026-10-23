# 2D 리마스터 디자인 자산

담당 [#137](https://github.com/edwardkim/ohmylemmings/issues/137), 첫 시안 [#138](https://github.com/edwardkim/ohmylemmings/issues/138).

- [Fun 01 손그림풍 시안](concepts/issue-138_fun01_handpainted_v1.png):1672×941 디자인 후보. 실제 HD 모드·충돌 mask·분리 atlas가 아니다.
- [생성 프롬프트](prompts/issue-138_fun01_handpainted_v1.txt): built-in image_gen에 사용한 최종 문구.
- [출처/규격/해시와 전체 제작 대응 목록](../manifests/remaster_art_direction.json).
- [아트/표시 계약](../../mydocs/plans/issue-138_hd_art_contract.md), [결과·검토 한계](../../mydocs/reports/issue-138_hd_art_direction.md).

이 폴더에는 새로 생성한 디자인 후보를 보관한다. 원본 캡처/바이너리는 Git에 넣지 않으며 ignored assets/local에서만 참조한다. 원본 데이터·캐릭터/상표의 권리와 기존 제3자 고지에 새 허가를 부여하지 않는다. 게임 글자·숫자·배치·물리 경계는 생성 그림 대신 명시한 계약과 실제 세션을 따른다.

[#144 HD-03a](https://github.com/edwardkim/ohmylemmings/issues/144)에서 [걷기16프레임](issue-144-walk-v1.png)과 [암석 재질](issue-144-rock-v1.png)을 별도 PNG로 제작했다. [팩 manifest](../manifests/remaster_pack.json)에 원본 slot/frame·실제 사각형·표시 크기/발 기준점과 hash를 연결한다. [걷기 프롬프트](prompts/issue-144_walk_v1.txt), [암석 프롬프트](prompts/issue-144_rock_v1.txt), [결과/재현 명령](../../mydocs/reports/issue-144_hd_assets.md)을 참고한다.

선택 PNG는 수정 없이 보존한다. 걷기 atlas의alpha1 잔여는 팩의 명시적 cutoff1에 따라 표시 시 제외하며, 나머지 alpha와 픽셀은 유지한다. 보행/색 번짐의 실제 게임 검토와 암석의 반복 경계 인증은 후속이다. 팩 후보의2애니메이션/16프레임과 실제 게임 통합 완료를 구분한다. HD-03a 당시 `runtime_ready=false`·통합0개였으며 현재 상태는 아래 HD-05a를 따른다.

[자산 미리보기 화면](previews/issue-144_assets.png)은 실제 브라우저의 PNG/발 기준점 검토 화면이다. 게임 플레이 캡처가 아니며, 재생 도구는 위 결과 보고서의 명령으로 만든다.

[#147 HD-03b](https://github.com/edwardkim/ohmylemmings/issues/147)에서 [굴착](issue-147-dig-v1.png)·[계단 좌우](issue-147-build-v1.png)·[OhNo/폭발](issue-147-bomb-v1.png)·[입출구](issue-147-facilities-v1.png)·[13조작 아이콘](issue-147-ui-v1.png)을 추가했다. 정확한5개 프롬프트·실제 규격/해시·65개 새 캐릭터 프레임 대응은 [결과 보고서](../../mydocs/reports/issue-147_hd_actions.md)와 현재 팩 manifest를 따른다. HD-03b 당시 후보는7애니메이션81캐릭터 프레임·11시설 프레임·13아이콘/7atlas106자산이다. 당시 미제작이던21애니메이션256프레임은 아래 HD-05a에서 연결했으며, 다른 테마·시설/효과·전체 UI는 후속이다.

[동작/아이콘 미리보기](previews/issue-147_actions.png), [입구 미리보기](previews/issue-147_facilities.png)는 실제 PNG 검토 화면이다. 새 PNG도 원본 그대로 보존하며 일부 최대alpha254·여백 잔여cutoff1/4를 명시한다. 캐릭터 크기/위상과 출구 doorway·trigger 정합은 실제 게임 통합에서 검토한다. 이는 HD-03b 완료 당시 `runtime_ready=false`·통합0개의 기록이다.

[#151 HD-04](https://github.com/edwardkim/ohmylemmings/issues/151)에서 Native/Web의 실제 HD 미리보기와 원본 모드·입력을 연결했다. 현재 `runtime_ready=true`는 **부분 HD 미리보기 사용 가능**을 뜻하며 `full_remaster_ready=false`다. 원본 시안 이미지는 계속 제작 참고용이다. 원본 입구의1닫힘→2~9열기→0열림 유지에 맞춰 자산 프레임 대응도 보정했다. [실제 렌더러의 자체 합성 장면](previews/issue-151_native_synthetic.png)은 원본 바이너리를 사용하지 않은 캡처이며, 실제 Fun01 캡처는 ignored assets/local에 보존한다. [결과/남은 원본 동작·효과](../../mydocs/reports/issue-151_hd_integration.md)를 참고한다.

## #154 HD-05a 전체 캐릭터 프레임

현재 Native/Web 미리보기는28동작337논리 프레임 모두 새 캐릭터 PNG를 사용한다. built-in image_gen으로 선택한 새12장/256포즈와 실제 사각형·방향/발 기준점·PNG 해시는 [대응 목록](../manifests/hd_actors.json), [팩](../manifests/remaster_pack.json), [결과/재현](../../mydocs/reports/issue-154_hd_actors.md)에 기록한다. PNG의 RGB/alpha 샘플을 편집/재표본화하지 않았고 각 새 atlas의 cutoff8은 표시할 때만 적용한다.

| 자산 | PNG | 정확한 생성 프롬프트 | 선택 포즈 |
|---|---|---|---|
| air | [PNG](issue-154-air-v1.png) | [prompt](prompts/issue-154_air_v1.txt) | 10 |
| climb | [PNG](issue-154-climb-v1.png) | [prompt](prompts/issue-154_climb_v1.txt) | 32 |
| bash | [PNG](issue-154-bash-v1.png) | [prompt](prompts/issue-154_bash_v1.txt) | 56 |
| bashleft2 | [PNG](issue-154-bashleft2-v1.png) | [prompt](prompts/issue-154_bashleft2_v1.txt) | 8 |
| mine | [PNG](issue-154-mine-v1.png) | [prompt](prompts/issue-154_mine_v1.txt) | 38 |
| minefix | [PNG](issue-154-minefix-v1.png) | [prompt](prompts/issue-154_minefix_v1.txt) | 9 |
| mineleft | [PNG](issue-154-mineleft-v1.png) | [prompt](prompts/issue-154_mineleft_v1.txt) | 1 |
| umbrella | [PNG](issue-154-umbrella-v1.png) | [prompt](prompts/issue-154_umbrella_v1.txt) | 16 |
| block | [PNG](issue-154-block-v1.png) | [prompt](prompts/issue-154_block_v1.txt) | 32 |
| exitdrown | [PNG](issue-154-exitdrown-v1.png) | [prompt](prompts/issue-154_exitdrown_v1.txt) | 24 |
| splat | [PNG](issue-154-splat-v1.png) | [prompt](prompts/issue-154_splat_v1.txt) | 16 |
| vapor | [PNG](issue-154-vapor-v1.png) | [prompt](prompts/issue-154_vapor_v1.txt) | 14 |

[28동작 연락표](previews/issue-154_actors.png)는 실제 브라우저 Canvas 캡처이고 [우산 SDL 장면](previews/issue-154_native_floater.png)은 자체 합성 지형의 실제 Native 출력이다. 원본 캡처는 공개하지 않는다. 캐릭터 키의 누락 없음은 전체 리마스터 완료와 다르며 다른 테마/시설/효과/글꼴과 지원 배포는 HD-05 후속/HD-06이다.128MiB/32atlas 한계로 확장한 팩을 사용하려면 현재 Native/Web 빌드를 사용한다. 기존 v1 소형 팩은 계속 읽을 수 있다.

## #157 HD-05b 일반 지형 재질

일반 세트0~4에 HD 재질을 연결하고 원본 일방통행 화살표를 유지했다. 아래4장과 기존 [세트0 암석](issue-144-rock-v1.png)을 사용한다. built-in image_gen으로 생성했으며 PNG 샘플은 수정/재표본화하지 않았다.

| 세트 | PNG | 정확한 생성 프롬프트 |
|---|---|---|
| 1 · 화산암 | [PNG](issue-157-fire-v1.png) | [prompt](prompts/issue-157_fire_v1.txt) |
| 2 · 대리석 | [PNG](issue-157-marble-v1.png) | [prompt](prompts/issue-157_marble_v1.txt) |
| 3 · 사암 | [PNG](issue-157-pillar-v1.png) | [prompt](prompts/issue-157_pillar_v1.txt) |
| 4 · 결정 | [PNG](issue-157-crystal-v1.png) | [prompt](prompts/issue-157_crystal_v1.txt) |

[반복 재질 검토 화면](previews/issue-157_materials.png)은 실제 자산 Canvas 캡처다. [출처/해시·검증](../manifests/hd_terrain.json)과 [결과/재현](../../mydocs/reports/issue-157_hd_terrain.md)을 참고한다. 런타임 팩은 참조 사각형의 RGBA를 그대로 재배치해23atlas366자산/109,503,312바이트로 정리했다. 제작 PNG와 manifest의 원본 좌표는 보존한다. 원본 지형 형상은 유지하며273개 조각별 재디자인·특수4장면·나머지 시설/효과/전체 UI·지원 배포는 후속이다.

## #160 HD-05c 전체 테마 입출구·출구 불꽃

5테마 입출구55프레임과 순수 장식30프레임을 Native/Web 미리보기에 연결했다. 아래9장을 built-in image_gen으로 생성했다. 실제 사각형·원본 객체 메타데이터·위상·기준점·출처와 해시는 [대응 목록](../manifests/hd_portals.json), [팩](../manifests/remaster_pack.json), [결과/재현](../../mydocs/reports/issue-160_hd_portals.md)에 기록한다. 선택 PNG의 RGB/alpha는 수정하거나 재표본화하지 않았고 cutoff8은 표시할 때만 적용한다.

| 자산 | PNG | 정확한 생성 프롬프트 |
|---|---|---|
| fire 입출구 | [PNG](issue-160-fire-v1.png) | [prompt](prompts/issue-160_fire_v1.txt) |
| marble 입출구 | [PNG](issue-160-marble-v1.png) | [prompt](prompts/issue-160_marble_v1.txt) |
| pillar 입출구 | [PNG](issue-160-pillar-v1.png) | [prompt](prompts/issue-160_pillar_v1.txt) |
| crystal 입출구 | [PNG](issue-160-crystal-v1.png) | [prompt](prompts/issue-160_crystal_v1.txt) |
| dirt 불꽃 | [PNG](issue-160-flame-dirt-v1.png) | [prompt](prompts/issue-160_flame_dirt_v1.txt) |
| fire 불꽃 | [PNG](issue-160-flame-fire-v1.png) | [prompt](prompts/issue-160_flame_fire_v1.txt) |
| marble 불꽃 | [PNG](issue-160-flame-marble-v1.png) | [prompt](prompts/issue-160_flame_marble_v1.txt) |
| pillar 불꽃 | [PNG](issue-160-flame-pillar-v1.png) | [prompt](prompts/issue-160_flame_pillar_v1.txt) |
| crystal 불꽃 | [PNG](issue-160-flame-crystal-v1.png) | [prompt](prompts/issue-160_flame_crystal_v1.txt) |

[5테마 출입구 연락표](previews/issue-160_portals.png), [자체 제작 코스의 실제 Web Canvas](previews/issue-160_web_course.png), [동일 코스의 실제 Native SDL](previews/issue-160_native_course.png)을 확인할 수 있다. 원본 맵 캡처는 ignored에 보존한다. 전체 리마스터와 지원 HD 배포는 후속이다.

[#163 팩 정리](../../mydocs/reports/issue-163_hd_pack.md): 원본32PNG·440자산을 유지하고 런타임27atlas/131,699,864바이트로 정리했습니다. 필터링에 민감한 페이지는 기존 배치를 유지하며 Native/Canvas 화면 동일성을 검증했습니다. 전체 리마스터·성능 인수는 후속입니다.
