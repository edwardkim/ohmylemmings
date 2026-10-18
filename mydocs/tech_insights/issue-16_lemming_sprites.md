# G03-07 MAIN.DAT 레밍 스프라이트

작업 이슈: [GitHub #16](https://github.com/edwardkim/ohmylemmings/issues/16). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 형식과 근거

원작 VGA MAIN.DAT는 DAT 구간 7개입니다. 구간 0의 21104바이트에 애니메이션 28개·프레임 337개·63488픽셀이 연속 저장됩니다. 각 프레임은 행 우선 비트플레인 2~4개이며 MSB가 왼쪽, 플레인 0이 색상 비트 0입니다. 프레임마다 모든 플레인이 이어진 뒤 다음 프레임으로 넘어갑니다. 별도 마스크 없이 색상 0이 투명합니다. 폭은 16, 마지막 폭발 이미지만 32×32입니다.

| 고정 소스 | 확인한 내용 |
|---|---|
| [Lemmix Styles.Base.pas](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Styles.Base.pas#L401) | `InitMetadata`의 28개 주소·크기·프레임·방향·발 기준점, `Load`의 구간 0 연결 |
| [Lemmix Dos.MainDat.pas](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Dos.MainDat.pas#L77), [Dos.Bitmaps.pas](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Dos.Bitmaps.pas) | DAT 해제·비트플레인·연속 프레임 추출 |
| [TS GameResources](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/game-resources.ts#L55) | MAIN 구간 0을 LemmingsSprite에 전달 |
| [TS LemmingsSprite](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/lemmings-sprite.ts), [Animation](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/animation.ts) | 같은 디스크 순서·크기·프레임 수, 색상 0 투명·프레임 생성 |

원본 0번 구간의 시작 주소는 Walking 0x0000, Digging 0x02D0, Mining 0x2950/0x30A0, Vaporizing 0x4350, Explosion 0x50F0입니다. 마지막 폭발 플레인은 0x5270에서 끝나며 중간 빈 공간은 없습니다. 전체 목록은 [검증 매니페스트](../../assets/manifests/lemming_sprites.json)에 보존합니다.

MAIN 스프라이트에는 팔레트가 없습니다. 결과는 0~15 인덱스를 보존하며 외부에서 16색 팔레트를 선택합니다. 특히 증발 이미지는 상위 색상 인덱스도 사용합니다. Lemmix의 최종 팔레트 조립·벽돌색 인덱스 변경, TS의 GROUND 팔레트 선택은 렌더러의 정책으로 남깁니다. RGB가 검정인 비영 인덱스도 투명해지지 않습니다.

## 방향과 기준점 차이

[호환성 기준](issue-5_compatibility_baseline.md)에 따라 Lemmix의 방향 연결과 발 기준점을 사용합니다. 기준점의 음수가 이미지 좌상단 오프셋입니다. 실제 이미지 데이터는 두 소스에서 동일하며 방향별 이미지를 자동 반전하지 않습니다.

| 대상 | Lemmix/API | 고정 TS |
|---|---|---|
| Mining 오른쪽/왼쪽, 슬롯 14/15 | 원점 (-8,-13) | (-8,-12) |
| Vaporizing, 슬롯 22 | 원점 (-8,-14), 양방향 공유 | (-8,-10), 오른쪽에만 등록 |
| Explosion, 슬롯 27 | 원점 (-16,-25) | (-8,-10) |
| Shrugging, 슬롯 24/25 | 오른쪽 24·왼쪽 25 | 두 번째 등록의 dir=0이 오른쪽도 25로 덮어씀 |

18종 이미지 중 방향별 10쌍과 공유 8개가 총 28개입니다. 공유는 Digging·Drowning·Splatting·Exiting·Vaporizing·Blocking·OhNo·Explosion입니다. 반복·한 번 재생, 틱별 프레임 선택, 물리상의 레밍 좌표와 상태 전환은 이번 API에서 구현하지 않습니다.

## API와 오류 계약

[sprites.rs](../../crates/lemmings-resources/src/sprites.rs)는 외부 의존성·할당·I/O 없는 `no_std` API입니다.

- `MainDatFile::parse`: 전체 입력의 구간 수 7·모든 헤더/압축 본문 체크섬·구간 0 선언 길이 21104를 확인합니다. 후미 구간 손상도 오류입니다. `as_bytes()`와 `sections()`로 원문·다른 구간을 빌립니다. 구간 1~6의 압축 명령이나 UI 형식까지 인증하지는 않습니다.
- `decode_sprites_into`: 정확히 21104바이트 호출자 버퍼에 구간 0을 해제하고 출력만 빌리는 `LemmingSprites`를 반환합니다. 길이 오류는 출력을 보존합니다. 압축 명령 오류는 부분 출력 가능성이 있으므로 출력을 버립니다.
- `LemmingSprites::parse`: 이미 해제한 레코드의 정확한 길이를 검사합니다. `animation(slot)`으로 디스크 순서, `for_sprite(kind, direction)`으로 의미·방향을 선택합니다.
- `LemmingFrames::info`: 슬롯·종류·선택 방향 또는 공유·주소·크기·플레인·프레임 수·발 좌표를 제공합니다. `frame(index)`는 범위를 검사하며 순환·초기 프레임 회전·자동 반전을 하지 않습니다.
- `SpriteFrame::pixel`과 `decode_into`: 기존 `IndexedPixel`을 반환합니다. 큰 좌표는 곱셈 전에 검사합니다. 정확한 픽셀 출력 길이가 아니면 아무것도 쓰지 않습니다. 투명 픽셀은 색상 0·점유 false입니다.

파일·구간·프레임·출력 오류에 위치와 기대/실제 값을 담으며 하위 DAT·압축 오류는 `Error::source`로 연결합니다. 정확한 길이의 임의 바이트도 플레인 데이터로는 유효하므로 파일 출처는 별도 원본 해시로 식별합니다.

## 독립 검증

Python은 원본 MAIN.DAT의 역방향 압축 비트를 직접 읽어 7개 구간을 해제하고 G02의 전체 해시·바이트와 대조했습니다. 고정 Lemmix 메타데이터를 읽고 프레임별 전체 플레인을 이진 문자열로 풀어 Rust의 28개 메타데이터·337프레임·63488개 색상/투명 픽셀과 비교했습니다. 불투명 픽셀은 11142개입니다.

실제 고정 TS의 FileContainer·압축 해제·LemmingsSprite·Animation·PaletteImage·Frame·ColorPalette도 실행했습니다. import/로그 연결만 조정하고 원본 생성자·디코딩·그리기 알고리즘은 유지했습니다. `Animation.loadFromFile` 호출을 감싸 28개 생성 결과를 기록해, TS 조회 테이블에서 사라지는 오른쪽 Shrugging도 비교했습니다. 색마다 다른 16색 진단 팔레트로 RGB·알파·점유 마스크까지 전수 대조했습니다. 전체 픽셀은 일치하며 위의 기준점 4개·조회 2개 차이는 별도 기록했습니다. `getFrame`의 재생 동작은 검증 범위가 아닙니다.

[스프라이트 검증 매니페스트](../../assets/manifests/lemming_sprites.json)에 구간 7개의 해시, 애니메이션·개별 프레임의 픽셀 해시, 메타데이터, 추가 고정 소스 9개와 차이 목록을 기록했습니다. 기존 원본·소스·비교 매니페스트는 변경하지 않습니다. 새 파일은 확보용 `fetch --manifest` 입력이 아닙니다.

## 원본 검사 재현

원본을 [확보](../../assets/README.md)한 뒤 저장소 루트에서 실행합니다. `inspect_sprites MAIN_DAT_FILE OUTPUT_DIRECTORY`는 새 디렉터리에 해제 구간 0과 애니메이션별 맵 28개를 저장하고 JSONL 메타데이터를 표준 출력으로 보냅니다. 맵은 디스크 프레임 순서·프레임 내 행 우선이며 불투명 색상 1~15, 투명 0x80의 한 바이트 인코딩입니다. 기존 출력은 덮어쓰지 않습니다.

아래 명령은 임시 디렉터리로 반복 실행할 수 있습니다. 현재 Rust 결과를 고정 기대값과 비교하며 독립 Python/TS 알고리즘을 다시 실행하는 명령은 아닙니다.

```sh
python3 tools/reference_assets.py verify
python3 - <<'PY'
from pathlib import Path
import hashlib, json, subprocess, tempfile
sha = lambda data: hashlib.sha256(data).hexdigest()
manifest = json.loads(Path('assets/manifests/lemming_sprites.json').read_text())
for item in manifest['inputs']:
    assert sha(Path(item['path']).read_bytes()) == item['sha256'], item['path']
def run(example, destination):
    return subprocess.run([
        'cargo', 'run', '--quiet', '--locked', '--offline', '-p', 'lemmings-resources',
        '--example', example, '--', 'assets/local/dos-original/MAIN.DAT', str(destination)
    ], check=True, capture_output=True, text=True).stdout
with tempfile.TemporaryDirectory(prefix='issue-16-reproduce-', dir='assets/local') as folder:
    root = Path(folder)
    rows = [json.loads(s) for s in run('inspect_sprites', root/'sprites').splitlines()]
    run('unpack_dat', root/'sections')
    assert len(rows) == 28 and len(list((root/'sprites').iterdir())) == 29
    for target in manifest['sections']:
        data = (root/'sections'/f"part-{target['part']:03}.bin").read_bytes()
        assert len(data) == target['size'] and sha(data) == target['sha256']
    inner = (root/'sprites'/'section-000.bin').read_bytes()
    assert inner == (root/'sections'/'part-000.bin').read_bytes()
    for row, target in zip(rows, manifest['animations']):
        assert row == {k: target[k] for k in row}
        pixels = (root/'sprites'/row['pixel_file']).read_bytes()
        size = row['width'] * row['height']
        assert len(pixels) == size * row['frame_count']
        assert set(pixels) <= {*range(1,16), 128}
        assert len(pixels) - pixels.count(128) == row['opaque_pixels']
        assert sha(pixels) == target['pixel_sha256']
        assert [sha(pixels[f*size:(f+1)*size]) for f in range(row['frame_count'])] == target['frame_sha256']
    assert sum(r['frame_count'] for r in rows) == 337
    assert sum(r['frame_count'] * r['width'] * r['height'] for r in rows) == 63488
print('PASS: seven sections, 28 animations, 337 frames/63488 pixels, metadata and input hashes')
PY
```

## 테스트와 미리보기

합성 테스트 14개는 모든 짧은 접두부·추가 바이트, 전체 레이아웃·방향 연결, 2/3/4비트플레인·마지막 바이트·32폭 바이트 경계, 프레임 순서, 투명도·상위 색상, 잘못된 슬롯/프레임/큰 좌표, 정확한 출력 길이·경계 보호 값, 전체 7개 구간·후미 손상, 압축 오류 source, 호출자 출력만 빌리는 수명을 검증합니다.

CLI 오류 10개는 인자 누락/초과·파일 누락·짧은 헤더·마지막 구간 체크섬·구간 수 6/8·스프라이트 선언 길이·압축 명령·덮어쓰기 거부를 포함합니다. 원본과 기존 출력 해시를 보존했습니다.

로컬 `assets/local/issue-16-sprites/preview.png`는 각 애니메이션의 처음 최대 4프레임을 두 배 확대해 표시합니다. GROUND0의 Standard+Preview 진단 팔레트를 사용하며 최종 게임 팔레트 조립 검증은 아닙니다. PNG·바이너리·JSONL·원본·참조 소스 사본은 Git 추적에서 제외합니다.

- [결과 보고서](../reports/issue-16_lemming_sprites.md), [진행 기록](../working/issue-16_lemming_sprites.md), [작업일지](../orders/20261018.md#issue-16)
