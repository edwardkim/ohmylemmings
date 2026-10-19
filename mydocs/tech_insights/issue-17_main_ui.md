# G03-08 MAIN.DAT UI·폰트

작업 이슈: [GitHub #17](https://github.com/edwardkim/ohmylemmings/issues/17). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 지원 형식과 근거

원작 VGA MAIN.DAT의 구간 2·3·4·6에서 고정 레퍼런스가 사용하는 19개 묶음·200개 이미지/글리프를 해석합니다. 각 이미지는 완전한 행 우선 비트플레인 1~4개이며 MSB가 왼쪽, 플레인 0이 색상 비트 0입니다. 632폭 로고까지 지원합니다. 팔레트·문자열 배치·화면 조립·입력·HUD 동작은 별도입니다.

| 근거 | 확인한 내용 |
|---|---|
| [Lemmix GameScreen.Menu](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/GameScreen.Menu.pas#L242) | 로고·버튼·아이콘·스크롤 각 16프레임·평가판 4개·릴 주소/치수 |
| [Lemmix GameScreen.Base](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/GameScreen.Base.pas#L321), [Dos.MainDat](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Dos.MainDat.pas#L89) | 배경과 메뉴 글꼴 94개, ASCII 순서·글꼴 시작 주소 |
| [Lemmix Game.SkillPanel](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.SkillPanel.pas#L408) | 구간 6 패널·38개 글꼴, 구간 2의 우/좌 숫자 교차 저장 |
| [TS MainImageSprites](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/main-image-sprites.ts), [SkillPanelSprites](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/skill-panel-sprites.ts) | 같은 이미지 주소/치수, 배경·패널·패널 글자의 불투명 0, 나머지 이미지의 투명 0 |

레퍼런스 버전은 Lemmix `40e9bc34451f0e9127fd53b290a3877e700d143d`, TS `10c7c8b9167258503d02589cf048e2d83a43dd0a`입니다. 기존 소스·원본 매니페스트와 추가 고정 소스 식별값으로 입력을 확인합니다. 일부 Pascal 소스는 CP1252로 읽으며 원문 바이트는 그대로 유지합니다.

| 구간 | 해제 바이트 | 지원 목록 | 이미지 수 |
|---|---:|---|---:|
| 2 | 8384 | 0x1900부터 숫자 오른쪽·왼쪽 각 10개, 8×8·1플레인·묶음별 stride 16 | 20 |
| 3 | 61968 | 배경 320×104·2플레인, 로고 632×94, 버튼 6개 120×61, 아이콘 2개 64×31 | 10 |
| 4 | 36080 | 좌/우 스크롤 각 16개 48×16, 릴 16×16, 평가판 4개 72×27, 메뉴 글꼴 94개 16×16·3플레인 | 131 |
| 6 | 8224 | 스킬 패널 320×40·4플레인, 패널 글꼴 38개 8×16·3플레인 | 39 |
| 합계 | 114656 | 19개 묶음·216192픽셀 | 200 |

기본 정책은 마스크 없이 색상 0을 투명하게 처리합니다. 배경·스킬 패널·패널 글꼴은 **0도 불투명**합니다. RGB가 검정인지와는 무관합니다. 인덱스 0~15를 재배치하지 않고 `Menu`·`Level`·`Monochrome` 팔레트 역할만 제공합니다. 메뉴의 고정 16색, 레벨 팔레트 조립·확대, 숫자의 검정/흰색 RGB 선택은 호출자의 책임입니다.

평가판의 디스크 순서는 Mayhem·Taxing·Tricky·Fun입니다. Lemmix의 `PaintCurrentSection`은 표시 난이도에 맞춰 역순을 선택합니다. API는 원래 순서를 유지하며 게임 난이도 ID로 재배치하지 않습니다.

## 레퍼런스 차이와 미해석 범위

TS의 UI 조회 함수는 스크롤의 첫 프레임만 반환하고 평가판·메뉴 글꼴 읽기를 제공하지 않습니다. 이 부분은 Lemmix의 전체 레이아웃을 지원합니다. TS가 생성하는 빈칸 글리프와 흰색 9×8 빈 숫자 영역은 저장 이미지가 아니므로 반환하지 않습니다. 공백 처리·문자열 간격·숫자 지우기는 후속 UI 조립 단계입니다.

지원 정책 내 픽셀 차이는 발견하지 않았습니다. Lemmix 팔레트의 RGB 0과 실제 비트 인덱스를 구분하며, 최종 화면 합성이나 실제 DOS 실행 호환성까지 판정하지 않습니다. Oh No!/Holiday/CGA/Tandy의 다른 레이아웃도 지원 범위가 아닙니다.

구간 3·6은 전체 바이트를 해석합니다. 구간 2의 `[0,6400)`, `[6560,8384)`와 구간 4의 `[0,10752)`는 고정 레퍼런스의 해당 읽기 경로에서 사용하지 않으므로 의미를 추정하지 않고 보존합니다. `as_bytes()`로 전체 원문을 읽을 수 있습니다. 전체 MAIN.DAT의 구간 1·5도 이번 형식 해석 범위 밖입니다.

## API와 오류 계약

[ui.rs](../../crates/lemmings-resources/src/ui.rs)는 `no_std`·무할당·I/O 없는 API이며 기존 [MainDatFile](issue-16_lemming_sprites.md)을 재사용합니다.

- `MainUiSection::ALL`: 구간 2/3/4/6의 태그·고정 해제 길이입니다. 태그는 파일 출처를 인증하지 않습니다.
- `MainDatFile::decode_ui_into`: 선택 구간의 선언 길이와 정확한 호출자 출력 길이를 쓰기 전에 검사합니다. 이 오류에서는 출력이 보존됩니다. 압축 명령 오류는 부분 출력을 만들 수 있으므로 출력을 버립니다. 반환 뷰는 출력만 빌립니다.
- `MainUiRecord::parse`: 정확한 해제 길이를 검사하고 전체 원문을 빌립니다. `images(kind)`는 묶음의 구간 태그를 확인합니다. `MainUiImageKind::info()`와 `MAIN_UI_IMAGE_KINDS`로 전체 메타데이터를 조회합니다.
- `UiImages::image(index)`: 원래 순서·stride로 이미지를 읽습니다. 범위 검사 후 주소를 계산하며 순환 재생·복제·반전을 하지 않습니다.
- `menu_glyph(char)`: 저장된 ASCII `!`~`~` 94개를 선택합니다. 공백·제어 문자·비 ASCII는 오류입니다.
- `panel_glyph(char)`: `%0123456789-ABCDEFGHIJKLMNOPQRSTUVWXYZ` 순서입니다. ASCII 소문자만 대문자에 연결하며 다른 문자를 대체하지 않습니다.
- `UiImage::pixel`·`decode_into`: 기존 `IndexedPixel`을 반환합니다. 좌표는 곱셈 전에 검사하고, 출력 길이 오류는 모든 픽셀을 보존합니다.

오류는 구간·종류·글자·인덱스·기대/실제 크기를 명시합니다. 압축 오류는 `Error::source`를 연결합니다. 정확한 크기의 임의 플레인 바이트도 이미지로는 유효하므로 원본 식별은 별도 고정 해시로 수행합니다.

## 전체 원본 대조

독립 Python은 원본 MAIN.DAT를 역방향 비트로 직접 해제하고, 모든 7개 구간이 이전 G02/#16의 바이트·해시와 일치함을 확인했습니다. 고정 Lemmix 소스에서 주소·치수·프레임 수를 읽고 전체 플레인 이진 문자열로 픽셀을 구성했습니다. 모든 200개 이미지·216192개 색상/점유 픽셀·글리프 코드·메타데이터가 Rust와 일치합니다. 불투명 픽셀은 138379개입니다.

실제 고정 TS의 DAT 해제·PaletteImage·Frame·ColorPalette도 실행했습니다. 72개는 원본 MainImageSprites/SkillPanelSprites 조회 함수로, 나머지 128개는 Lemmix 레이아웃과 실제 TS 비트플레인/Frame 코드로 확인했습니다. 메뉴는 고정 메뉴 팔레트, 패널은 색마다 다른 진단 팔레트, 숫자는 원본 함수의 흰색 팔레트를 사용해 RGB·알파·점유를 전수 대조했습니다. import/로그 연결만 조정하고 픽셀 알고리즘은 수정하지 않았습니다. 생성된 공백·빈 숫자 영역은 저장 데이터 대조에서 제외했습니다.

[UI 검증 매니페스트](../../assets/manifests/main_ui.json)에 4개 구간·200개 이미지 해시, 묶음 메타데이터·미해석 범위, 입력 해시·추가 소스 12개와 차이를 기록했습니다. 기존 입력 매니페스트는 변경하지 않으며 새 파일은 확보 도구의 `fetch --manifest` 입력이 아닙니다.

## 원본 검사 재현

저장소 루트에서 원본을 [확보](../../assets/README.md)한 뒤 실행합니다. `inspect_ui MAIN_DAT_FILE OUTPUT_DIRECTORY`는 네 구간을 모두 해제·검증한 뒤 새 디렉터리에 구간 원문 4개와 픽셀 맵 200개를 기록합니다. JSONL은 묶음·인덱스·구간·주소·치수·플레인·정책·글리프 코드·점유 수를 담습니다. 출력은 행 우선 한 바이트로 불투명 인덱스 0~15 또는 투명 0x80이며 기존 파일을 덮어쓰지 않습니다.

아래 명령은 임시 디렉터리로 반복 실행할 수 있습니다. 현재 구현을 고정 기대값과 대조하며 독립 Python/TS 알고리즘을 다시 실행하는 명령은 아닙니다.

```sh
python3 tools/reference_assets.py verify
python3 - <<'PY'
from pathlib import Path
import hashlib, json, subprocess, tempfile
sha = lambda data: hashlib.sha256(data).hexdigest()
manifest = json.loads(Path('assets/manifests/main_ui.json').read_text())
for item in manifest['inputs']:
    assert sha(Path(item['path']).read_bytes()) == item['sha256'], item['path']
def run(example, destination):
    return subprocess.run([
        'cargo', 'run', '--quiet', '--locked', '--offline', '-p', 'lemmings-resources',
        '--example', example, '--', 'assets/local/dos-original/MAIN.DAT', str(destination)
    ], check=True, capture_output=True, text=True).stdout
with tempfile.TemporaryDirectory(prefix='issue-17-reproduce-', dir='assets/local') as folder:
    root = Path(folder)
    rows = [json.loads(s) for s in run('inspect_ui', root/'ui').splitlines()]
    run('unpack_dat', root/'sections')
    assert len(rows) == 200 and len(list((root/'ui').iterdir())) == 204
    previous = json.loads(Path('assets/manifests/lemming_sprites.json').read_text())
    for target in previous['sections']:
        data = (root/'sections'/f"part-{target['part']:03}.bin").read_bytes()
        assert len(data) == target['size'] and sha(data) == target['sha256']
    for target in manifest['sections']:
        data = (root/'ui'/f"section-{target['part']:03}.bin").read_bytes()
        assert len(data) == target['size'] and sha(data) == target['sha256']
        assert data == (root/'sections'/f"part-{target['part']:03}.bin").read_bytes()
    expected = {(r['kind'],r['index']): r for r in manifest['images']}
    assert len(expected) == 200
    for row in rows:
        target = expected.pop((row['kind'],row['index']))
        assert row == {key: target[key] for key in row}
        pixels = (root/'ui'/row['pixel_file']).read_bytes()
        assert len(pixels) == row['width'] * row['height']
        assert set(pixels) <= {*range(16),128}
        assert len(pixels) - pixels.count(128) == row['opaque_pixels']
        assert sha(pixels) == target['pixel_sha256']
    assert not expected and sum(r['width']*r['height'] for r in rows) == 216192
print('PASS: seven section hashes, 19 families/200 images/216192 pixels, metadata and input hashes')
PY
```

## 테스트와 미리보기

합성 테스트 16개는 전체 4개 구간의 짧은 접두부·추가 바이트, 구간 태그, 19개 레이아웃·200개 이미지, 632폭 로고·전체 16프레임 스크롤·좌우 숫자 stride·문자 범위, 2/3/4플레인·투명도·큰 좌표·출력 보존·압축 오류 source·출력만 빌리는 수명을 확인합니다.

CLI 오류 16개에서 인자·파일·헤더·체크섬·구간 수·4개 구간 각각의 선언 크기와 압축 명령·덮어쓰기 거부를 확인했습니다. 잘못된 입력은 출력 디렉터리를 만들지 않으며 원본과 기존 출력을 보존합니다.

로컬 `assets/local/issue-17-ui/preview.png`는 메뉴·글리프·패널과 스크롤의 첫 4프레임을 표시합니다. 패널은 GROUND0 Standard+Preview 진단 팔레트이며 최종 게임 화면의 색상 검증은 아닙니다. 이미지·바이너리·JSONL·참조 소스 사본은 Git 추적에서 제외합니다.

- [결과 보고서](../reports/issue-17_main_ui.md), [진행 기록](../working/issue-17_main_ui.md), [작업일지](../orders/20261018.md#issue-17)

## #56 VGA 통합에서 추가 확인

위 기록은 #17 당시의19묶음200이미지입니다. [#56](issue-56_skill_panel.md)에서 실제 DOS VGA의 구간2 패널/폰트를 별도로 식별해39이미지를 추가했습니다. 구간6의 기존 출력은 유지하며 현재 `inspect_ui`는21묶음239이미지를 출력합니다. 위 과거 재현 스크립트의200개 수량 검사는 당시 범위에 한정되므로 현재 전체 출력 수량에는 사용하지 않습니다.
