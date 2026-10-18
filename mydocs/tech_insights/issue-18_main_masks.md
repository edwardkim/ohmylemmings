# G03-09 MAIN.DAT 마스크·카운트다운

작업 이슈: [GitHub #18](https://github.com/edwardkim/ohmylemmings/issues/18). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 형식과 지원 목록

원작 VGA MAIN.DAT 구간 1은 388바이트입니다. 1비트 이미지 23프레임·3104비트를 행 우선·MSB 왼쪽으로 저장합니다. 비트 1은 선택된 마스크/글리프 전경이며 RGB·알파·충돌 판정을 뜻하지 않습니다. 전체 바이트를 해석하며 프레임 사이 패딩이나 미해석 후미는 없습니다.

| 묶음 | 시작 주소 | 크기 | 프레임 | 프레임 바이트 |
|---|---:|---|---:|---:|
| BashingRight | 0x000 | 16×10 | 4 | 20 |
| BashingLeft | 0x050 | 16×10 | 4 | 20 |
| MiningRight | 0x0A0 | 16×13 | 2 | 26 |
| MiningLeft | 0x0D4 | 16×13 | 2 | 26 |
| Explosion | 0x108 | 16×22 | 1 | 44 |
| Countdown | 0x134 | 8×8 | 10 | 8 |

지형 마스크 13프레임과 숫자 9→0의 저장 글리프 10개를 모두 제공합니다. `countdown_digit(value)`는 `9-value` 프레임을 선택합니다. 전체 숫자 순서는 원본 형상에서 확인했으며 Lemmix의 5→1 부분 목록과도 일치합니다. 좌우 데이터는 별도 원본 이미지이며 자동 반전하지 않습니다.

## 고정 레퍼런스와 차이

| 근거 | 확인 내용 |
|---|---|
| [Lemmix Styles.Base](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Styles.Base.pas#L456) | 마스크 5종의 주소·크기·프레임, 0x154의 카운트다운 5프레임 |
| [Lemmix Game](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas#L1812) | 비영 소스로 지형 삭제, 적용 위치와 표시용 숫자 5→1 선택 |
| [TS MaskProvider](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/mask-provider.ts) | 마스크 5종 뒤에서 숫자 10개를 순서대로 읽음 |
| [TS Mask](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/mask.ts), [MaskList](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/mask-list.ts) | 원시 비트·부호 표현·조회 극성·길이 getter 문제 |

커밋은 Lemmix `40e9bc34451f0e9127fd53b290a3877e700d143d`, TS `10c7c8b9167258503d02589cf048e2d83a43dd0a`입니다. 원본 50개·기존 소스 46개와 추가 고정 소스 식별값을 확인했습니다.

Lemmix의 카운트다운 시작 0x154는 전체 숫자 뱅크의 4번 프레임입니다. 실제 사용 5프레임은 숫자 5·4·3·2·1이며, 주석의 zero 언급과 달리 숫자 0은 포함하지 않습니다. 그 앞 32바이트는 패딩이 아니라 숫자 9·8·7·6입니다. 마지막 8바이트는 숫자 0입니다. TS의 숫자 표시 시스템은 count를 그대로 프레임 인덱스로 전달하므로 값 기반 API와 구분합니다. 타이머 정책은 구현하지 않습니다.

TS는 `Int8Array`에 비트 1을 -128, 비트 0을 0으로 저장합니다. `Mask.at`는 원시 값이 **0일 때 true**를 반환하므로 이번 API와 극성이 반대입니다. 원시 배열을 0/1로 정규화해 비교하고, 별도로 모든 유효 좌표에서 `at`의 반대 극성을 확인했습니다. 해당 조회를 Rust로 그대로 복사하지 않습니다.

`MaskList.length`는 `this.frames` 대신 비한정 `frames`를 참조합니다. 실제 Node 실행에서 `ReferenceError`가 발생했습니다. 검증에서는 고정 소스의 프레임 수와 `getMask`를 사용하며 디코딩 알고리즘을 수정하지 않았습니다. 이 관찰은 TypeScript 레퍼런스의 문제이며 Rust API의 길이는 고정 메타데이터에서 읽습니다.

## 위치와 적용 경계

비트맵에는 위치가 저장되어 있지 않습니다. 이번 API는 형상·방향·순서만 반환합니다.

- Lemmix의 바싱은 현재 스프라이트 좌상단을 사용합니다.
- 채굴은 스프라이트 좌상단을 기준으로 두 번째 마스크에서 xDelta와 Y+1을 추가합니다. TS의 고정 Y=-12와 Lemmix 채굴 이미지의 기준점은 다릅니다.
- 폭발은 레밍 기준 (-8,-14)이며 두 레퍼런스가 같습니다.
- 카운트다운은 Lemmix에서 X-1·현재 스프라이트 상단-12입니다. TS의 고정 (-1,-19)과 구분합니다.

적용 좌표·클리핑·강철/충돌·선택 틱·지형 삭제/추가·실제 화면은 후속 코어/렌더러 범위입니다. 마스크 해석 성공을 실제 DOS 실행 호환성으로 판정하지 않습니다.

## API와 오류 계약

[masks.rs](../../crates/lemmings-resources/src/masks.rs)는 기존 [MainDatFile](issue-16_lemming_sprites.md)을 확장하며 외부 패키지·할당·I/O 없는 `no_std` API입니다.

- `decode_masks_into`: 구간 1의 선언 길이와 정확한 388바이트 출력 길이를 쓰기 전에 검사합니다. 크기 오류는 출력을 보존합니다. 압축 명령 오류는 부분 출력 가능성이 있으므로 출력을 버립니다. 성공한 `MainMasks`는 출력만 빌립니다.
- `MainMasks::parse`·`as_bytes`: 정확한 해제 길이를 검사하고 전체 원문을 빌립니다. 파일 출처는 별도 원본 해시로 식별합니다.
- `frames(kind)`·`bashing(direction)`·`mining(direction)`: 원래 묶음·방향별 프레임을 제공합니다. `MainMaskKind::info()`와 `MAIN_MASK_KINDS`로 목록·주소·치수·프레임 수를 읽습니다.
- `countdown_digit(0..=9)`: 값 기반 조회입니다. 10 이상은 뺄셈 전에 거부합니다.
- `MaskFrames::frame(index)`: 정확한 인덱스를 검사하고 원래 순서의 `MaskBitmap`을 빌립니다. 순환·회전·복제·반전을 하지 않습니다.
- `MaskBitmap::at`·`decode_into`: 비트 1은 true입니다. 큰 좌표는 곱셈 전에 검사하며 범위 밖은 `None`입니다. 정확한 boolean 출력 길이가 아니면 아무것도 쓰지 않습니다. 프레임 원문도 `as_bytes()`로 읽습니다.

오류에 프레임 종류·인덱스·숫자·기대/실제 길이를 담고 압축 오류는 `Error::source`를 연결합니다.

## 전체 원본 대조

독립 Python은 원본 MAIN을 역방향 비트로 직접 해제하고 이전 7개 구간의 모든 바이트·해시와 대조했습니다. 고정 Lemmix/TS 소스에서 레이아웃을 읽고 전체 원문을 이진 문자열로 풀어 23프레임·3104비트를 Rust와 비교했습니다. 선택 비트는 923개입니다. Lemmix의 5→1 부분 구간도 같은 원문을 가리킵니다.

실제 고정 TS의 DAT 해제·MaskProvider·MaskList·Mask를 실행해 388바이트 소비와 23개 프레임 전체를 확인했습니다. import/로그 연결만 조정했으며 모든 원시 비트가 Rust·Python과 일치합니다. 조회 극성·길이 getter·고정 위치 차이는 별도 관찰로 기록했습니다.

[검증 매니페스트](../../assets/manifests/main_masks.json)에 원문·프레임별 0/1 해시·묶음 목록·선택 비트 수·추가 소스 7개·입력 해시와 TS 관찰을 고정했습니다. 기존 매니페스트는 변경하지 않으며 새 파일은 확보용 `fetch --manifest` 입력이 아닙니다.

## 원본 검사 재현

원본을 [확보](../../assets/README.md)한 뒤 저장소 루트에서 실행합니다. `inspect_masks MAIN_DAT_FILE OUTPUT_DIRECTORY`는 해제 구간 1과 프레임별 행 우선 0/1 파일 23개를 새 출력 디렉터리에 기록합니다. JSONL은 종류·프레임·주소·치수·숫자 값·선택 수를 담고 기존 파일은 덮어쓰지 않습니다.

아래 명령은 임시 디렉터리로 반복 실행할 수 있습니다. 현재 Rust 결과를 고정 기대값과 대조하며 독립 Python/TS 알고리즘을 다시 실행하는 명령은 아닙니다.

```sh
python3 tools/reference_assets.py verify
python3 - <<'PY'
from pathlib import Path
import hashlib, json, subprocess, tempfile
sha = lambda data: hashlib.sha256(data).hexdigest()
manifest = json.loads(Path('assets/manifests/main_masks.json').read_text())
for item in manifest['inputs']:
    assert sha(Path(item['path']).read_bytes()) == item['sha256'], item['path']
def run(example, destination):
    return subprocess.run([
        'cargo', 'run', '--quiet', '--locked', '--offline', '-p', 'lemmings-resources',
        '--example', example, '--', 'assets/local/dos-original/MAIN.DAT', str(destination)
    ], check=True, capture_output=True, text=True).stdout
with tempfile.TemporaryDirectory(prefix='issue-18-reproduce-', dir='assets/local') as folder:
    root = Path(folder)
    rows = [json.loads(s) for s in run('inspect_masks', root/'masks').splitlines()]
    run('unpack_dat', root/'sections')
    assert len(rows) == 23 and len(list((root/'masks').iterdir())) == 24
    previous = json.loads(Path('assets/manifests/lemming_sprites.json').read_text())
    for target in previous['sections']:
        data = (root/'sections'/f"part-{target['part']:03}.bin").read_bytes()
        assert len(data) == target['size'] and sha(data) == target['sha256']
    raw = (root/'masks'/'section-001.bin').read_bytes()
    assert raw == (root/'sections'/'part-001.bin').read_bytes()
    assert len(raw) == 388 and sha(raw) == manifest['section']['sha256']
    expected = {(r['kind'],r['index']):r for r in manifest['frames']}
    assert len(expected) == 23
    for row in rows:
        target = expected.pop((row['kind'],row['index']))
        assert row == {key:target[key] for key in row}
        bits = (root/'masks'/row['bit_file']).read_bytes()
        assert len(bits) == row['width'] * row['height'] and set(bits) <= {0,1}
        assert sum(bits) == row['selected_bits'] and sha(bits) == target['bit_sha256']
        size = len(bits)//8
        independent = bytes(map(int, f"{int.from_bytes(raw[row['offset']:row['offset']+size],'big'):0{len(bits)}b}"))
        assert bits == independent
    assert not expected and sum(r['width']*r['height'] for r in rows) == 3104
    assert sum(r['selected_bits'] for r in rows) == 923
print('PASS: seven section hashes, 23 frames/3104 bits, metadata and input hashes')
PY
```

## 테스트와 미리보기

합성 테스트 14개는 모든 짧은 접두부·추가 바이트, 전체 프레임·비트/행/바이트 순서·좌우 독립 이미지·숫자 9→0와 5→1 연결, 잘못된 숫자·큰 좌표/프레임, boolean 출력/경계 보호 값, 잘못된 선언/출력 길이의 버퍼 보존, 압축 오류 source와 출력만 빌리는 수명을 확인합니다.

CLI 오류 10개는 인자 누락/초과·파일 누락·짧은 헤더·후미 체크섬·구간 수 6/8·마스크 선언 길이·압축 명령·덮어쓰기 거부를 포함합니다. 원본·기존 맵·미리보기의 해시가 보존됐습니다.

로컬 `assets/local/issue-18-masks/preview.png`는 전체 23프레임을 확대하고 선택 비트를 흰색으로 표시합니다. 원본·PNG·비트 파일·JSONL·추가 소스 사본은 Git 추적에서 제외합니다.

- [결과 보고서](../reports/issue-18_main_masks.md), [진행 기록](../working/issue-18_main_masks.md), [작업일지](../orders/20261018.md#issue-18)
