# G03-02 ODDTABLE·표시 카탈로그 계약

작업 이슈: [GitHub #11](https://github.com/edwardkim/ohmylemmings/issues/11). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 범위와 출처

[G03-01](issue-10_level_records.md)의 기본 레코드를 Fun·Tricky·Taxing·Mayhem 각 30개 표시 레벨에 연결하고, 필요한 항목의 초기 수치·제목만 ODDTABLE에서 적용합니다. 그래픽 이미지 해석·지형 재구성·실제 실행은 이번 범위에 포함하지 않습니다.

- Lemmix `40e9bc34451f0e9127fd53b290a3877e700d143d`: [120개 SectionTable](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Styles.Dos.pas#L28-L48), [GetEntry 연결 계산](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Styles.Dos.pas#L183-L212), [ODD 구조](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Dos.Structures.pas#L182-L199).
- Lemmings.ts `10c7c8b9167258503d02589cf048e2d83a43dd0a`: [설정](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/public/data/config.json), [OddTableReader](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/lemmings/odd-table-reader.ts#L16-L74).

[소스 매니페스트](../../assets/manifests/reference_sources.json)의 해시를 확인했습니다. Lemmix의 1~160 인코딩 순서표와 TS의 부호 있는 십진 파일/구간 설정은 120개 모두 일치합니다. 생산 코드와 합성 테스트는 각각 두 표현을 사용해 잘못된 전사를 검사합니다.

## 연결 규칙과 의존 방향

기존 코어 `LevelId`는 난이도와 표시 번호 1~30을 보장합니다. `CatalogEntry::for_level(id)`는 고정 표 값에서 1을 뺀 뒤, 하위 비트로 ODD 사용 여부를 정하고 나머지를 2로 나눈 기본 번호를 얻습니다. 기본 번호를 8로 나눈 몫이 파일 번호 0~9, 나머지가 구간 0~7입니다. ODD를 쓰면 기본 번호 자체가 ODD 인덱스입니다.

Fun 1은 LEVEL009.DAT의 구간 1이며 ODD를 쓰지 않습니다. Fun 8은 표시 전체 순번 7이지만 LEVEL000.DAT 구간 6, ODD 인덱스 6입니다. [기존 비교 계약](issue-5_compatibility_baseline.md)의 주의사항처럼 TS의 표시 순번을 ODD 조회에 넘기는 경로와 잘못된 범위 조건을 이식하지 않습니다.

이번 단계에서 `lemmings-resources → lemmings-core` 로컬 의존 하나를 추가했습니다. 검증된 표시 ID를 재사용하며 코어는 리소스 파서·파일 I/O에 의존하지 않습니다. 외부 패키지는 없고 두 라이브러리는 계속 `no_std`·무할당입니다. [모듈 경계](../plans/issue-6_rust_workspace.md)도 이 결정에 맞춰 갱신했습니다.

## ODD 형식과 검증

한 레코드는 56바이트입니다. 앞 24바이트는 big-endian u16 12개로 생성률·개체 수·구출 수·시간(분)·직업 8종 수량이며, 뒤 32바이트는 제목입니다. 제목의 공백·NUL·비 UTF-8 원문을 보존하며 UTF-8 해석은 별도 `Result`입니다.

`OddTable::parse`는 길이가 56의 배수인지 검사하고 **전체 레코드**의 수치를 기본 레벨과 같은 공통 검증 함수로 확인합니다. 선택하지 않는 레코드도 손상됐으면 실패합니다. 빈 표는 허용하지만 조회는 범위 오류이며, 부분 레코드를 버리지 않습니다. 원본은 4480바이트·80개 레코드로 모두 유효하고 40개를 카탈로그에서 사용합니다.

`get(index)`는 곱셈 전에 범위를 확인하므로 `usize::MAX`도 오류입니다. `OddError`는 길이·조회 범위·레코드 필드 오류를 구분합니다. 필드 오류는 레코드 인덱스를 감싸며 내부 `LevelError.offset`은 그 56바이트 레코드 기준 위치입니다.

## 파일 결합과 변형 적용

`DosLevelFile::parse(file_index, bytes)`는 파일 번호 0~9, 전체 DAT 구간 경계·XOR, 정확히 8개 구간, 각 구간의 선언 해제 크기 2048을 검사합니다. 구간 뒤 쓰레기 데이터도 거부합니다. 구간 메타데이터 8개만 고정 배열에 저장하고 명령 해제는 선택 시 수행합니다. 파일 번호는 호출자가 붙인 식별값이며 바이트의 출처 인증이나 SHA 검증을 대신하지 않습니다.

`entry.load(&file, optional_odd, &mut output)`은 다음 순서로 처리합니다.

1. 파일 번호 일치와 필요한 ODD 존재·인덱스를 확인합니다. 이 오류에서는 출력 버퍼를 바꾸지 않습니다.
2. 선택된 구간을 호출자 소유 2048바이트 버퍼에 해제하고 기본 레벨 전체를 검증합니다. 이 단계의 오류에서는 출력이 바뀔 수 있으므로 버립니다.
3. ODD가 있으면 초기 수치와 제목만 별도 유효 뷰에 적용합니다. 화면 X·GROUND·특수 그래픽·Superlemming·모든 배치는 기본값을 유지합니다.

`ResolvedLevel.header()`와 `title_bytes()`는 최종 표시 수치·제목입니다. `base_bytes()`는 변경하지 않은 기본 레코드, 배치 iterator는 그 기본 레코드를 사용합니다. 입력 DAT·ODD와 기본 버퍼에 변형 값을 덮어쓰지 않습니다. `CatalogError`는 파일 결합·구간 수/크기·누락 ODD·DAT·해제·레벨 오류에 파일/구간 또는 ODD 인덱스 문맥을 붙이고 원인 오류를 보존합니다.

## 원본 검사 재현

[리소스 확보](../../assets/README.md) 후 저장소 루트에서 실행합니다. 예제의 캐시·파일 I/O·JSON 출력은 호스트에만 있습니다. 원본 데이터가 들어 있는 `base_hex` 출력도 무시되는 로컬 폴더에 둡니다.

```sh
python3 tools/reference_assets.py verify
cargo run -p lemmings-resources --example inspect_catalog --locked --offline -- \
  assets/local/dos-original > assets/local/catalog-issue-11.jsonl
python3 - <<'PY'
import hashlib
import json
from pathlib import Path
manifest = json.loads(Path('assets/manifests/dos_compatibility.json').read_text())
for entry in manifest['inputs']:
    assert hashlib.sha256(Path(entry['path']).read_bytes()).hexdigest() == entry['sha256']
rows = [json.loads(line) for line in Path('assets/local/catalog-issue-11.jsonl').read_text().splitlines()]
assert len(rows) == len(manifest['levels']) == 120
for index, (row, expected) in enumerate(zip(rows, manifest['levels'])):
    assert row['global_index'] == index
    assert row['group'] == expected['group'] and row['number'] == expected['number']
    assert f"LEVEL{row['file']:03}.DAT" == expected['file']
    for key in ['part', 'odd_index', 'release_rate', 'total', 'required', 'minutes', 'skills', 'screen_x', 'graphics', 'special']:
        assert row[key] == expected[key]
    assert row['superlemming'] == (expected['superlemming'] == 65535)
    assert bytes.fromhex(row['title_hex']).decode('ascii').strip() == expected['title']
    base = bytes.fromhex(row['base_hex'])
    assert len(base) == 2048 and hashlib.sha256(base).hexdigest() == expected['base_record_sha256']
assert sum(row['odd_index'] is not None for row in rows) == 40
assert rows[7]['odd_index'] == 6
print('PASS: 120 routes and effective properties; 40 variants; all base hashes unchanged')
PY
```

JSONL은 표시 순서이며 `global_index`는 0~119, `number`는 난이도 내 1~30입니다. 파일·구간·ODD 인덱스와 유효 헤더·제목 hex·기본 레코드 hex·배치 개수를 제공합니다. 이 검사는 그래픽 이미지 재구성이나 실제 플레이 통과를 뜻하지 않습니다.

결과: [보고서](../reports/issue-11_level_catalog.md), [진행 기록](../working/issue-11_level_catalog.md), [작업일지](../orders/20261018.md#issue-11).
