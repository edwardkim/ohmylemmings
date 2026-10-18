# G03-06 초기 레벨 지형 합성

작업 이슈: [GitHub #15](https://github.com/edwardkim/ohmylemmings/issues/15). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 기준과 레퍼런스 차이

[호환성 계약](issue-5_compatibility_baseline.md)의 Lemmix 기준 1584×160 월드 지형을 재구성합니다. 뷰포트 시작점·객체·강철·동적 지형·MAIN.DAT 스프라이트는 합성하지 않습니다. 인덱스·점유 여부와 원시 6비트 팔레트를 반환하며 실행 호환성 인증은 후속 통합 범위입니다.

| 근거 | 적용 규칙 |
|---|---|
| [Lemmix Dos.Consts.pas](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Dos.Consts.pas#L58) | 월드 폭 1584. TS의 기본 폭 1600과 구분 |
| [Lemmix Game.Rendering.pas](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.Rendering.pas#L387) | 파일 슬롯 순서, 소스 마스크, 수직 반전, no-overwrite 우선 분기, 특수 배경 (304,0) |
| [Lemmix Level.Loader.pas](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Level.Loader.pas) | 기존 레벨 파서의 부호 좌표를 사용. Y 원값 256도 음수 범위 |
| [TS GroundRenderer](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/ground-renderer.ts) | erase 우선으로 Lemmix와 차이. 특수 배경은 일반 지형을 대체 |
| [TS Frame](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/frame.ts), [ColorPalette](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/lemmings/color-palette.ts) | 실제 점유 마스크·RGB 변환·그리기를 독립 비교에 사용 |

일반 배치는 클리핑 전에 수직 반전하며, 마스크가 없는 소스 픽셀은 건너뜁니다. no-overwrite는 빈 목적지에만 그립니다. 이 플래그가 없을 때 erase는 목적지를 비우며, 둘 다 없으면 덮어씁니다. 두 플래그를 동시에 켜도 no-overwrite가 우선합니다. 불투명 색상 0과 검정 RGB도 점유 픽셀입니다.

특수 선택자 1~4는 VGASPEC0~3에 연결합니다. 해당 960×160 이미지를 x=304에 놓고 일반 지형 배치를 전부 건너뜁니다. 일반 지형은 GROUND의 Custom 팔레트, 특수 지형은 VGASPEC 팔레트를 사용합니다. TS의 Y 원값 256 처리 차이는 [기존 기록](issue-10_level_records.md)에 따르며 원본 80개에는 해당 값이 없습니다.

## API와 오류 계약

[terrain.rs](../../crates/lemmings-resources/src/terrain.rs)는 `no_std`·무할당·I/O 없는 합성을 제공합니다.

- `TerrainGraphics::new(index, images)`: 검증된 GROUND/VGAGR 뷰에 0~4 태그를 부여합니다.
- `SpecialTerrain::decode_into(index, record, planes)`: 0~3 태그·정확한 57600바이트 출력을 검사합니다. 동일 레코드의 이미지와 팔레트를 묶으며 원본 레코드 수명과 분리됩니다.
- `compose_terrain(level, source, output)`: 정확히 253440개 `IndexedPixel`을 받습니다. 길이·일반/특수 선택자·전체 활성 지형 ID를 먼저 검증하므로 모든 오류에서 출력이 보존됩니다. 화면 밖 마지막 슬롯의 누락 ID도 거부합니다.
- `LevelTerrain`: 출력 버퍼만 빌리고 팔레트를 복사합니다. `pixels()`, `palette()`, 경계 검사하는 `pixel(x,y)`를 제공합니다. 투명 출력은 항상 색상 0·점유 false입니다.

태그는 호출자가 지정하는 선택자이며 파일 내용의 진위나 GROUND/VGAGR 쌍을 인증하지 않습니다. `ResolvedLevel::base_record()`로 ODD 적용 전 기본 배치를 전달할 수 있습니다. ODD는 배치·그래픽 선택자를 바꾸지 않습니다. 오류에는 원래 슬롯·필요/실제 선택자를 담고 하위 이미지·특수 배경 오류의 source를 연결합니다.

## 원본 전체 대조

80개 기본 배치(일반 76·특수 4)를 독립 Python으로 재구성하고 120개 표시 진입점(일반 116·특수 4)에 연결했습니다. 모든 30412800개 인덱스·점유 픽셀과 팔레트가 Rust와 일치합니다. Python은 고정 해시의 기본 레코드·해제 VGAGR/VGASPEC를 읽고 비트 문자열·독립 RLE·부호 좌표·플래그·클리핑으로 계산했습니다.

실제 고정 TS의 DAT 해제·GroundReader·LevelReader·VgaSpecReader·GroundRenderer·Frame·ColorPalette도 실행했습니다. import/로그 제거와 자료형 생성자 어댑터만 사용했으며 렌더링 알고리즘은 수정하지 않았습니다. TS 1600폭 결과를 **행마다 1584폭으로 자르고**, 결합 플래그 입력에서만 erase를 끄면 Lemmix 분기와 같아집니다. Frame의 점유 마스크가 없는 픽셀은 RGBA 모두 0으로 정규화했습니다. 이 조건에서 120개 전체 점유·RGB·팔레트가 일치합니다. RGB 해시 비교는 각 채널 `raw6 << 2`, 점유 알파 255를 사용하며 라이브러리가 확대 방식을 강제하지는 않습니다.

원본 기본 배치 117개, 표시 카탈로그 131개 배치에 erase+no-overwrite가 함께 있습니다. TS 원래 우선순위로는 아래 12개 진입점에서 차이가 납니다. 따라서 무수정 TS와 일치한다고 판정하지 않습니다.

| 진입점 | RGBA 차이 픽셀 | 점유 차이 픽셀 |
|---|---:|---:|
| Fun 13 / Tricky 2 | 각각 184 | 각각 184 |
| Fun 15 / Taxing 1 | 각각 1190 | 각각 1190 |
| Fun 20 / Mayhem 7 | 각각 95 | 각각 95 |
| Tricky 9 / Mayhem 6 | 각각 813 | 각각 601 |
| Tricky 21 | 16411 | 16411 |
| Taxing 12 | 117 | 117 |
| Mayhem 12 | 6665 | 6529 |
| Mayhem 18 | 209 | 209 |
| 합계 | 27966 | 27406 |

[지형 검증 매니페스트](../../assets/manifests/level_terrain.json)에 80개 기본 맵의 인덱스·정규화 RGBA 해시·팔레트·점유 수, 입력 해시, 추가 고정 소스 식별값과 차이 목록을 기록합니다. 확보 도구의 `fetch --manifest` 입력은 아닙니다. 기존 입력 매니페스트는 변경하지 않습니다.

## 원본 검사 재현

저장소 루트에서 원본을 [확보](../../assets/README.md)한 뒤 실행합니다. 예제는 새 출력 디렉터리에 120개 맵을 기록하며 기존 파일을 덮어쓰지 않습니다. JSONL은 표시 ID·원본 연결·선택자·팔레트·점유 수를 담습니다. 맵 한 파일은 행 우선 253440바이트이며 불투명 인덱스 0~7, 투명 0x80으로 저장됩니다.

아래 명령은 임시 출력 디렉터리를 사용해 반복 실행할 수 있습니다. 현재 구현의 결과를 고정 기대값과 대조하는 명령이며 독립 Python/TS 구현을 다시 실행하는 명령은 아닙니다.

```sh
python3 tools/reference_assets.py verify
python3 - <<'PY'
import hashlib, json, subprocess, tempfile
from pathlib import Path
sha = lambda data: hashlib.sha256(data).hexdigest()
manifest = json.loads(Path('assets/manifests/level_terrain.json').read_text())
for item in manifest['inputs']:
    assert sha(Path(item['path']).read_bytes()) == item['sha256'], item['path']
expected = {(m['file'], m['part']): m for m in manifest['maps']}
catalog = json.loads(Path('assets/manifests/dos_compatibility.json').read_text())['levels']
assert len(expected) == 80 and len(catalog) == 120
with tempfile.TemporaryDirectory(prefix='issue-15-reproduce-', dir='assets/local') as output:
    result = subprocess.run([
        'cargo', 'run', '--quiet', '--locked', '--offline', '-p', 'lemmings-resources',
        '--example', 'inspect_terrain', '--', 'assets/local/dos-original', output
    ], check=True, capture_output=True, text=True)
    rows = [json.loads(line) for line in result.stdout.splitlines()]
    assert len(rows) == 120 and len(list(Path(output).iterdir())) == 120
    for index, (row, entry) in enumerate(zip(rows, catalog)):
        key = (f"LEVEL{row['file']:03}.DAT", row['part'])
        assert key == (entry['file'], entry['part'])
        assert row['global_index'] == index
        for field in ('group', 'number', 'graphics', 'special'):
            assert row[field] == entry[field], (index, field)
        target = expected[key]
        for field in ('graphics', 'special', 'palette_hex', 'opaque_pixels'):
            assert row[field] == target[field], (index, field)
        pixels = (Path(output) / row['pixel_file']).read_bytes()
        assert len(pixels) == 1584 * 160 and set(pixels) <= {*range(8), 128}
        assert len(pixels) - pixels.count(128) == target['opaque_pixels']
        assert sha(pixels) == target['pixel_sha256'], index
        palette = bytes.fromhex(row['palette_hex'])
        lookup = {128: bytes(4)}
        lookup.update({i: bytes(c << 2 for c in palette[i*3:i*3+3]) + b'\xff' for i in range(8)})
        rgba = b''.join(lookup[pixel] for pixel in pixels)
        assert sha(rgba) == target['rgba_shifted_sha256'], index
print('PASS: 120 maps, 30412800 indexed/RGBA pixels, palettes, routes and input hashes')
PY
```

## 검증 기록

합성 테스트 14개는 전체 8개 플래그 조합·순서·반전·마스크 구멍·불투명 검정·네 경계 클리핑·화면 밖 배치·마지막 누락 ID·선택자/출력 길이 오류의 출력 보존·네 특수 배경·팔레트와 출력만 빌리는 수명을 확인합니다.

호스트 예제는 인자 누락/초과·파일 누락·GROUND 길이·VGASPEC 체크섬·ODD 수치·LEVEL 절단·출력 덮어쓰기 거부의 8개 경우에서 실패와 경로 문맥을 확인했습니다. 기존 맵과 미리보기의 해시는 유지됩니다.

로컬 미리보기 `assets/local/issue-15-terrain/preview.png`는 Fun 1·Fun 22·Tricky 14·Taxing 15·Mayhem 22를 위에서부터 나열하고 투명 영역을 검정 바탕으로 표시합니다. 이미지·바이너리 출력·참조 소스 사본은 Git 추적에서 제외합니다.

- [결과 보고서](../reports/issue-15_level_terrain.md), [진행 기록](../working/issue-15_level_terrain.md), [작업일지](../orders/20261018.md#issue-15)
