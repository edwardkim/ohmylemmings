# G03-05 VGASPEC 특수 배경·팔레트·RLE

작업 이슈: [GitHub #14](https://github.com/edwardkim/ohmylemmings/issues/14). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 고정 근거

- Lemmix `40e9bc34451f0e9127fd53b290a3877e700d143d`: [Dos.Structures.pas의 `TDosVgaSpecPaletteHeader`](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Dos.Structures.pas#L176-L181), [Dos.Bitmaps.pas의 `GetSectionsAndPalette`·`LoadFromStream`](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Dos.Bitmaps.pas#L267-L462): 헤더, RLE, 4개 스트립의 플레인·이미지 결합.
- Lemmings.ts `10c7c8b9167258503d02589cf048e2d83a43dd0a`: [VgaSpecReader의 `read`·`readImage`·`readPalettes`](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/lemmings/vga-spec-reader.ts), [PaletteImage](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/lemmings/palette-image.ts): 두 단계 해제, 색상 인덱스·투명도.

원본 50개·고정 소스 46개를 크기·SHA-256으로 재검증했습니다. 앞 단계에서 추가 확보한 `Dos.Bitmaps.pas`·`palette-image.ts`도 고정 해시와 대조했습니다. [검증 매니페스트](../../assets/manifests/vgaspec_backgrounds.json)에 입력 해시와 추가 소스 식별값을 기록하며 기존 매니페스트는 변경하지 않았습니다.

## 두 단계 형식

`VGASPEC0.DAT`~`VGASPEC3.DAT`는 각각 DAT 구간 하나입니다. [기존 DAT 해제기](issue-9_dat_decompression.md)를 적용하면 다음 내부 레코드가 나옵니다.

| 내부 오프셋 | 크기 | 내용 |
|---|---:|---|
| 0 | 24 | VGA RGB 8색, 성분당 여섯 비트 |
| 24 | 8 | EGA 하드웨어 선택자 |
| 32 | 8 | 미해석 헤더 원값 |
| 40 | 가변 | 순서대로 4개 RLE 스트립 |

앞 32바이트는 0~63을 요구합니다. 마지막 8바이트는 의미를 단정하거나 범위를 제한하지 않고 그대로 제공합니다. VGA는 [기존 `Rgb6`](issue-12_ground_metadata.md)의 원값·명시적 두 확대 방식을 재사용합니다.

각 스트립은 960×40 픽셀의 색상 3플레인, 즉 **14400바이트**로 해제됩니다. 스트립 4개를 위에서 아래로 쌓아 **960×160** 배경을 만듭니다. 플레인마다 4800바이트이며 낮은 색상 비트부터, 각 바이트는 MSB를 왼쪽 픽셀로 읽습니다. 전체 플레인 버퍼는 `스트립 0의 3플레인 → 스트립 1의 3플레인 → …` 순서입니다. 전체 이미지의 같은 색상 비트끼리 모은 배열이 아닙니다.

| 명령 바이트 C | 동작 |
|---|---|
| 0~127 | 뒤의 C+1바이트를 그대로 복사 |
| 128 | 스트립 종료; 정확히 14400바이트를 생성했어야 함 |
| 129~255 | 뒤의 한 바이트를 257-C번 반복 |

128은 무시하는 명령이 아닙니다. 복사 데이터·반복 값 안의 128은 일반 데이터입니다. 패킷은 스트립 경계를 넘을 수 없고 네 번째 종료 표식 뒤에는 바이트가 없어야 합니다. 스트립 간 암묵적 패딩·이전 스트립 데이터 재사용·누락 종료 표식을 허용하지 않습니다. 이 정책은 원본 4개 모두를 통과하며 레퍼런스의 느슨한 EOF 처리보다 엄격합니다.

## API·메모리·오류

- `VgaSpecFile::parse(bytes)`: 전체 DAT 프레이밍·XOR·후미와 정확히 한 구간인지 검사합니다. `decoded_size()` 크기의 버퍼를 `decode_into()`에 전달하면 내부 레코드를 검증해 반환합니다. 길이 오류는 출력을 보존하며 이후 오류에서는 버퍼 내용을 버립니다.
- `VgaSpecRecord::parse(bytes)`: 이미 DAT 해제한 내부 레코드의 헤더·전체 RLE·네 종료 표식·후미를 검증합니다. `as_bytes()`·`vga_palette()`·`ega_palette()`·`unknown_palette_bytes()`로 원문·팔레트를 읽습니다.
- `decode_stripe_into(index, output)`: 스트립 0~3 하나를 정확히 14400바이트에 해제합니다. `decode_into(output)`은 정확히 `VGASPEC_PLANAR_SIZE=57600`바이트에 네 스트립을 해제하고 `VgaSpecImage`를 반환합니다.
- `VgaSpecImage::pixel(x,y)`: 배경 내부 좌표의 `IndexedPixel { color, opaque }`를 반환하며 범위 밖은 `None`입니다. 색상 0은 투명, 1~7은 RGB 값과 무관하게 불투명입니다. `decode_into(&mut pixels)`는 정확히 153600개의 행 우선 픽셀을 요구합니다. `planar_bytes()`는 해제된 플레인 버퍼를 빌려줍니다.

내부 레코드는 검증 시 픽셀·스크래치 배열을 할당하지 않고 패킷을 순회합니다. 같은 순회 함수를 출력 버퍼와 함께 재사용해 해제합니다. 검증된 불변 레코드의 스트립·플레인·픽셀 해제는 인덱스/출력 길이 오류에서 호출자 메모리를 보존합니다. 반환 이미지의 수명은 플레인 출력 버퍼에만 연결되어 내부 RLE 버퍼를 해제한 뒤에도 사용할 수 있습니다.

`VgaSpecError`는 DAT·구간 수·해제·헤더·팔레트·스트립 인덱스·패킷 절단·스트립 크기·출력 초과·후미·출력 길이를 구분합니다. 패킷 오류의 `offset`은 40바이트 헤더를 포함한 **내부 레코드 기준**이고 스트립은 0~3입니다. 원인 DAT/해제 오류의 압축 파일 기준 위치와 `source()`도 보존합니다. 큰 조회 좌표·인덱스는 곱셈 전에 검사합니다.

라이브러리는 계속 `no_std`·무할당이고 외부 패키지·I/O를 추가하지 않았습니다. 전체 픽셀 배열이 필요하지 않으면 `pixel()`로 조회할 수 있습니다. 파일명·특수 그래픽 번호 연결과 원본 인증은 호출자·입력 매니페스트 책임입니다.

## 원본·독립 비교와 남은 범위

Rust가 출력한 모든 내부 레코드·플레인·픽셀 바이트와 팔레트를 독립 Python과 대조했습니다. Python은 원본 DAT를 비트 문자열로 해제하고, 별도 RLE 반복자와 전체 플레인 이진 문자열을 사용했습니다. 4개 배경 **614400픽셀**과 **230400플레인 바이트**가 전부 같습니다. 내부 레코드는 기존 G02-02 해제 결과와도 같습니다.

고정 TS의 BinaryReader·비트 읽기/쓰기·DAT 해제·FileContainer·VgaSpecReader·PaletteImage도 실행했습니다. import·로깅만 대체하고, Frame/ColorPalette 어댑터로 인덱스 출력과 팔레트 설정 호출을 캡처했습니다. 플레인 읽기 호출 앞에서 입력을 복사해 RLE 결과도 비교했습니다. Node 24.11.0의 `--experimental-transform-types`를 사용하며 npm 패키지는 설치하지 않았습니다. 모든 내부/플레인/픽셀 바이트와 VGA 성분의 4배 확대 값이 일치했습니다.

TS의 스트립 배치 호출은 x=304, y=0/40/80/120이었습니다. 호출 좌표만 확인했으며 원본 Frame의 RGB·알파 렌더링을 실행한 것은 아닙니다. 이 라이브러리는 원점 (0,0)의 960×160 배경을 제공합니다. 전체 레벨 배치·팔레트 합성·지형 재구성은 G03-06, 충돌·DOS 실행 호환성은 후속 범위입니다.

## 원본 검사 재현

[원본 확보](../../assets/README.md) 후 저장소 루트에서 실행합니다. 호스트 예제는 압축 원본 4개를 직접 읽습니다. 내부 데이터·픽셀을 포함한 JSONL은 무시되는 로컬 폴더에 저장하며 명령 실패 시 부분 출력을 버립니다.

```sh
python3 tools/reference_assets.py verify
cargo run -p lemmings-resources --example inspect_vgaspec --locked --offline -- \
  assets/local/dos-original > assets/local/issue-14_vgaspec.jsonl
python3 - <<'PY'
import hashlib
import json
from pathlib import Path
contract = json.loads(Path('assets/manifests/vgaspec_backgrounds.json').read_text())
for entry in contract['inputs']:
    assert hashlib.sha256(Path(entry['path']).read_bytes()).hexdigest() == entry['sha256']
rows = [json.loads(line) for line in Path('assets/local/issue-14_vgaspec.jsonl').read_text().splitlines()]
assert len(rows) == len(contract['backgrounds']) == 4
for row, expected in zip(rows, contract['backgrounds']):
    assert row['set'] == expected['set'] and (row['width'], row['height']) == (960, 160)
    for key in ['vga', 'ega', 'unknown', 'transparent_pixels']:
        assert row[key] == expected[key]
    decoded = {}
    for key, size_key in [('inner', 'inner_size'), ('planar', 'planar_size'), ('pixel', 'pixel_count')]:
        data = bytes.fromhex(row[key + '_hex'])
        assert len(data) == expected[size_key]
        assert hashlib.sha256(data).hexdigest() == expected[key + '_sha256']
        decoded[key] = data
    inner = decoded['inner']
    assert [list(inner[i*3:i*3+3]) for i in range(8)] == row['vga']
    assert list(inner[24:32]) == row['ega'] and list(inner[32:40]) == row['unknown']
    # Independently reconstruct all RLE strips and indexed pixels from the inner record.
    source = iter(inner[40:])
    strips = []
    for _ in range(4):
        strip = bytearray()
        while True:
            code = next(source)
            if code == 128:
                break
            if code < 128:
                strip.extend(next(source) for _ in range(code + 1))
            else:
                strip.extend(bytes([next(source)]) * (257 - code))
        assert len(strip) == 14400
        strips.append(bytes(strip))
    assert next(source, None) is None and b''.join(strips) == decoded['planar']
    pixels = bytearray()
    for strip in strips:
        planes = [f'{int.from_bytes(strip[i*4800:(i+1)*4800], "big"):038400b}' for i in range(3)]
        for bits in zip(*planes):
            color = sum(int(v) * (2**i) for i, v in enumerate(bits))
            pixels.append(color or 128)
    assert pixels == decoded['pixel'] and pixels.count(128) == row['transparent_pixels']
print('PASS: 4 palettes, all inner/planar/pixel hashes, independent RLE and 614400 pixels')
PY
```

`set`는 파일 번호 0~3입니다. `inner_hex`는 DAT 해제 레코드, `planar_hex`는 스트립 순 플레인, `pixel_hex`는 행 우선 픽셀입니다. 출력 픽셀은 인덱스 1~7을 그대로, 투명한 0은 0x80으로 인코딩합니다. 이 인코딩은 비교용이며 라이브러리는 불투명 여부를 bool로 제공합니다. 새 검증 매니페스트는 확보용 `fetch --manifest` 입력이 아닙니다.

결과: [보고서](../reports/issue-14_vgaspec_backgrounds.md), [진행 기록](../working/issue-14_vgaspec_backgrounds.md), [작업일지](../orders/20261018.md#issue-14).
