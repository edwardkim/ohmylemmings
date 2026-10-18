# G03-10 레벨 객체 이미지 배치·합성

작업 이슈: [GitHub #19](https://github.com/edwardkim/ohmylemmings/issues/19). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 기준과 레퍼런스 차이

[호환성 계약](issue-5_compatibility_baseline.md)의 1584×160 월드와 [기존 지형 합성](issue-15_level_terrain.md)을 사용합니다. 정적 미리보기·GROUND의 시작 프레임·명시한 디스크 프레임을 합성하며 게임 시작 상태나 애니메이션 틱을 재현하는 API는 아닙니다.

| 고정 근거 | 적용 규칙 |
|---|---|
| [Lemmix Game.Rendering.pas](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.Rendering.pas#L413) | 원래 지형 비트와 객체 비트 보존. only-on-terrain 우선 분기, 두 패스와 패스 안 파일 순서, 프레임별 수직 반전 |
| [Lemmix Styles.Base.pas](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Styles.Base.pas#L619) | 고정 DOS 8색 + 현재 Custom 8색, 7번은 8번 복사. 특수 레벨의 8번은 RGB (124,124,0) |
| [Dos.Structures.pas](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Dos.Structures.pas#L395), [Dos.Bitmaps.pas](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Dos.Bitmaps.pas#L73) | Custom 확대는 floor(raw6×255/63), 고정 색상과 VGASPEC 실제 표시 확대는 raw6<<2. 같은 저장소 안에서도 경로가 다름 |
| [Lemmix Meta.Structures.pas](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Meta.Structures.pas#L192) | 시작 프레임과 미리보기 주소의 관계. Lemmix는 주소 차이를 stride로 정수 나눗셈 |
| [TS ObjectManager](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/game-play/object-manager.ts), [DisplayImage](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/view/display-image.ts#L311) | 파일 순서 단일 패스, no-overwrite와 only-overwrite 모두 원래 지형만 조회. 두 플래그 결합 시 어느 픽셀도 그리지 않음 |
| [TS GroundReader](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/lemmings/ground-reader.ts#L169) | 객체의 Standard+Preview 팔레트·raw6<<2는 채택한 게임 팔레트와 다름 |

only-on-terrain 객체를 먼저, 나머지를 나중에 그리며 각 패스는 원래 슬롯 순서입니다. 전자는 원래 지형의 점유를 검사합니다. 후자의 no-overwrite는 지형 또는 앞 객체가 차지한 픽셀을 건너뜁니다. 두 플래그가 결합되면 only-on-terrain을 적용합니다. 수직 반전 후 네 경계에서 자르고, 소스 마스크가 0인 픽셀은 배경을 보존합니다. 레벨 객체 Y는 부호 없는 값이므로 위쪽 음수 배치는 표현되지 않습니다.

일반 지형의 로컬 인덱스에 8을 더해 전역 색상 번호로 보존합니다. 객체가 지형을 덮어도 지형 비트는 유지됩니다. **이 비트는 그리기 출처이며 강철·충돌 마스크가 아닙니다.** 특수 레벨은 같은 상위 색상 번호라도 객체와 배경의 RGB 확대 방식이 달라 두 팔레트를 분리합니다. 불투명 검정도 유지합니다. Lemmix의 색상 0 투명 처리와 별도 마스크 정책은 합성 fixture에서 다르지만 원본 540개 객체 프레임에서는 선택 결과가 모두 같았습니다.

## API와 오류 계약

[objects.rs](../../crates/lemmings-resources/src/objects.rs)는 `no_std`·무할당·I/O 없는 API입니다.

- `ObjectGraphics::new(index, images)`: 검증된 GROUND/VGAGR에 0~4 선택자 태그를 부여합니다. 특수 배경에도 해당 GROUND 객체를 사용합니다.
- `ObjectFrameSelection::{Preview, Initial, Explicit(&[usize;32])}`: 정확한 미리보기 주소, GROUND 시작 프레임, 원래 32개 레벨 슬롯별 인덱스입니다. 빈 슬롯은 무시하고 활성 슬롯에는 자동 순환을 적용하지 않습니다. 미리보기는 비정렬 주소도 그대로 읽습니다.
- `compose_objects(level, graphics, terrain, selection, output)`: 정확히 253440개 `ScenePixel`을 받습니다. 길이·그래픽 선택자·화면 밖을 포함한 모든 활성 ID/프레임을 먼저 검사하므로 **모든 오류에서 출력이 보존**됩니다. 슬롯별 검증된 이미지 32개는 스택에 저장합니다.
- `ScenePixel { color, terrain, object }`: 전역 인덱스와 독립 출처 비트입니다. 투명 출력은 0/false/false, `opaque()`는 두 비트의 OR입니다.
- `LevelScene`: 출력만 빌립니다. `pixels()`, 객체용 RGB8 `palette()`, 로컬 RGB8 `terrain_palette()`, 경계를 검사하는 `pixel(x,y)`·`rgba(x,y)`를 제공합니다. 투명 RGBA는 모두 0입니다.

원본·GROUND/VGAGR 쌍과 같은 레벨의 `LevelTerrain`을 결합하는 책임은 호출자에게 있습니다. 태그는 파일 진위나 지형 배치의 동일성을 인증하지 않습니다. 원시 RGB6·미해석 필드·이전 API는 변경하지 않습니다. `ObjectComposeError`는 잘못된 태그·선택자·누락 ID·이미지/프레임·출력 길이를 구분하며 이미지 오류에 원래 배치 슬롯과 `source()`를 보존합니다.

## 원본 전체 대조

원본 50개·기존 소스 46개와 추가 고정 소스 13개의 크기·git blob/SHA-256을 확인했습니다. 독립 Python이 LEVEL·VGAGR·VGASPEC의 **94개 DAT 구간을 원본에서 다시 해제**해 기존 모든 바이트를 유지했습니다. 비트 문자열·별도 RLE/좌표/클리핑/두 패스 구현으로 지형과 객체를 재구성했습니다.

80개 기본 배치의 객체는 454개이며 결합 플래그가 5개, 반전 배치는 0개입니다. 반전은 합성 fixture에서 검증합니다. 원본 54개 객체 리소스의 미리보기 주소는 모두 시작 프레임과 같았습니다. 미리보기·마지막 디스크 프레임의 **160개 기본 결과 / 240개 표시 결과 / 60825600개 색상·출처 픽셀**과 두 팔레트가 Rust와 일치합니다. `initial` 모드의 120개 출력도 별도로 실행해 미리보기와 바이트 단위로 대조했습니다.

실제 고정 TS의 DAT·GroundReader·LevelReader·MapObject·Animation·ObjectManager를 실행해 1332개 그리기 호출을 얻었습니다. 실제 GroundRenderer의 이미지 복사와 Frame.setPixel로 객체를 그려 전체 점유·RGB를 대조했습니다. 입력의 팔레트·두 패스·결합 플래그 우선순위와 기존 지형을 채택한 규칙으로 맞췄으며 알고리즘 본문은 변경하지 않았습니다. import/로그·보조 자료형·GUI 버퍼 생성만 어댑터로 대체했습니다. 마지막 프레임은 틱 0에 해당 프레임을 선택하도록 입력을 지정했습니다. Node 24.11.0·`--experimental-transform-types`를 사용했고 npm 의존성은 없습니다.

동일한 채택 팔레트·배경을 제공하고 실제 TS ObjectManager/DisplayImage 그리기 규칙을 그대로 실행하면 아래 차이가 남습니다. TS의 원래 팔레트도 9개 일반/특수 조합에서 다릅니다. 무수정 TS나 실제 DOS 게임 실행과 일치한다고 판정하지 않습니다.

| 표시 진입점 | 프레임 선택 | 그리기 규칙 차이 픽셀 |
|---|---|---:|
| Taxing 13 | 미리보기 | 563 |
| Taxing 13 | 마지막 | 546 |

[검증 매니페스트](../../assets/manifests/level_objects.json)는 입력 6개 해시, 추가 소스 13개, 54개 프레임 메타데이터, 160개 기본 결과의 인덱스/출처·RGBA 해시·팔레트·점유 수와 TS 차이를 기록합니다. 확보 도구의 `fetch --manifest` 입력이 아닙니다.

## 원본 검사 재현

원본을 [확보](../../assets/README.md)한 뒤 루트에서 실행합니다. [inspect_objects](../../crates/lemmings-resources/examples/inspect_objects.rs)는 `ORIGINAL_ASSET_DIRECTORY OUTPUT_DIRECTORY preview|initial|last`를 받으며 기존 파일을 덮어쓰지 않습니다. JSONL은 카탈로그 연결·선택자·모드·두 RGB8 팔레트·지형/객체/전체 점유 수를 담습니다. 출력 한 파일은 행 우선 253440바이트이며 하위 4비트는 색상, bit 4는 지형, bit 5는 객체입니다. 투명 값은 0입니다.

아래 명령은 새 임시 폴더에서 세 모드를 실행해 고정 기대값과 대조합니다. 독립 Python/TS 알고리즘 자체를 다시 실행하는 명령은 아닙니다.

```sh
python3 tools/reference_assets.py verify
python3 - <<'PY'
import hashlib, json, subprocess, tempfile
from pathlib import Path
sha = lambda data: hashlib.sha256(data).hexdigest()
m = json.loads(Path('assets/manifests/level_objects.json').read_text())
for item in m['inputs']:
    assert sha(Path(item['path']).read_bytes()) == item['sha256'], item['path']
expected = {(r['file'], r['part'], r['mode']): r for r in m['maps']}
catalog = json.loads(Path('assets/manifests/dos_compatibility.json').read_text())['levels']
assert len(expected) == 160 and len(catalog) == 120
with tempfile.TemporaryDirectory(prefix='issue-19-reproduce-', dir='assets/local') as temporary:
    for mode in ('preview', 'initial', 'last'):
        output = Path(temporary) / mode
        result = subprocess.run([
            'cargo', 'run', '--quiet', '--locked', '--offline', '-p', 'lemmings-resources',
            '--example', 'inspect_objects', '--', 'assets/local/dos-original', str(output), mode
        ], check=True, capture_output=True, text=True)
        rows = [json.loads(line) for line in result.stdout.splitlines()]
        assert len(rows) == len(list(output.iterdir())) == 120
        for index, (row, entry) in enumerate(zip(rows, catalog)):
            key = (f"LEVEL{row['file']:03}.DAT", row['part'])
            assert key == (entry['file'], entry['part'])
            assert row['global_index'] == index and row['mode'] == mode
            for field in ('group', 'number', 'graphics', 'special'):
                assert row[field] == entry[field], (mode, index, field)
            target = expected[*key, 'preview' if mode == 'initial' else mode]
            for field in ('graphics', 'special', 'palette_hex', 'terrain_palette_hex',
                          'terrain_pixels', 'object_pixels', 'opaque_pixels'):
                assert row[field] == target[field], (mode, index, field)
            pixels = (output / row['pixel_file']).read_bytes()
            assert len(pixels) == 1584 * 160 and sha(pixels) == target['pixel_sha256']
            assert all(v == 0 or 16 <= v < 64 for v in pixels)
            assert sum(bool(v & 16) for v in pixels) == target['terrain_pixels']
            assert sum(bool(v & 32) for v in pixels) == target['object_pixels']
            assert sum(bool(v & 48) for v in pixels) == target['opaque_pixels']
            palette = bytes.fromhex(row['palette_hex'])
            terrain = bytes.fromhex(row['terrain_palette_hex'])
            lookup = {0: bytes(4)}
            for value in set(pixels) - {0}:
                color = value & 15
                rgb = (palette[color*3:color*3+3] if value & 32 else
                       terrain[(color-8)*3:(color-8)*3+3])
                lookup[value] = rgb + b'\xff'
            rgba = b''.join(lookup[value] for value in pixels)
            assert sha(rgba) == target['rgba_sha256'], (mode, index)
print('PASS: 360 scenes, indexed/provenance/RGBA hashes, palettes, routes and six input hashes')
PY
```

## 검증 범위와 다음 책임

합성 TDD는 새 16개 Red·기존 165개 통과에서 Green 181개를 확인했습니다. 모든 플래그 조합·파일 순서/두 패스·빈 슬롯·세 프레임 선택·비정렬 미리보기·색상/마스크·클리핑·극단 좌표·원본 슬롯별 프레임·선택자/출력 오류 보존·특수 배경·출력만 빌리는 수명을 검증합니다. `make check`는 Rust 단위 183개·문서 예제 14개·Python 8개와 포맷·Clippy·빌드·WASM을 통과했습니다.

CLI 오류 10개는 인자·모드·파일·GROUND/LEVEL 절단·VGAGR/VGASPEC 체크섬·ODD 값·덮어쓰기 거부를 검사했습니다. 원본·앞 단계 결과를 보존했습니다. 미리보기 `assets/local/issue-19-scenes/preview.png`는 Fun 1·Fun 22·Tricky 14·Taxing 13·Mayhem 22 순서이며 픽셀 데이터에서 생성·확인했습니다. 원본·생성 바이너리·PNG·JSONL·소스 사본은 추적하지 않습니다.

G03의 확인된 리소스 형식 fixture와 기준 레벨 지형·객체·이미지 재구성 증거를 갖췄습니다. 전체 G03 완료는 게시·실제 CI 확인 후 상위 #1에 반영합니다. 미해석 UI 범위·MAIN 구간 5는 원문을 유지하며 의미를 추정하지 않습니다. 사운드는 G10, 지형/강철/충돌·행동 틱은 G04~G07, 플랫폼 화면과 실제 실행 비교는 G06 이후의 책임입니다. 다음 구현은 별도 이슈의 G04-01 픽셀 지형 읽기·경계 검증입니다.

- [결과 보고서](../reports/issue-19_level_objects.md), [진행 기록](../working/issue-19_level_objects.md), [작업일지](../orders/20261018.md#issue-19)
