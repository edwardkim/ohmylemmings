# G02-02 DOS DAT 압축 해제 계약

작업 이슈: [GitHub #9](https://github.com/edwardkim/ohmylemmings/issues/9). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 입력과 고정 출처

[G02-01](issue-8_dat_sections.md)의 `DatSection`을 입력으로 사용합니다. 헤더·본문 경계·예약 워드·체크섬은 이미 검증되어 있으며 원본 프로필은 `dos-original-vga`입니다.

- Lemmix `40e9bc34451f0e9127fd53b290a3877e700d143d`: [GetNextBits와 역방향 출력](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Dos.Compression.pas#L174-L217), [명령 디스패치](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Dos.Compression.pas#L219-L252).
- Lemmings.ts `10c7c8b9167258503d02589cf048e2d83a43dd0a`: [BitReader](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/file/bit-reader.ts#L12-L53), [출력과 참조 복사](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/file/bit-writer.ts#L24-L77), [UnpackFilePart](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/file/unpack-file-part.ts#L56-L113).

[소스 매니페스트](../../assets/manifests/reference_sources.json)의 파일 해시를 재검증하고 대조했습니다. 두 구현의 정상 알고리즘을 참고하되, 범위 오류를 로그만 남기고 계속 처리하는 동작은 채택하지 않습니다.

## 역방향 비트와 명령

본문의 마지막 바이트부터 앞쪽으로 이동하며 각 바이트는 LSB부터 읽습니다. 다중 비트 필드에서는 먼저 읽은 비트가 결과의 상위 비트입니다. 출력도 버퍼 끝에서 시작해 앞으로 채웁니다. 초기 마지막 바이트에서 헤더의 `initial_bits`만 사용하고, 이후 바이트는 8비트씩 사용합니다. 초기 비트 수가 0이면 마지막 바이트는 건너뛰며 본문 XOR에는 여전히 포함됩니다.

| 읽는 순서의 접두 비트 | 추가 필드 | 동작 |
|---|---|---|
| `00` | 길이 3비트 | 값 + 1바이트 리터럴, 길이 1~8 |
| `01` | 거리 8비트 | 길이 2 복사, 거리 값 + 1, 최대 256 |
| `100` | 거리 9비트 | 길이 3 복사, 최대 거리 512 |
| `101` | 거리 10비트 | 길이 4 복사, 최대 거리 1024 |
| `110` | 길이 8비트, 거리 12비트 | 길이 값 + 1, 거리 값 + 1; 최대 256바이트·거리 4096 |
| `111` | 길이 8비트 | 값 + 9바이트 리터럴, 길이 9~264 |

리터럴의 각 바이트는 8비트 필드로 읽습니다. 복사는 현재 쓰는 위치보다 오른쪽의 이미 해제된 바이트를 참조합니다. 복사 거리가 1이고 길이가 크면 방금 쓴 바이트를 계속 재사용하므로, 역방향의 겹침 복사를 순차적으로 처리합니다.

손으로 구성한 기준 fixture는 `00 / 000 / 01000001`이며 출력은 `A`입니다. 초기 비트 5, 본문 `[0x82, 0x00]`으로 읽기 방향과 필드 비트 순서를 고정합니다. 테스트용 bit-string 패커 결과도 이 고정 바이트열과 대조합니다.

## 버퍼·오류·입력 소비 정책

[decompress.rs](../../crates/lemmings-resources/src/decompress.rs)의 `DatSection::decompress_into(&mut output)`은 정확히 선언된 크기의 버퍼를 요구하고 성공 시 `Ok(())`를 반환합니다. 큰 버퍼는 필요한 크기의 슬라이스로 전달합니다. 라이브러리는 `no_std`이며 할당·파일 I/O·외부 의존성이 없습니다.

| 오류 | 조건과 출력 상태 |
|---|---|
| `OutputSize` | 버퍼 길이가 선언과 다름; 쓰기 전 거부하므로 버퍼 유지 |
| `UnexpectedEnd` | 필드를 읽을 유효 비트가 부족함; 해당 비트 읽기는 위치를 전진시키지 않음 |
| `OutputOverrun` | 명령 길이가 남은 출력보다 큼; 해당 명령의 쓰기를 시작하지 않음 |
| `InvalidBackReference` | 거리가 이미 생성한 바이트 수보다 큼; 호출자 버퍼의 미기록 부분 참조 금지 |
| `TrailingBits` | 출력을 모두 채웠지만 선언된 유효 비트가 남음; 0비트도 오류 |

`OutputSize` 외 오류는 앞선 명령의 결과 또는 완성된 출력이 남을 수 있으므로, 실패한 버퍼는 버립니다. 모든 오류의 `offset`은 구간 헤더의 파일 위치입니다. `bit_offset`·`command_bit`은 역방향 읽기 순서로 센 논리적 비트 수이며 물리적인 파일 비트 주소가 아닙니다.

본문 앞에 남은 바이트·유효 비트는 모두 거부합니다. 헤더가 유효 범위 밖으로 선언한 마지막 바이트의 상위 패딩은 무시합니다. 이 정책은 원본 102개 구간 모두 해제 후 유효 비트가 정확히 0개 남는 관찰과 두 레퍼런스의 비트 순서를 바탕으로 정했습니다. 모든 커스텀 DAT 변형의 호환성을 주장하지 않습니다.

크기를 검사한 뒤 출력 위치를 감소시키고, 복사 거리는 이미 초기화된 접미부 길이와 대조합니다. 모든 명령은 1바이트 이상 출력하므로 종료하며 작업량은 입력·출력 길이에 비례하고 내부 공간은 일정합니다. 입력과 출력의 별도 소유권은 Rust의 빌림 규칙으로 유지합니다.

## 개발용 해제와 고정 해시 재현

원본은 [확보 안내](../../assets/README.md)에 따라 준비합니다. 호스트 예제만 파일을 읽고 출력 버퍼를 할당합니다. 아래 출력 폴더는 새 폴더를 사용합니다. 예제는 기존 `part-*.bin`을 덮어쓰지 않으며, 후속 구간에서 실패하면 앞서 생성한 파일이 남을 수 있습니다. 성공 종료를 확인한 결과만 사용합니다.

```sh
python3 tools/reference_assets.py verify
cargo build -p lemmings-resources --example unpack_dat --locked --offline
for dat_file in assets/local/dos-original/ADLIB.DAT \
  assets/local/dos-original/LEVEL*.DAT \
  assets/local/dos-original/MAIN.DAT \
  assets/local/dos-original/VGAGR*.DAT \
  assets/local/dos-original/VGASPEC*.DAT
do
  target/debug/examples/unpack_dat "$dat_file" \
    "assets/local/decoded-issue-9/${dat_file##*/}" || exit 1
done
```

같은 저장소 루트에서 기존 81개 기대값을 대조합니다. 원본·해제 결과는 모두 무시되는 `assets/local/` 아래에 있습니다.

```sh
python3 - <<'PY'
import hashlib
import json
from pathlib import Path
manifest = json.loads(Path('assets/manifests/dos_compatibility.json').read_text())
for entry in manifest['inputs']:
    assert hashlib.sha256(Path(entry['path']).read_bytes()).hexdigest() == entry['sha256']
root = Path('assets/local/decoded-issue-9')
for entry in manifest['decoded_parts']:
    data = (root / entry['file'] / f"part-{entry['part']:03}.bin").read_bytes()
    assert len(data) == entry['size']
    assert hashlib.sha256(data).hexdigest() == entry['sha256']
assert len(manifest['decoded_parts']) == 81
assert len(list(root.glob('*.DAT/part-*.bin'))) == 102
print('PASS: 81 frozen output hashes; 102 decoded parts')
PY
```

검증 수치·TDD·실제 CI·남은 범위는 [보고서](../reports/issue-9_dat_decompression.md), 현재 상태는 [진행 기록](../working/issue-9_dat_decompression.md), 당일 작업은 [일지](../orders/20261018.md#issue-9)에 기록합니다.
