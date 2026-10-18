# G04-03 영역·마스크 기반 지형 추가/삭제

작업 이슈: [GitHub #22](https://github.com/edwardkim/ohmylemmings/issues/22). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 표현과 API

[읽기 / #20](issue-20_terrain_mask.md)·[가변 뷰 / #21](issue-21_terrain_edits.md)의 행 우선 0/1 표현·생성 검증·대여 불변식을 유지합니다. [terrain/bulk.rs](../../crates/lemmings-core/src/terrain/bulk.rs)는 `TerrainMaskMut`의 영역 연산을 구현하는 자식 모듈입니다. 색상·리소스·할당/I/O·unsafe·외부 패키지를 추가하지 않습니다.

| API | 계약 |
|---|---|
| `fill_rect(x: i32, y: i32, width: usize, height: usize, solid: bool)` | 교집합 사각형의 모든 셀을 1 또는 0으로 변경. 폭/높이 0·완전히 외부인 영역은 무효 변경 |
| `apply_mask(x: i32, y: i32, mask: TerrainMask, solid: bool)` | 마스크의 좌상단을 (x,y)에 배치. 1인 소스 셀만 변경하고 0은 목적지 보존 |
| `TerrainChange { changed_pixels, bounds }` | 실제 값이 달라진 셀 수와 최소 경계. 변화가 없으면 `0 / None` |
| `TerrainRect { x, y, width, height }` | 변경된 목적지 셀의 반개구간 경계. 요청 영역·선택된 전체 영역과 구분 |

마스크를 회전/반전하거나 기준점을 보정하지 않습니다. 음수 배치에서도 소스의 행·열 정렬을 유지합니다. 소스는 이미 검증된 `TerrainMask`이므로 연산 중 검증 오류나 부분 실패가 없습니다. 원시 버퍼의 생성 오류는 연산 전에 처리하며 기존 목적지를 건드리지 않습니다. 소스는 불변이고 같은 저장소를 목적지와 동시에 빌리는 사용은 E0502 compile-fail 예제로 거부합니다.

반복 추가·빈 영역 삭제·선택 비트가 전부 0인 경우는 변경 수 0입니다. 마스크가 선택한 셀이 이미 같은 값이면 변경 경계에도 포함하지 않습니다. 연산은 호출 순서대로 즉시 반영하며 행동별 틱·강철·파기 성공 판정을 결정하지 않습니다.

## 오버플로 없는 클리핑

축마다 목적지 시작·소스 시작·교집합 길이를 구합니다. 양수 원점은 `limit-origin`, 음수 원점은 `length-unsigned_abs(origin)`을 검사한 뒤 최소 길이를 선택합니다. `origin+length`, 요청 면적, `-i32::MIN`을 계산하지 않습니다.

따라서 `usize::MAX` 폭/높이의 사각형도 실제 지형과 겹치는 부분만 처리합니다. 소스/목적지 인덱스는 검증된 크기 이내이며 새 좌표 산술은 클리핑 뒤 수행합니다. 변경 수는 목적지 면적 이내, 반환 경계는 지형 내부입니다. 생성 O(면적) 이후 영역 연산은 교집합 면적에 비례하며 추가 메모리는 O(1)입니다.

## 합성·레퍼런스 전수 대조

새 테스트 14개는 Red에서 모두 실패했고 기존 코어 26개·리소스 181개는 통과했습니다. 구현 후 코어 40개·리소스 181개가 통과했습니다. 네 경계·비대칭 마스크·소스 정렬·0 보존·실제 변경 경계·반복/빈 연산·호출 순서·소스/외부 canary·`i32::MIN/MAX`·`usize::MAX`를 검사합니다.

목적지 폭/높이 1~2의 모든 이진 상태 26개, 마스크 폭/높이 1~2의 모든 상태 26개, 사각형 폭/높이 0~3, 원점 각 축 -2~목적지 크기와 추가/삭제를 전수 조합했습니다. **사각형 18944건 + 마스크 30784건 = 49728건**의 전체 버퍼·변경 수·최소 경계를 독립 좌표 집합의 대칭 차와 대조했습니다. 더 큰 비대칭 입력과 극단 값은 별도 fixture에서 검증합니다.

실제 [고정 TS SolidLayer](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/solid-layer.ts#L31)의 유효 좌표 set/clear를 **12628번** 실행했습니다. 하네스가 명시한 선택/클리핑을 적용하고 쓰기 전/후 전체 버퍼에서 변경 수·경계를 측정합니다. TS는 벌크 API·결과 메타데이터를 제공하지 않으며 외부 쓰기 행 별칭은 [#21의 차이](issue-21_terrain_edits.md#레퍼런스와-경계-차이)에 따릅니다. TS의 무수정 영역/행동 로직과 동일하다고 판정하지 않습니다.

목적지 폭→높이→패턴→Y→X 순서에서 사각형 폭→높이→false/true, 이어 마스크 폭→높이→패턴→false/true 순서입니다. 각 결과의 변경 수·경계 존재 0/1·경계 4값(있는 경우)·전체 행 우선 바이트를 연결한 FNV-1a 64비트 서명은 **`f4207a057fb5986b`**이며 Rust와 TS 기반 비교 스트림이 같습니다. 이번 코퍼스의 값은 모두 한 바이트 이내입니다. 이 서명은 합성 비교용이며 실제 게임 상태 인증을 뜻하지 않습니다.

기존 소스 46개와 [매니페스트](../../assets/manifests/reference_sources.json)의 SolidLayer 1290바이트·SHA-256 `cf1cd1a45d2bb6fd7fb266ad69bfb0e7787ecde397b9990ecc830660c8c655e5`·git blob `ed2f4ea51f2dff3f5b39249ac84c71992f031af9`를 확인했습니다. 원문을 수정 없이 Node 24.11.0의 `--experimental-transform-types`로 실행했으며 npm 의존성은 없습니다.

## 재현 명령

루트의 `make check`는 원본·Node 없이 단위/문서 테스트·포맷·Clippy·네이티브 빌드·WASM 컴파일을 검사합니다. 전수 영역 연산 서명은 다음으로 확인합니다.

```sh
cargo test --locked --offline -p lemmings-core terrain::bulk::tests::exhaustive_region_edits_match_coordinate_sets_and_pinned_ts_stream -- --exact --nocapture
```

실제 TS 대조에는 Node 24.11.0과 [확보 도구](../../assets/README.md#레퍼런스-소스)로 준비한 고정 소스만 필요합니다. 원본 게임 파일은 사용하지 않고 임시 출력은 제거합니다.

```sh
python3 tools/reference_assets.py verify --manifest assets/manifests/reference_sources.json
python3 - <<'PY'
from pathlib import Path
import hashlib, json, subprocess, tempfile
source = Path('assets/local/reference-sources/lemmings-ts__src__game__resources__solid-layer.ts')
assert hashlib.sha256(source.read_bytes()).hexdigest() == 'cf1cd1a45d2bb6fd7fb266ad69bfb0e7787ecde397b9990ecc830660c8c655e5'
harness = r'''
let operations=0,rectangles=0,masked=0,validWrites=0,hash=0xcbf29ce484222325n;
const mix=v=>{hash=BigInt.asUintN(64,(hash^BigInt(v))*0x100000001b3n);};
function check(dw,dh,pattern,x,y,sw,sh,bits,solid,isMask) {
    const cells=new Int8Array(Array.from({length:dw*dh},(_,i)=>(pattern>>i)&1)),before=Array.from(cells);
    const expected=new Set();
    for(let j=0;j<dh;j++)for(let i=0;i<dw;i++)if(before[j*dw+i])expected.add(`${i},${j}`);
    const layer=new SolidLayer(dw,dh,cells);
    for(let sy=0;sy<sh;sy++)for(let sx=0;sx<sw;sx++) {
        if(isMask&&!(bits&(1<<(sy*sw+sx))))continue;
        const dx=x+sx,dy=y+sy;
        if(dx<0||dx>=dw||dy<0||dy>=dh)continue;
        if(solid){layer.setGroundAt(dx,dy);expected.add(`${dx},${dy}`);}
        else{layer.clearGroundAt(dx,dy);expected.delete(`${dx},${dy}`);}
        validWrites++;
    }
    const changed=[];
    for(let j=0;j<dh;j++)for(let i=0;i<dw;i++) {
        const k=j*dw+i;
        if(cells[k]!==Number(expected.has(`${i},${j}`)))throw new Error('set mismatch');
        if(cells[k]!==before[k])changed.push([i,j]);
    }
    mix(changed.length);mix(Number(changed.length>0));
    if(changed.length) {
        const xs=changed.map(p=>p[0]),ys=changed.map(p=>p[1]);
        for(const v of [Math.min(...xs),Math.min(...ys),Math.max(...xs)-Math.min(...xs)+1,Math.max(...ys)-Math.min(...ys)+1])mix(v);
    }
    for(const v of cells)mix(v);
    operations++;if(isMask)masked++;else rectangles++;
}
for(let dw=1;dw<=2;dw++)for(let dh=1;dh<=2;dh++)for(let pattern=0;pattern<2**(dw*dh);pattern++)for(let y=-2;y<=dh;y++)for(let x=-2;x<=dw;x++) {
    for(let rw=0;rw<=3;rw++)for(let rh=0;rh<=3;rh++)for(const solid of [false,true])check(dw,dh,pattern,x,y,rw,rh,0,solid,false);
    for(let sw=1;sw<=2;sw++)for(let sh=1;sh<=2;sh++)for(let bits=0;bits<2**(sw*sh);bits++)for(const solid of [false,true])check(dw,dh,pattern,x,y,sw,sh,bits,solid,true);
}
console.log(JSON.stringify({operations,rectangles,masked,validWrites,fnv1a64:hash.toString(16).padStart(16,'0')}));
'''
with tempfile.TemporaryDirectory(prefix='issue-22-reference-') as temporary:
    target = Path(temporary) / 'check.ts'
    target.write_text(source.read_text() + harness)
    result = subprocess.run(['node', '--experimental-transform-types', str(target)],
                            check=True, capture_output=True, text=True)
    actual = json.loads(result.stdout)
    assert actual == {'operations': 49728, 'rectangles': 18944, 'masked': 30784,
                      'validWrites': 12628, 'fnv1a64': 'f4207a057fb5986b'}
    print('PASS:', actual)
PY
```

## G04 완료 기준과 다음 책임

로컬 `make check`는 Rust 단위 221개·문서 테스트 20개(compile-fail 2개 포함)·Python 8개와 WASM 컴파일을 통과했습니다. 위 두 재현 명령 블록도 그대로 실행해 같은 서명을 확인했습니다. 게시·실제 CI를 확인합니다.

G04-01 읽기/경계·G04-02 셀 쓰기·G04-03 영역/마스크 변형으로 G04의 합성 지형 경계·마스크·변형 픽셀 검증 증거를 갖췄습니다. 게시·실제 CI 확인 후 G04 전체를 완료로 반영합니다. COMP-03 전체 통합·강철/행동 규칙·리소스 연결·게임 경계/생존·실제 DOS/WASM 실행은 후속 목표 책임입니다. 다음은 별도 이슈의 **G05-01 결정론적 고정 틱·입력 순서 골격**입니다.

- [결과 보고서](../reports/issue-22_terrain_bulk.md), [진행 기록](../working/issue-22_terrain_bulk.md), [작업일지](../orders/20261018.md#issue-22)
