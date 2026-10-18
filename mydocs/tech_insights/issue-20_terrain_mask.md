# G04-01 픽셀 지형 읽기·경계 계약

작업 이슈: [GitHub #20](https://github.com/edwardkim/ohmylemmings/issues/20). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 표현과 책임

[호환성 계약](issue-5_compatibility_baseline.md)의 마스크 정규화 형식에 맞춰 좌상단 원점·행 우선·픽셀당 1바이트·행 패딩 없는 **0=빈 공간 / 1=지형**을 사용합니다. 색상·알파·강철·일방향·Blocker·객체 트리거와 구분합니다. 그래픽의 투명 값 0x80이나 객체 합성 출처 비트를 직접 입력하면 오류입니다.

크기는 호출자가 지정합니다. 표시 폭 1584로 고정하거나 개체 좌표를 표시 영역으로 잘라내지 않습니다. 계약의 개체 x=0..1647·하단 y=163 판정은 후속 게임 로직 책임입니다. 범위 밖에 지형이 없다는 조회 결과만으로 개체 생존·벽·이탈을 결정하지 않습니다.

[고정 TS SolidLayer.hasGroundAt](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/solid-layer.ts#L14)의 외부 false 정책을 읽기용 참고로 사용했습니다. TS는 버퍼 없음도 false, 모든 비영 값은 지형으로 처리합니다. Rust는 생성 단계에 완전한 0/1 버퍼를 요구하고 오류를 반환합니다. 실제 DOS 물리·지형 변형과 같다고 판정하는 단계는 아닙니다.

## API와 안전성

[코어 terrain.rs](../../crates/lemmings-core/src/terrain.rs)의 `TerrainMask<'a>`는 호출자 `&[u8]`를 빌리는 읽기 전용 뷰입니다. `Clone`/`Copy`도 같은 원문을 참조합니다. 라이브러리는 `no_std`·무할당·I/O 없이 동작하며 코어는 리소스 크레이트를 참조하지 않습니다.

| API | 계약 |
|---|---|
| `TerrainMask::new(width, height, cells)` | 비어 있지 않은 크기 → checked 면적 → 정확한 길이 → 전체 0/1 값 순으로 검사 |
| `width()`, `height()`, `pixel_count()` | 검증된 크기와 실제 셀 수 |
| `as_bytes()` | 복사·변환하지 않은 원래 바이트 슬라이스 |
| `pixel(x: usize, y: usize)` | 지형 `Some(true)`, 내부 빈 셀 `Some(false)`, 외부 `None` |
| `row(y: usize)` | 정확한 원본 한 행, 외부 `None` |
| `is_solid(x: i32, y: i32)` | 음수·범위 밖 false, 내부는 0/1 조회. 부호 변환 실패를 처리 |

`TerrainMaskError`는 `EmptyDimensions`, `AreaOverflow`, `BufferSize`, `CellValue`를 구분합니다. 첫 잘못된 셀의 행 우선 인덱스와 값을 보존하고 Display·core::error::Error를 구현합니다. 입력을 수정하거나 잘못된 값을 정규화하지 않습니다.

생성은 O(width×height), 조회는 O(1)입니다. 곱셈 오버플로와 정확한 길이를 먼저 검증한 뒤, 각 조회에서 좌표를 검사하고 산술을 수행합니다. 따라서 `usize::MAX` 좌표도 오버플로나 행 경계 침범 없이 거부합니다. `usize`의 플랫폼별 폭을 가정하지 않고 면적을 검사하며 음수 정수를 무부호로 직접 캐스팅하지 않습니다.

## 합성 전수 검증

새 테스트 12개는 Red에서 모두 실패했고 기존 코어 2개·리소스 181개는 통과했습니다. 구현 후 코어 14개·리소스 181개가 모두 통과했습니다. 빈 크기·면적 오버플로·길이 우선순위·모든 비이진 값 2~255·첫 오류 위치·비대칭 배치·1행/1열·1648×164 임의 크기·원문 포인터/행 슬라이스·외부 canary·i32/usize 극단 값을 확인합니다.

폭과 높이가 각각 1~3인 모든 이진 마스크 **682개**를 열거했습니다. 각 마스크 주변 한 칸을 포함한 **15970개 좌표**를 독립 좌표 집합과 비교했습니다. `pixel`의 내부/외부 구분, 부호 조회, 모든 행의 원문과 불변성을 확인합니다.

[소스 매니페스트](../../assets/manifests/reference_sources.json)의 고정 SolidLayer 1290바이트·git blob `ed2f4ea51f2dff3f5b39249ac84c71992f031af9`·SHA-256 `cf1cd1a45d2bb6fd7fb266ad69bfb0e7787ecde397b9990ecc830660c8c655e5`를 검증했습니다. 실제 TS 클래스를 수정 없이 Node 24.11.0의 `--experimental-transform-types`로 실행해 같은 좌표 집합에 대조했습니다. 새 npm 의존성은 없습니다.

폭→높이→패턴 순서, 각 패턴의 Y→X 조회 순서로 부호 결과 0/1과 일반 조회 결과 0/1/외부 2를 연결하고 모든 행 바이트를 뒤에 연결한 FNV-1a 64비트 서명은 **`2eeedec0fe771b08`**입니다. Rust 테스트와 실제 TS의 서명이 같습니다. 이는 이 합성 조회 스트림의 비교값이며 실제 게임 실행 인증이나 암호학적 해시가 아닙니다.

## 재현 명령

저장소 루트에서 `make check`는 원본·Node 없이 모든 합성 테스트·문서 예제·포맷·Clippy·빌드·WASM 컴파일을 검사합니다. 해당 전수 테스트의 서명은 아래로 확인합니다.

```sh
cargo test --locked --offline -p lemmings-core terrain::tests::exhaustive_small_masks_match_coordinate_sets_and_pinned_ts_digest -- --exact --nocapture
```

실제 TS 대조는 Node 24.11.0과 [확보 도구](../../assets/README.md#레퍼런스-소스)로 준비한 고정 소스가 있을 때 실행합니다. 원본 게임 데이터는 필요 없습니다. 임시 TS 파일은 자동 제거합니다.

```sh
python3 tools/reference_assets.py verify --manifest assets/manifests/reference_sources.json
python3 - <<'PY'
from pathlib import Path
import hashlib, json, subprocess, tempfile
source = Path('assets/local/reference-sources/lemmings-ts__src__game__resources__solid-layer.ts')
assert hashlib.sha256(source.read_bytes()).hexdigest() == 'cf1cd1a45d2bb6fd7fb266ad69bfb0e7787ecde397b9990ecc830660c8c655e5'
harness = r'''
let masks = 0, queries = 0, hash = 0xcbf29ce484222325n;
const mix = v => { hash = BigInt.asUintN(64, (hash ^ BigInt(v)) * 0x100000001b3n); };
for (let w=1; w<=3; w++) for (let h=1; h<=3; h++) for (let pattern=0; pattern<2**(w*h); pattern++) {
    const coords = [];
    for (let y=0; y<h; y++) for (let x=0; x<w; x++) coords.push([x,y]);
    const points = coords.filter((_,i) => Boolean(pattern & (1<<i)));
    const cells = new Int8Array(coords.map((_,i) => (pattern>>i)&1));
    const layer = new SolidLayer(w,h,cells);
    for (let y=-1; y<=h; y++) for (let x=-1; x<=w; x++) {
        const wanted = points.some(([px,py]) => px===x && py===y);
        const actual = layer.hasGroundAt(x,y);
        if (actual!==wanted) throw new Error('query mismatch');
        mix(Number(actual));
        mix(x>=0 && x<w && y>=0 && y<h ? Number(wanted) : 2);
        queries++;
    }
    for (const v of cells) mix(v);
    masks++;
}
console.log(JSON.stringify({masks,queries,fnv1a64:hash.toString(16).padStart(16,'0')}));
'''
with tempfile.TemporaryDirectory(prefix='issue-20-reference-') as temporary:
    target = Path(temporary) / 'check.ts'
    target.write_text(source.read_text() + harness)
    result = subprocess.run(['node', '--experimental-transform-types', str(target)],
                            check=True, capture_output=True, text=True)
    actual = json.loads(result.stdout)
    assert actual == {'masks': 682, 'queries': 15970, 'fnv1a64': '2eeedec0fe771b08'}
    print('PASS:', actual)
PY
```

## 상태와 다음 범위

로컬 `make check`는 Rust 단위 195개·문서 예제 15개·Python 8개와 WASM 컴파일을 통과했습니다. 문서에 기재한 Rust/TS 재현 명령을 그대로 실행해 확인했습니다. WASM 실행·원본 연결·물리 호환성은 이번 범위가 아닙니다.

G04-01 읽기·경계의 구현·검증·게시·실제 CI·원격 tree를 확인했습니다. 최종 문서 게시·CI·#20 종료는 GitHub 이슈 기록에서 추적합니다. G04 전체는 쓰기·변형 검증까지 진행 중입니다. 다음은 별도 이슈의 G04-02 지형 쓰기·셀 변경입니다.

- [결과 보고서](../reports/issue-20_terrain_mask.md), [진행 기록](../working/issue-20_terrain_mask.md), [작업일지](../orders/20261018.md#issue-20)
