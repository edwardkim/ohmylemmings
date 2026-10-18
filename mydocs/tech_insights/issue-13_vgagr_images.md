# G03-04 VGAGR 비트플레인·마스크·이미지 경계

작업 이슈: [GitHub #13](https://github.com/edwardkim/ohmylemmings/issues/13). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 고정 근거

- Lemmix `40e9bc34451f0e9127fd53b290a3877e700d143d`: [Dos.Bitmaps.pas의 `TDosPlanarBitmap.GetByteMap`](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Dos.Bitmaps.pas#L126-L188)은 행 너비/8, 플레인 간격, MSB부터 읽는 픽셀과 낮은 색상 비트부터의 플레인 순서를 정의합니다.
- Lemmings.ts `10c7c8b9167258503d02589cf048e2d83a43dd0a`: [PaletteImage](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/lemmings/palette-image.ts)의 `readImageData`·`setTransparencyByData`, [GroundReader](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/lemmings/ground-reader.ts)의 `readImages`, [LevelLoader](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/level-loader.ts)의 두 VGAGR 구간 연결을 대조했습니다.
- 필드·엔디언·팔레트는 선행 [GROUND 분석](issue-12_ground_metadata.md)을 사용합니다. 원본 50개·기존 고정 소스 46개의 크기·SHA-256을 재검증했습니다.

추가로 읽은 `Dos.Bitmaps.pas`(21999바이트)와 `palette-image.ts`(3258바이트)의 고정 커밋·SHA-256은 [이미지 검증 매니페스트](../../assets/manifests/vgagr_images.json)에 기록했습니다. Pascal 소스는 raw 다운로드의 Git blob SHA-1도 tree와 대조했습니다. 기존 입력 매니페스트는 변경하지 않았습니다.

## 이미지 형식

압축 `VGAGR0.DAT`~`VGAGR4.DAT`는 각각 두 DAT 구간입니다. 구간 0은 지형, 구간 1은 객체입니다. [G02 해제기](issue-9_dat_decompression.md)로 해제한 뒤 GROUND의 슬롯 주소를 적용합니다.

너비 W, 높이 H일 때 한 플레인은 `(W/8)*H`바이트입니다. 행은 위에서 아래로, 바이트의 최상위 비트는 왼쪽 픽셀입니다. 연속한 플레인 0·1·2·3이 색상 비트 0·1·2·3을 만듭니다. 지형은 색상 3플레인, 객체는 4플레인이며 마스크는 별도 1플레인입니다. 마스크 1은 불투명입니다. 색상 0도 불투명할 수 있고 투명 픽셀의 색상 인덱스도 보존합니다.

| 종류 | 이미지 시작 | 마스크 시작 | 프레임 이동 |
|---|---|---|---|
| 지형 | `image_offset` | 절대 `mask_offset` | 단일 이미지 |
| 객체 프레임 f | `image_offset + f*frame_size` | 해당 시작 + 상대 `mask_offset` | GROUND의 `frame_size` |
| 객체 미리보기 | 절대 `preview_offset` | 미리보기 시작 + 상대 `mask_offset` | 별도 주소 검사 |

시작 프레임은 초기 표시용 메타데이터입니다. `frame(0)`은 디스크의 첫 프레임이며 시작 프레임을 적용해 순서를 회전하지 않습니다. 미리보기는 독립 이미지 주소로 검사하며 애니메이션 프레임 경계라는 조건을 강제하지 않습니다.

## API와 오류 정책

- `VgagrFile::parse(bytes)`: 전체 DAT 프레이밍·XOR와 정확한 두 구간을 확인합니다. 추가 구간·손상된 후미도 검사하며 명령 해제는 이후 수행합니다.
- `decoded_sizes()`: 두 구간의 선언 해제 크기입니다. `decode_into(ground, terrain_buf, object_buf)`는 **두 버퍼 길이 모두** 확인한 뒤 해제합니다. 길이 오류는 양쪽 출력을 보존하며 이후 해제·메타데이터 오류에서는 둘 다 버립니다.
- `VgagrImages::new(ground, terrain, objects)`: 이미 해제한 두 구간을 결합할 때 사용합니다. 모든 64·16슬롯과 객체의 모든 프레임·미리보기를 검증합니다. 전부 0인 슬롯만 비어 있으며, 미해석 필드만 0이 아니어도 이미지 검증 대상입니다.
- `terrain(slot)`·`object(slot)`: 빈 슬롯은 `Ok(None)`, 범위 밖은 명시적 오류입니다. 객체는 `ObjectFrames`로 프레임 수·초기 프레임·`frame(index)`·`preview()`를 제공합니다.
- `PlanarImage::pixel(x,y)`: 경계 밖은 `None`입니다. `decode_into(&mut [IndexedPixel])`는 정확히 W×H개가 필요하며 길이 오류에서 출력을 바꾸지 않습니다. 각 픽셀의 `color`와 `opaque`는 별개입니다.

지원하는 DOS 원작 레이아웃의 검증 정책은 양수 높이·8의 배수인 양수 너비, 객체 프레임 수>0·초기 프레임<프레임 수, 색상 데이터 뒤의 마스크, 마스크 끝 이상인 객체 프레임 간격입니다. 색상/마스크 사이와 프레임 뒤의 패딩은 허용합니다. 마지막 프레임 뒤의 사용하지 않는 패딩까지 요구하지는 않습니다. 모든 색상·마스크 창은 해당 해제 구간 안에 있어야 합니다. 슬롯 간 이미지 공유를 금지하지 않습니다.

`VgagrError`는 DAT·구간 수·출력 버퍼·해제·슬롯·치수·프레임·마스크·이미지 창·픽셀 출력 오류를 구분합니다. 원인 DAT/해제 오류는 `source()`로 보존합니다. `ImageRange`의 주소는 **해제 구간 기준**이고 `frame=Some(f)`는 객체 프레임, `None`은 지형 또는 객체 미리보기입니다. `MaskOffset`은 지형에서 절대 주소, 객체에서 상대 주소입니다.

호출자 인덱스는 곱셈 전에 확인하고, 창 경계는 길이에서 오프셋을 뺀 값으로 검사합니다. GROUND의 u8/u16 크기와 프레임 주소 계산은 네이티브·WASM 32비트 범위 안입니다. 두 라이브러리는 계속 `no_std`·무할당이며 코어의 리소스 역방향 의존은 없습니다.

## 팔레트와 범위의 한계

지형 `color`는 Custom 8색의 로컬 인덱스, 객체는 Standard+Preview 16색의 인덱스입니다. 이 단계는 RGB 확대·알파 합성·최종 렌더러를 선택하지 않습니다. Lemmix 내부에도 앞 단계에서 조사한 전 범위 확대 함수와 별도로 `Dos.Bitmaps.pas`의 2비트 왼쪽 이동 변환이 있으므로 저장소 이름만으로 색상 확대 정책을 하나로 가정하지 않습니다.

모든 원본 너비는 8의 배수였습니다. TS는 비트 버퍼를 계속 읽지만 Lemmix는 행 너비/8을 사용하므로 비정렬 너비를 추정 지원하지 않습니다. 레퍼런스의 느슨한 EOF 처리도 복제하지 않고 완전한 창을 요구합니다. 파일명·그래픽 세트 연결과 원본 인증은 호출자·매니페스트 책임입니다. 경계 검증은 잘못 결합한 파일의 의미나 슬롯 간 주소 소유권까지 인증하지 않습니다.

VGASPEC·전체 레벨 지형 합성·충돌 모델·DOS 실행 호환성은 후속 범위입니다.

## 원본 대조와 재현

원본 5세트에서 지형 273개·객체 애니메이션 540프레임·미리보기 54개, 총 **867개 이미지·771664픽셀**을 검사했습니다. Rust 출력 전체를 독립 Python의 DAT 해제·플레인별 이진 문자열 해석과 비교했습니다. 별도로 고정 TS의 BinaryReader·비트 읽기/쓰기·DAT 해제·FileContainer·PaletteImage를 실행해 전체 픽셀 바이트를 대조했습니다. TS 하네스는 import를 제거하고 로깅만 대체했으며 픽셀·해제 알고리즘은 유지했습니다. Node 24.11.0의 `--experimental-transform-types`를 사용했고 npm 패키지는 설치하지 않았습니다.

10개 해제 구간의 크기·SHA-256도 Python·TS·기존 G02-02 결과가 일치합니다. 검증 후 세트별 픽셀 연결 해시와 구간 해시를 새 매니페스트에 고정했습니다. 이 파일은 확보용 `reference_assets.py --manifest` 입력이 아닙니다.

[원본 확보](../../assets/README.md) 후 저장소 루트에서 실행합니다. `inspect_vgagr`는 5세트의 압축 원본을 직접 읽고 검증·해제합니다. 출력은 원본에서 파생된 픽셀이므로 무시되는 로컬 폴더에 둡니다.

```sh
python3 tools/reference_assets.py verify
cargo run -p lemmings-resources --example inspect_vgagr --locked --offline -- \
  assets/local/dos-original > assets/local/issue-13_vgagr.jsonl
python3 - <<'PY'
import hashlib
import json
import struct
from pathlib import Path
contract = json.loads(Path('assets/manifests/vgagr_images.json').read_text())
for entry in contract['inputs']:
    assert hashlib.sha256(Path(entry['path']).read_bytes()).hexdigest() == entry['sha256']
rows = [json.loads(line) for line in Path('assets/local/issue-13_vgagr.jsonl').read_text().splitlines()]
assert len(rows) == 867 and {r['set'] for r in rows} == set(range(5))
for expected in contract['sets']:
    set_id = expected['set']
    ground = Path(f'assets/local/dos-original/GROUND{set_id}O.DAT').read_bytes()
    geometry = []
    for slot in range(64):
        v = struct.unpack_from('<BBHHH', ground, 448+slot*8)
        if any(v):
            geometry.append(('terrain', slot, None, v[0], v[1]))
    objects = 0
    for slot in range(16):
        v = struct.unpack_from('<H4B6H3B3HB', ground, slot*28)
        if any(v):
            objects += 1
            geometry.extend(('object', slot, f, v[3], v[4]) for f in range(v[2]))
            geometry.append(('preview', slot, None, v[3], v[4]))
    actual = [r for r in rows if r['set'] == set_id]
    assert len(actual) == len(geometry) and objects == expected['object_count']
    digest = hashlib.sha256()
    count = 0
    for row, shape in zip(actual, geometry):
        assert tuple(row[k] for k in ['kind', 'slot', 'frame', 'width', 'height']) == shape
        pixels = bytes.fromhex(row['pixel_hex'])
        assert len(pixels) == row['width']*row['height']
        limit = 8 if row['kind'] == 'terrain' else 16
        assert all((v & 0x70) == 0 and (v & 0x0f) < limit for v in pixels)
        digest.update(pixels)
        count += len(pixels)
    for kind, key in [('terrain', 'terrain_count'), ('object', 'frame_count'), ('preview', 'preview_count')]:
        assert sum(r['kind'] == kind for r in actual) == expected[key]
    assert count == expected['pixel_count'] and digest.hexdigest() == expected['pixel_sha256']
print('PASS: 867 images/frames/previews, 771664 pixels, geometry and all five frozen pixel hashes')
PY
```

JSONL 순서는 세트 번호 → 사용 지형 슬롯 → 사용 객체 슬롯별 전체 프레임과 미리보기입니다. `frame=null`은 지형·미리보기이며 `kind`로 구분합니다. `pixel_hex`의 각 바이트는 하위 비트에 색상 인덱스, 비트 7에 투명 여부(1=투명)를 담습니다. 이 인코딩은 TS 비교·호스트 출력용이며 라이브러리 픽셀 API는 별도 bool을 제공합니다.

결과: [보고서](../reports/issue-13_vgagr_images.md), [진행 기록](../working/issue-13_vgagr_images.md), [작업일지](../orders/20261018.md#issue-13).
