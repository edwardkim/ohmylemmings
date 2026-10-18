# G03-01 DOS 기본 레벨 레코드 계약

작업 이슈: [GitHub #10](https://github.com/edwardkim/ohmylemmings/issues/10). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 범위와 고정 출처

[G02](issue-9_dat_decompression.md)로 해제한 2048바이트 기본 레코드를 읽습니다. [원본 매니페스트](../../assets/manifests/dos_original.json)의 LEVEL000~009.DAT는 각각 8개 구간을 가집니다. ODDTABLE 변형과 표시 순서 연결은 G03-02, 그래픽 결합·렌더링은 후속 G03 범위입니다.

- Lemmix `40e9bc34451f0e9127fd53b290a3877e700d143d`: [TLVLRec 배치](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Dos.Structures.pas#L18-L106), [실제 레벨 변환](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Level.Loader.pas#L74-L193).
- Lemmings.ts `10c7c8b9167258503d02589cf048e2d83a43dd0a`: [LevelReader](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/lemmings/level-reader.ts#L46-L188).

[소스 매니페스트](../../assets/manifests/reference_sources.json)로 46개 파일을 다시 검증했습니다. 형식·좌표는 Lemmix의 실제 변환 코드를 기준으로 선택했습니다. 레퍼런스끼리 다른 값과 지원 정책을 아래에 구분합니다. 실제 DOS 화면과의 비교는 후속 실행 단계에서 수행합니다.

## 레코드 구성과 헤더 정책

| 시작 위치 | 길이 | 내용 |
|---|---|---|
| `0x0000` | 32 | big-endian u16 헤더 16개 |
| `0x0020` | 256 | 객체 32개 × 8바이트 |
| `0x0120` | 1600 | 지형 400개 × 4바이트 |
| `0x0760` | 128 | 강철 영역 32개 × 4바이트 |
| `0x07e0` | 32 | 제목 원문 바이트 |

헤더 순서는 생성률·개체 수·구출 수·시간(분), 직업 8종 수량, 화면 X·GROUND 선택자·특수 그래픽 선택자·Superlemming입니다. 직업은 climber, floater, bomber, blocker, builder, basher, miner, digger 순서입니다.

지원하는 헤더 범위는 생성률 1~99, 개체 수 1~255, 구출 수 1~개체 수, 시간 1~99, 직업 수량 0~99입니다. GROUND는 0~4, 특수 선택자는 0~4이며 0은 없음, 1~4는 VGASPEC0~3입니다. Superlemming은 0 또는 `0xffff`만 허용합니다. 이 정책은 원작 프로필과 Lemmix의 수치 범위를 기준으로 하며, 원본의 실측 범위와 동일하다는 뜻은 아닙니다.

범위 밖 값은 보정하지 않고 오류를 반환합니다. 화면 X는 원시 u16을 보존하며 카메라 경계로 제한하지 않습니다. 제목도 32바이트를 그대로 빌리며 공백·NUL·비 UTF-8 바이트를 허용합니다. `title()`은 별도의 UTF-8 변환 결과를 반환하고 DOS 문자 인코딩을 추측하지 않습니다. 제목 위치는 실제 구조와 TS가 일치하는 `0x07e0`이며 Pascal의 별도 `LVL_TITLE_OFFSET` 상수 값은 사용하지 않습니다.

## 배치 해석과 레퍼런스 차이

모든 좌표는 자르거나 화면 경계로 보정하지 않습니다. `slot`은 빈 항목을 제외하기 전의 원래 0부터 시작하는 번호이며 순서를 유지합니다.

| 구분 | 빈 항목 | 좌표·크기 | ID·플래그 |
|---|---|---|---|
| 객체 | 8바이트가 모두 0 | big-endian X - 16, unsigned Y | ID 하위 4비트; modifier `0x80` no-overwrite, `0x40` only-on-terrain; display가 정확히 `0x8f`이면 상하 반전 |
| 지형 | 4바이트가 모두 `0xff` | unsigned X 12비트 - 16; signed Y 9비트 - 4 | ID 하위 6비트; 첫 바이트 상위 3비트는 no-overwrite·반전·erase |
| 강철 | 4바이트가 모두 0 | 앞 9비트 X × 4 - 16; 다음 7비트 Y × 4; size 상위/하위 nibble은 너비/높이, 각각 (값 + 1) × 4 | 마지막 예약 바이트는 0 요구 |

객체의 flags가 0이어도 나머지 바이트가 있으면 활성입니다. 객체 modifier·ID의 남는 비트와 지형의 미사용 비트는 해석에서 무시하고 원본 전체 바이트로 보존합니다. 원본 지형에는 X 쪽 미사용 비트가 켜진 항목 11개, ID 쪽 68개가 있어 이를 손상으로 거부하면 안 됩니다.

TS는 객체 flags 0을 빈 항목으로 취급하며 지형 Y의 부호 경계가 `>256`입니다. 여기서는 Lemmix의 `>=256`을 채택해 인코딩 256을 -260으로 읽습니다. 원본에서는 이 경계가 관찰되지 않았으므로 합성 테스트로 고정했습니다. 강철은 TS의 좌표 비트 배치와 너비/높이 순서가 달라 원본의 활성 392개 영역 모두 해석 차이가 납니다. Lemmix의 읽기·쓰기 코드가 사용하는 비트 배치를 채택했으며 독립적인 비트 문자열 분석과 대조했습니다. TS와 전체 배치가 일치한다고 주장하지 않습니다.

## API와 오류

[level.rs](../../crates/lemmings-resources/src/level.rs)의 `LevelRecord::parse(&bytes)`가 정확한 길이·헤더 범위·강철 예약 바이트를 먼저 검증합니다. 실패 시 record를 반환하지 않습니다. `header()`는 값 복사, `as_bytes()`·`title_bytes()`는 입력을 빌립니다. `objects()`·`terrains()`·`steel()`은 검증된 배치를 반환하는 `FusedIterator`로, 끝난 후 계속 종료하고 새 호출로 다시 순회할 수 있습니다.

`LevelError`는 길이 불일치, 필드 범위 오류, 지원하지 않는 Superlemming 값, 강철 예약 바이트를 구분합니다. 오류의 `offset`은 해제된 레코드 안의 위치입니다. DAT 파일·구간 문맥은 호출자가 별도로 연결합니다. 라이브러리는 `no_std`·무할당·외부 의존성 0개이며 코어 크레이트·플랫폼 I/O에 의존하지 않습니다.

## 원본 검사 재현

[원본 확보](../../assets/README.md) 후 저장소 루트에서 실행합니다. 예제는 원본 DAT를 직접 읽고 구간별 고정 크기 버퍼에 해제한 뒤 파싱합니다. 출력은 무시되는 로컬 폴더 안에 둡니다.

```sh
python3 tools/reference_assets.py verify
cargo run -p lemmings-resources --example inspect_levels --locked --offline -- \
  assets/local/dos-original/LEVEL*.DAT > assets/local/level-issue-10.jsonl
python3 - <<'PY'
import json
from pathlib import Path
rows = [json.loads(line) for line in Path('assets/local/level-issue-10.jsonl').read_text().splitlines()]
assert len(rows) == 80
manifest = json.loads(Path('assets/manifests/dos_compatibility.json').read_text())
by_key = {(f"LEVEL{r['file_index']:03}.DAT", r['part']): r for r in rows}
count = 0
for entry in manifest['levels']:
    if entry['odd_index'] is not None:
        continue
    row = by_key[entry['file'], entry['part']]
    for key in ['release_rate', 'total', 'required', 'minutes', 'skills', 'screen_x', 'graphics', 'special']:
        assert row[key] == entry[key]
    assert bytes.fromhex(row['title_hex']).decode('ascii').strip() == entry['title']
    count += 1
assert count == 80
assert [sum(len(r[key]) for r in rows) for key in ['objects', 'terrain', 'steel']] == [454, 11964, 392]
print('PASS: 80 base records; catalog metadata and placement counts')
PY
```

JSONL의 `file_index`는 전달한 파일 순서, `part`는 0부터 시작하는 DAT 구간 번호, `dat_offset`은 구간 헤더의 파일 위치입니다. 배치 배열은 객체 `[slot,x,y,id,반전,no-overwrite,only-on-terrain]`, 지형 `[slot,x,y,id,반전,no-overwrite,erase]`, 강철 `[slot,x,y,width,height]` 순서입니다. 제목은 손실 없이 비교하도록 `title_hex`로 출력합니다.

검증 결과: [보고서](../reports/issue-10_level_records.md), [진행 기록](../working/issue-10_level_records.md), [작업일지](../orders/20261018.md#issue-10).
