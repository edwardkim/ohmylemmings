# G04-02 지형 쓰기·셀 변경 계약

작업 이슈: [GitHub #21](https://github.com/edwardkim/ohmylemmings/issues/21). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 표현과 책임

[읽기 전용 마스크 / #20](issue-20_terrain_mask.md)의 행 우선·픽셀당 1바이트·0/1 표현과 생성 검증을 재사용합니다. [코어 terrain.rs](../../crates/lemmings-core/src/terrain.rs)의 `TerrainMaskMut<'a>`는 호출자 `&mut [u8]`를 독점 대여하며 Clone/Copy를 구현하지 않습니다. 원시 가변 바이트·행을 노출하지 않고 bool 쓰기만 제공해 유효한 뷰의 모든 셀이 계속 0/1임을 보장합니다.

이번 단계는 셀 하나의 점유 변경입니다. 색상·영역/비트 마스크 변형·클리핑·강철·게임의 벽/생존 규칙·리소스 연결·틱은 후속 범위입니다. 크기를 표시 폭으로 고정하지 않습니다.

## API와 대여 수명

| API | 계약 |
|---|---|
| `TerrainMaskMut::new(width, height, cells)` | 기존 TerrainMask 생성 검증을 그대로 호출. 모든 오류에서 버퍼 보존 |
| `set_pixel(x: usize, y: usize, solid: bool)` | `Some(true)` 변경됨, `Some(false)` 기존 값 유지, `None` 범위 밖. 좌표 검사 후 산술·쓰기 |
| `set_solid(x: i32, y: i32, solid: bool)` | 같은 결과 계약. 음수·부호 변환 실패도 `None`, 전체 버퍼 보존 |
| `as_read_only(&self)` | 복사·재검사 없는 짧은 `TerrainMask<'_>` 재대여. 이 뷰를 사용하는 동안 쓰기 금지 |
| `into_read_only(self)` | 가변 뷰를 소비하고 원래 버퍼 수명의 `TerrainMask<'a>`로 전환 |

읽기·행·크기는 읽기 뷰의 기존 API를 사용합니다. 읽기 뷰는 데이터 복사본이 아니며, 대여가 끝난 뒤에야 다음 쓰기를 할 수 있습니다. 소비 전환 후에는 원래 가변 래퍼보다 오래 읽기 뷰를 사용할 수 있지만 원본 버퍼보다 오래 보유할 수 없습니다. E0502 compile-fail 문서 예제로 읽기/쓰기 동시 별칭 거부를 확인합니다.

생성은 O(width×height), 셀 변경·읽기 전환은 O(1)입니다. 생성 오류 타입·순서·기존 읽기 API는 변경하지 않았습니다. 라이브러리의 `no_std`·무할당·I/O 없음·안전한 Rust와 코어 의존성 0개를 유지합니다.

## 레퍼런스와 경계 차이

[고정 TS SolidLayer](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/solid-layer.ts#L31)의 실제 `setGroundAt`·`clearGroundAt`를 유효 좌표에서 실행했습니다. TS 함수는 변경 여부를 반환하지 않아 하네스가 쓰기 전/후 조회 차이를 측정합니다. Rust는 이 변경 여부를 반환하고 반복 쓰기를 유지로 구분합니다.

TS의 쓰기에는 좌표 검사가 없어 읽기와 경계 계약이 다릅니다. 3×2 마스크에서 `(3,0)` 쓰기는 다음 행 첫 셀인 인덱스 3을, `(-1,1)` 쓰기는 앞 행 마지막 셀인 인덱스 2를 변경합니다. Rust는 두 경우 모두 `None`과 전체 버퍼 불변을 보장합니다. 범위 밖 TS 쓰기와 일치한다고 판정하지 않습니다.

[기존 소스 매니페스트](../../assets/manifests/reference_sources.json)의 46개 파일을 검증했고 SolidLayer 1290바이트·git blob `ed2f4ea51f2dff3f5b39249ac84c71992f031af9`·SHA-256 `cf1cd1a45d2bb6fd7fb266ad69bfb0e7787ecde397b9990ecc830660c8c655e5`를 대조했습니다. 실제 원문을 수정 없이 Node 24.11.0의 `--experimental-transform-types`로 실행했으며 npm 패키지를 추가하지 않았습니다.

## 합성 전수 검증

새 테스트 12개는 Red에서 모두 실패했고 기존 코어 14개·리소스 181개는 통과했습니다. 구현 후 코어 26개·리소스 181개가 통과했습니다. 생성 실패/복구·모든 비이진 값·원문 포인터·canary·인접 행/네 모서리·반복 쓰기·i32/usize 극단 좌표·독립 가변 하위 슬라이스·읽기 재대여/소비 수명을 확인합니다.

폭·높이 1~3의 모든 이진 마스크 **682개**를 열거하고 각 주변 한 칸 좌표마다 false·false·true·true·원래 값으로 쓰기를 시도했습니다. 전체 **79850번 시도** 중 유효 좌표 쓰기는 **27530번**입니다. 매 시도 후 결과와 전체 버퍼를 독립 좌표 집합에 대조하고 각 좌표의 마지막 시도 후 초기 상태로 복원했습니다. 두 Rust 쓰기 API를 교차 사용합니다.

실제 TS는 유효 좌표 쓰기에 사용하고, 외부 시도에는 명시한 무변경/거부 정책을 적용한 비교 스트림을 만들었습니다. 폭→높이→패턴→Y→X→5번 시도 순서로 결과(변경 1/유지 0/외부 2)와 매번 전체 행 우선 바이트를 연결한 FNV-1a 64비트 서명은 **`03ec6f2a34e44882`**이며 Rust와 일치합니다. 이는 합성 스트림 비교값이며 실제 게임 실행이나 암호학적 해시를 뜻하지 않습니다.

## 재현 명령

루트의 `make check`는 원본·Node 없이 모든 테스트·문서 예제·포맷·Clippy·빌드·WASM 컴파일을 검사합니다. 전수 쓰기 테스트의 서명은 다음으로 확인합니다.

```sh
cargo test --locked --offline -p lemmings-core terrain::edit_tests::exhaustive_cell_edits_match_coordinate_sets_and_pinned_ts_stream -- --exact --nocapture
```

실제 TS 대조는 Node 24.11.0과 [확보 도구](../../assets/README.md#레퍼런스-소스)로 준비한 고정 소스로 실행합니다. 원본 게임 데이터는 필요 없고 임시 TS 파일은 자동 제거합니다.

```sh
python3 tools/reference_assets.py verify --manifest assets/manifests/reference_sources.json
python3 - <<'PY'
from pathlib import Path
import hashlib, json, subprocess, tempfile
source = Path('assets/local/reference-sources/lemmings-ts__src__game__resources__solid-layer.ts')
assert hashlib.sha256(source.read_bytes()).hexdigest() == 'cf1cd1a45d2bb6fd7fb266ad69bfb0e7787ecde397b9990ecc830660c8c655e5'
harness = r'''
let masks=0, attempts=0, validWrites=0, hash=0xcbf29ce484222325n;
const mix=v=>{hash=BigInt.asUintN(64,(hash^BigInt(v))*0x100000001b3n);};
for(let w=1;w<=3;w++) for(let h=1;h<=3;h++) for(let pattern=0;pattern<2**(w*h);pattern++) {
    const coords=[];
    for(let y=0;y<h;y++) for(let x=0;x<w;x++) coords.push([x,y]);
    const cells=new Int8Array(coords.map((_,i)=>(pattern>>i)&1)), initial=Array.from(cells);
    const points=new Set(coords.filter((_,i)=>Boolean(pattern&(1<<i))).map(([x,y])=>`${x},${y}`));
    const layer=new SolidLayer(w,h,cells);
    for(let y=-1;y<=h;y++) for(let x=-1;x<=w;x++) {
        const inside=x>=0&&x<w&&y>=0&&y<h, key=`${x},${y}`, restore=points.has(key);
        for(const solid of [false,false,true,true,restore]) {
            let result=2;
            if(inside) {
                const before=layer.hasGroundAt(x,y), changed=points.has(key)!==solid;
                if(solid) {layer.setGroundAt(x,y);points.add(key);}
                else {layer.clearGroundAt(x,y);points.delete(key);}
                const after=layer.hasGroundAt(x,y);
                if(after!==solid || (after!==before)!==changed) throw new Error('write mismatch');
                result=Number(after!==before); validWrites++;
            }
            mix(result);
            for(let i=0;i<cells.length;i++) {
                const [px,py]=coords[i];
                if(cells[i]!==Number(points.has(`${px},${py}`))) throw new Error('neighbor mismatch');
                mix(cells[i]);
            }
            attempts++;
        }
    }
    if(initial.some((v,i)=>v!==cells[i])) throw new Error('restore mismatch');
    masks++;
}
const alias=[];
for(const [x,y] of [[3,0],[-1,1]]) {
    const cells=new Int8Array(6);
    new SolidLayer(3,2,cells).setGroundAt(x,y);
    alias.push(Array.from(cells).flatMap((v,i)=>v?[i]:[]));
}
console.log(JSON.stringify({masks,attempts,validWrites,fnv1a64:hash.toString(16).padStart(16,'0'),alias}));
'''
with tempfile.TemporaryDirectory(prefix='issue-21-reference-') as temporary:
    target = Path(temporary) / 'check.ts'
    target.write_text(source.read_text() + harness)
    result = subprocess.run(['node', '--experimental-transform-types', str(target)],
                            check=True, capture_output=True, text=True)
    actual = json.loads(result.stdout)
    assert actual == {'masks': 682, 'attempts': 79850, 'validWrites': 27530,
                      'fnv1a64': '03ec6f2a34e44882', 'alias': [[3], [2]]}
    print('PASS:', actual)
PY
```

## 상태와 다음 범위

로컬 `make check`는 Rust 단위 207개·문서 테스트 17개(별칭 거부 compile-fail 1개 포함)·Python 8개와 WASM 컴파일을 통과했습니다. 문서의 Rust/TS 명령을 그대로 실행해 확인했습니다. 구현 게시·실제 CI·원격 tree까지 확인했습니다. 최종 문서 게시·CI·#21 종료는 GitHub 이슈 기록에서 추적합니다. G04 전체는 영역/마스크 기반 추가·삭제 검증까지 진행 중이며 다음은 별도 이슈의 G04-03입니다. 실제 DOS/WASM 실행·게임 물리·트리거는 인증하지 않습니다.

- [결과 보고서](../reports/issue-21_terrain_edits.md), [진행 기록](../working/issue-21_terrain_edits.md), [작업일지](../orders/20261018.md#issue-21)
