# G11-03 특수 장면 연결 계약

이슈: [#89](https://github.com/edwardkim/ohmylemmings/issues/89) · [G11 계획](../plans/issue-85_compatibility_validation.md) · [결과](../reports/issue-89_special_scenes.md).

## 자원 선택과 소유권

[G11-02 LevelId](issue-87_rank_selection.md)의 공통 로더에서 special 거부를 제거했다. 레벨 헤더 special은 1 기반, 파일 이름은 0 기반이다. [기존 VGASPEC 계약](issue-14_vgaspec_backgrounds.md)을 그대로 사용하며 새 포맷·외부 의존성은 없다.

| 표시 ID | 헤더 special | 필요한 특수 파일 |
| --- | --- | --- |
| Fun22 | 4 | VGASPEC3.DAT |
| Tricky14 | 2 | VGASPEC1.DAT |
| Taxing15 | 1 | VGASPEC0.DAT |
| Mayhem22 | 3 | VGASPEC2.DAT |

특수 DAT의 framing/XOR/단일 구간·해제와 inner palette/RLE를 검증한다. 소유한 inner record를 Assets에 보존하고 초기 충돌 마스크와 매 렌더에 같은 `compose_level_terrain` 경로를 사용한다. 파싱·simulation·플랫폼 경계와 resources/core의 no_std 계약은 유지한다. 반복 렌더의 RLE 해제/할당은 현재 명시적 선택이며 캐시·성능 개선은 G11-06 측정 후 판단한다.

## 지형·객체·다리

[기존 합성](issue-15_level_terrain.md)처럼 960×160 그림을 월드 (304,0)에 배치하고 일반 terrain placements는 적용하지 않는다. 충돌은 RGB가 아닌 픽셀 불투명도로 만든다. 색0은 RGB가 검정이 아니어도 투명하고, 다른 색의 검정은 고체다. 객체·강철·일방향·트랩은 기존 레벨 레코드/GROUND 경로다.

[객체 계약](issue-19_level_objects.md)의 VGASPEC terrain RGB6<<2와 객체 상위색 RGB6×255/63은 서로 다르다. 게임/객체 팔레트8과 다리 팔레트7은 특수 장면에서 (124,124,0)이다. 지형 변경 후에도 배경 팔레트를 보존하며, 다리는 지형 provenance로 객체의 only-on-terrain/no-overwrite 합성에 참여한다. 최종 노출 다리 픽셀만 palette7을 쓰고 객체가 가리면 객체 RGB를 쓴다. 삭제된 다리 마스크는 그림을 남기지 않는다.

초기 화면과 정적 GROUND first-frame의 차이를 구분한다. 네 원본 모두 기존 호스트의 연속 객체 +2 위상 때문에 불꽃21픽셀이 다르다. iteration4로 위상을 맞추면 첫 프레임과 전체 RGBA가 같고 입구는 아직 first-frame이다. 이 비교는 실제 DOS 새 실행 증거가 아니다.

## 호스트·음향·교체 실패

SDL은 기존 파일 어댑터를 재사용한다. WebAssets는 VGASPEC0..3.DAT만 추가 허용하며 대소문자 정규화·중복 거부·1MiB 상한·로드된 bank의 불변 스냅샷을 유지한다. 제품 UI의 같은 선택/재시작/Continue 경로로 특수 장면을 연다.

[기존 음향](issue-78_game_sound.md)의 special1..4 raw music ID는 [1,9,3,2], 0 기반 [0,8,2,1]이다. Assets의 헤더 값을 그대로 전달한다. 실패한 누락/손상 파일 교체는 현재 세션·화면·소수 시계·음원 세대를 유지하고, 성공한 재시작/전환만 새 세대를 만든다.

## 재현

```sh
source assets/local/dev-tools/env.sh
make check-rust check-python check-wasm
python3 tools/check_web.py
python3 tools/check_web.py --original-assets assets/local/dos-original --evidence assets/local/issue-89-review-reproduce
cargo run --release --locked --offline -p lemmings-desktop -- --assets assets/local/dos-original --difficulty fun --level 22
```

일반 검사에는 창작 VGASPEC4개와 실제 UI file input/Continue를 포함하며 원본이 필요 없다. 원본 검사 출력과 화면은 ignored assets/local에 둔다. [검증 목록](../../assets/manifests/special_scenes.json)에 공개 입력·소스·개별 실행·정적 대조의 수치/해시를 기록한다. 전체120개 초기340경계/3회/유효 해법 완주는 G11-04/05, 성능은 G11-06 후속이다.
