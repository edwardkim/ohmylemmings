# G03-03 GROUND 형식·메타데이터·팔레트

작업 이슈: [GitHub #12](https://github.com/edwardkim/ohmylemmings/issues/12). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 고정 근거와 범위

- [Lemmix `Dos.Structures.pas`](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Dos.Structures.pas#L113-L172)의 `TDosMetaObject`, `TDosMetaTerrain`, `TDosGroundRec`: 슬롯·필드 배치와 little-endian words.
- 같은 파일의 [색상 변환 함수](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Dos.Structures.pas#L395-L412): 6→8비트 전 범위 변환.
- [Lemmings.ts `GroundReader`](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/lemmings/ground-reader.ts)의 `readObjectImages`, `readTerrainImages`, `readPalettes`: 객체 루프 비트·트리거 좌표와 팔레트 사용법.
- [TS `BinaryReader`](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/file/binary-reader.ts)의 `readWordBE`는 이름과 달리 **little-endian**입니다. Rust는 `u16::from_le_bytes`를 사용합니다.

원본·소스 매니페스트의 50개·46개 파일을 크기와 SHA-256으로 재검증했습니다. 이번 단계는 원시 GROUND 메타데이터와 색상 해석입니다. VGAGR 비트플레인·마스크, VGASPEC, 지형 합성은 후속 범위입니다.

## 파일 배치

`GROUND0O.DAT`~`GROUND4O.DAT`는 각각 **1056바이트**이며 압축 DAT 헤더가 없습니다. 기존 `DatSections`에 넘기지 않습니다.

| 파일 오프셋 | 크기 | 내용 |
|---|---:|---|
| 0 | 448 | 객체 16슬롯 × 28바이트 |
| 448 | 512 | 지형 64슬롯 × 8바이트 |
| 960 / 968 / 976 | 각 8 | EGA Custom / Standard / Preview |
| 984 / 1008 / 1032 | 각 24 | VGA Custom / Standard / Preview, RGB 8색 |

객체 내부 위치는 다음과 같습니다. word는 모두 부호 없는 little-endian u16입니다.

| 슬롯 내 오프셋 | 형식 | 필드 |
|---|---|---|
| 0 | word | 애니메이션 플래그 원값 |
| 2 / 3 / 4 / 5 | 각 byte | 시작 프레임 / 프레임 수 / 너비 / 높이 |
| 6 / 8 | 각 word | 프레임당 바이트 수 / 프레임 이미지 기준 마스크 상대 주소 |
| 10 / 12 | 각 word | 미해석 필드 1·2 |
| 14 / 16 | 각 word | 트리거 left / top 원값 |
| 18 / 19 / 20 | 각 byte | 트리거 너비 / 높이 / 효과 ID |
| 21 / 23 / 25 | 각 word | 이미지 주소 / 미리보기 이미지 주소 / 미해석 필드 3 |
| 27 | byte | 효과음 ID |

21·23·25의 word는 비정렬입니다. 미리보기 값은 프레임 인덱스로 바꾸지 않고 바이트 주소를 보존합니다. 루프는 TS처럼 `flags & 1 == 0`만 해석하고 다른 비트는 유지합니다. `trigger_rect()`는 `(left*4, top*4-4, width*4, height*4)`이며 top=0은 -4입니다. 좌표는 i32, 크기는 u16으로 확대하므로 모든 입력값에서 오버플로가 없습니다. 효과·음향 ID를 실행 동작으로 연결하지 않습니다.

지형 슬롯은 `width:u8`, `height:u8`, `image_offset:u16`, `mask_offset:u16`, `unknown:u16` 순입니다. 객체 마스크와 달리 지형 마스크는 VGAGR 지형 구간 기준 **절대 주소**입니다. 파싱 중 빼기나 상대 주소 변환을 하지 않습니다.

## API와 검증 정책

`GroundRecord::parse(&bytes)`는 정확한 길이와 960~1055의 전체 팔레트 바이트가 0~63인지 검사합니다. `GroundError::Length`는 실제 길이, `PaletteValue`는 파일 기준 바이트 위치와 값을 제공합니다. 이 여섯 비트 범위 검사는 이 구현의 입력 정책이며 레퍼런스가 수행하지 않는 검증도 포함합니다.

원문 배열을 빌려 `as_bytes()`로 돌려주며, `object(slot)`·`terrain(slot)`은 범위 밖에서 `None`입니다. 인덱스 곱셈 전에 범위를 검사하므로 `usize::MAX`도 안전합니다. `objects()`·`terrains()`는 모든 16·64슬롯을 순서대로 제공하는 exact-size·fused iterator입니다. `is_empty()`는 **모든 디스크 필드가 0**일 때만 참입니다. 너비나 프레임 수가 0이라는 이유로 슬롯을 누락하지 않습니다.

미해석 값·플래그·치수·주소를 보존합니다. 치수, 프레임 수·시작 프레임, 프레임 크기, 주소 범위와 마스크 일관성은 여기서 유효한 이미지라고 인증하지 않습니다. VGAGR와 결합할 때 별도 검증해야 합니다. GROUND 안에는 그래픽 세트 번호나 출처 인증값이 없으므로 파일명·레벨 그래픽 번호 연결은 호출자 책임이며 해시는 매니페스트로 확인합니다.

두 라이브러리는 계속 `no_std`·무할당입니다. 기존 리소스→코어 로컬 의존만 유지하고 I/O·할당·JSON 출력은 호스트 예제에 둡니다.

## 팔레트와 레퍼런스 차이

`GroundPalette::{Custom, Standard, Preview}`로 각 뱅크를 명시합니다. `ega_palette()`는 여섯 비트 하드웨어 선택자 8개를 빌려주며 VGA 색상 인덱스나 RGB로 해석하지 않습니다. `vga_palette()`는 검증된 `Rgb6` 8개를 값으로 반환합니다. `components()`로 원래 RGB를 얻습니다.

RGB 확대 방식은 서로 다르므로 하나로 숨기지 않습니다.

| API | 근거·계산 | 입력 63 |
|---|---|---:|
| `to_rgb8_shifted()` | TS, `component << 2` | 252 |
| `to_rgb8_full_range()` | Lemmix, `floor(component*255/63)` | 255 |

TS는 지형에 Custom 8색, 객체 16색에 Standard 8색 + Preview 8색을 사용합니다. 이 단계는 세 뱅크를 그대로 노출하고 렌더러의 최종 확대 방식·투명도·알파를 결정하지 않습니다. 원본의 VGA 성분 최댓값은 60입니다. Custom과 Preview가 같은 경우가 있어도 별개 뱅크로 보존합니다. Dirt 세트의 두 뱅크 첫 색상은 실제로 다릅니다.

## 원본 검사 재현

[리소스 확보](../../assets/README.md) 후 저장소 루트에서 실행합니다. 검사 예제는 여러 GROUND 파일을 받아 파일당 JSONL 한 줄을 출력합니다. `file_index`는 **인자 순서**이며 그래픽 세트를 인증하지 않습니다. 각 객체의 `fields`는 슬롯 번호 다음에 디스크 필드 순서(위 표, unknown 3은 오프셋 25 위치)이며 지형도 슬롯 번호 다음에 디스크 필드 순서입니다.

```sh
python3 tools/reference_assets.py verify
cargo run -p lemmings-resources --example inspect_ground --locked --offline -- \
  assets/local/dos-original/GROUND[0-4]O.DAT > assets/local/issue-12_ground.jsonl
python3 - <<'PY'
import json
import struct
from pathlib import Path
rows = [json.loads(line) for line in Path('assets/local/issue-12_ground.jsonl').read_text().splitlines()]
assert len(rows) == 5
active_objects = active_terrain = 0
for i, row in enumerate(rows):
    b = Path(f'assets/local/dos-original/GROUND{i}O.DAT').read_bytes()
    expected = {'file_index': i, 'objects': [], 'terrain': [], 'palettes': []}
    for slot in range(16):
        v = struct.unpack_from('<H4B6H3B3HB', b, slot * 28)
        expected['objects'].append({'fields': [slot, *v], 'empty': not any(v),
            'loops': v[0] & 1 == 0, 'trigger': [v[9]*4, v[10]*4-4, v[11]*4, v[12]*4]})
    for slot in range(64):
        v = struct.unpack_from('<BBHHH', b, 448 + slot * 8)
        expected['terrain'].append({'fields': [slot, *v], 'empty': not any(v)})
    for bank in range(3):
        colors = [list(b[984+bank*24+j*3:987+bank*24+j*3]) for j in range(8)]
        expected['palettes'].append({'ega': list(b[960+bank*8:968+bank*8]), 'vga': colors,
            'shifted': [[v*4 for v in c] for c in colors],
            'full_range': [[v*255//63 for v in c] for c in colors]})
    assert row == expected, i
    active_objects += sum(not v['empty'] for v in row['objects'])
    active_terrain += sum(not v['empty'] for v in row['terrain'])
assert (active_objects, active_terrain) == (54, 273)
print('PASS: all 80 object slots, 320 terrain slots, 120 EGA selectors and 120 VGA colors')
PY
```

JSONL에는 모든 원시 필드, 빈 슬롯 여부, 루프·트리거 계산, EGA·VGA 원값과 두 RGB 확대 결과가 포함됩니다. Rust 결과와 독립 Python `struct` 해석 전체를 비교합니다. 원본과 JSONL 모두 무시되는 `assets/local/`에 보관합니다. 이 대조는 이미지 복원이나 실제 DOS 실행 검증을 뜻하지 않습니다.

결과: [보고서](../reports/issue-12_ground_metadata.md), [진행 기록](../working/issue-12_ground_metadata.md), [작업일지](../orders/20261018.md#issue-12).
