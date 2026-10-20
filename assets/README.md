# 원본 검증 리소스

DOS 원작 VGA 데이터 프로필의 출처와 파일 식별값을 [매니페스트](manifests/dos_original.json)에 고정합니다. 원본 파일은 `assets/local/dos-original/`에 확보하며 이 폴더는 Git 추적에서 제외됩니다.

## 확보와 검증

Python 3의 표준 라이브러리만 사용합니다. 저장소 루트에서 실행합니다.

```sh
python3 tools/reference_assets.py fetch
python3 tools/reference_assets.py verify
python3 -m unittest discover -s tools/tests -v
```

`fetch`는 고정 커밋의 소스를 내려받아 크기와 SHA-256을 확인한 뒤 파일을 씁니다. `verify`는 로컬 파일 50개를 매니페스트와 비교합니다. 엔진용 리소스 27개와 원작 실행·참고용 파일 23개로 구분됩니다.

별도 폴더에서 재현하려면 `fetch --destination assets/local/reproduced-dos-original`을 사용합니다.

## 레퍼런스 소스

[소스 매니페스트](manifests/reference_sources.json)는 Lemmix·Lemmings.ts의 고정 파일과 선언 위치를 기록합니다. 같은 도구에서 `--manifest assets/manifests/reference_sources.json`을 지정해 `fetch` 또는 `verify`를 실행합니다. 소스는 `assets/local/reference-sources/`에 저장합니다.

파일·함수의 역할과 후속 구현에서 읽을 순서는 [레퍼런스 코드 지도](../mydocs/tech_insights/issue-3_reference_code_map.md)를 참고합니다.

## 호환성 비교 목록

[호환성 매니페스트](manifests/dos_compatibility.json)는 120개 레벨의 DAT/ODD 연결·초기 조건, 해제 구간 81개의 크기·SHA-256, 음악 21개·효과음 18개의 주소를 기록합니다. `reference_assets.py`의 확보 매니페스트와는 용도가 다르므로 `fetch --manifest` 입력으로 사용하지 않습니다.

[비교 기준](../mydocs/tech_insights/issue-5_compatibility_baseline.md)에 따라 G02~G11에서 기대값과 실제 동작을 대조합니다. 현재 목록은 바이트·정적 구조 검증 결과이며 실행 호환성 통과 표가 아닙니다.

Rust 해제 예제와 기존 81개 구간의 해시 대조는 [G02-02 재현 명령](../mydocs/tech_insights/issue-9_dat_decompression.md#개발용-해제와-고정-해시-재현)을 따릅니다. 해제 결과도 `assets/local/` 안에 저장하며 기존 결과는 덮어쓰지 않습니다.

DAT에서 기본 레벨을 직접 해제·검사하는 `inspect_levels` 예제와 메타데이터 비교는 [G03-01 안내](../mydocs/tech_insights/issue-10_level_records.md#원본-검사-재현)를 따릅니다. ODDTABLE을 적용한 120개 진입점은 [G03-02 검사](../mydocs/tech_insights/issue-11_level_catalog.md#원본-검사-재현)의 `inspect_catalog`로 확인합니다. 기본 바이트를 포함하는 출력도 `assets/local/`에 둡니다.

원시 GROUND 파일의 모든 슬롯·EGA/VGA 뱅크·두 RGB 확대 결과는 [G03-03 검사](../mydocs/tech_insights/issue-12_ground_metadata.md#원본-검사-재현)의 `inspect_ground`로 확인합니다. GROUND는 압축 DAT 구간으로 읽지 않습니다.

VGAGR 지형·객체·미리보기 전체 픽셀은 [G03-04 재현 명령](../mydocs/tech_insights/issue-13_vgagr_images.md#원본-대조와-재현)의 `inspect_vgagr`로 검사합니다. [이미지 검증 매니페스트](manifests/vgagr_images.json)는 5세트의 픽셀 연결 해시·10개 해제 구간 해시·추가 고정 소스 식별값을 기록합니다. 확보용 `fetch --manifest` 입력이 아니며 RGB 렌더링·실행 호환성 결과도 아닙니다.

VGASPEC 특수 배경 4개는 [G03-05 재현 명령](../mydocs/tech_insights/issue-14_vgaspec_backgrounds.md#원본-검사-재현)의 `inspect_vgaspec`로 검사합니다. [특수 배경 검증 매니페스트](manifests/vgaspec_backgrounds.json)에 팔레트·내부/RLE 해제/픽셀 해시를 고정합니다. 이것도 확보용 `fetch --manifest` 입력이 아니며 전체 레벨 합성·RGB/알파 렌더링 결과를 뜻하지 않습니다.

초기 레벨 지형 120개는 [G03-06 재현 명령](../mydocs/tech_insights/issue-15_level_terrain.md#원본-검사-재현)의 `inspect_terrain`으로 검사합니다. [지형 검증 매니페스트](manifests/level_terrain.json)에 80개 기본 맵의 인덱스·정규화 RGBA 해시·팔레트·점유 수와 TS 우선순위 차이를 기록합니다. 원본 확인 후 새 출력 디렉터리를 사용하며 결과는 Git 추적에서 제외합니다. 확보용 입력이나 실행 호환성 인증은 아닙니다.

MAIN.DAT 레밍 스프라이트는 [G03-07 재현 명령](../mydocs/tech_insights/issue-16_lemming_sprites.md#원본-검사-재현)의 `inspect_sprites`로 확인합니다. [스프라이트 검증 매니페스트](manifests/lemming_sprites.json)에 28개 애니메이션·337프레임의 픽셀 해시·발 기준점·방향과 7개 해제 구간의 해시를 기록합니다. 팔레트·재생 규칙·UI/마스크 해석은 별도이며 확보용 입력으로 사용하지 않습니다.

MAIN.DAT UI·폰트는 [G03-08 재현 명령](../mydocs/tech_insights/issue-17_main_ui.md#원본-검사-재현)의 `inspect_ui`로 확인합니다. [UI 검증 매니페스트](manifests/main_ui.json)에 4개 구간·19개 묶음·200개 이미지/글리프 해시와 미해석 범위를 기록합니다. 원래 인덱스와 묶음별 투명도 정책을 검증하며 화면·문자열 조립·최종 레벨 팔레트 인증이나 확보용 입력은 아닙니다.

MAIN.DAT 지형 마스크·카운트다운은 [G03-09 재현 명령](../mydocs/tech_insights/issue-18_main_masks.md#원본-검사-재현)의 `inspect_masks`로 확인합니다. [마스크 검증 매니페스트](manifests/main_masks.json)에 구간 1·전체 23프레임의 0/1 해시·숫자 값·선택 수와 TS 조회 차이를 기록합니다. 비트 1은 선택됨을 뜻하며 위치·타이밍·지형 적용을 인증하거나 확보용 입력으로 사용하지 않습니다.

레벨 객체 정적 합성은 [G03-10 재현 명령](../mydocs/tech_insights/issue-19_level_objects.md#원본-검사-재현)의 `inspect_objects`로 검사합니다. [객체 합성 매니페스트](manifests/level_objects.json)에 미리보기·마지막 프레임 160개 기본 결과의 색상/출처·RGBA 해시·두 팔레트와 TS 차이를 기록합니다. `initial`도 지원하며 게임 틱·충돌·실제 실행 인증이나 확보용 입력은 아닙니다.

## 기준판 식별

기준 ID는 `dos-original-vga`입니다. 원본 매체의 세부 리비전은 아직 확인되지 않았으며 확보한 파일의 바이트 식별값으로 입력을 고정합니다. 확보 출처, 상호 대조 결과, 데모와의 차이는 [기준판 분석](../mydocs/tech_insights/issue-2_dos_original_baseline.md)에 기록합니다.

## 실제 DOS 영상 증거

[#28 재현 절차](../mydocs/tech_insights/issue-28_dos_comparison.md)와 [영상 관측 manifest](manifests/dos_observation.json)에 DOSBox·원본 EXE·설정/입력·세 실행·비교 그림 해시를 기록합니다. 별도 생성 런타임 seed 조건이며 키 디스크 원본 매체를 확보한 것은 아닙니다. 원본 50개와 EXE는 변경하지 않습니다. 상부 일곱 장면 일치와 전체 영역/속도 차이를 구분하며 전체 호환성 인증이 아닙니다. 영상·실행 복사본·파생 이미지는 assets/local/에만 보관합니다.

[#30 교정 manifest](manifests/dos_calibration.json)는 연속 객체 위상·타이머 경로·정수 pacing 보정, 기존 DOS 세 영상의 전체 장면 일치와 8개 추가 실행을 기록합니다. GUI 5000 cycles의 1영상 프레임 지연은 차이로 보존합니다. 제한된 읽기 전용 카운터 폴링·정적 코드 범위 해시는 전체 DOS 상태 인증이 아니며 파생 데이터는 계속 assets/local/에만 둡니다.

- [Digger 원본/기준/네이티브 증거](manifests/digger.json): #31의18개 기준 함수·합성/원본 상태·전체 삭제 목록·실제 마우스·DOS 두 실행·전체 세 장면 비교의 hash-only 기록입니다. 원본 파생 영상·화면·메모리 숫자 관측·전체 추적/바이너리는 ignored assets/local에만 둡니다.

- [Basher 원본/기준/네이티브 증거](manifests/basher.json): #32의21개 기준 함수·합성/원본 상태·전체 삭제·실제 B키/마우스·DOS 두 실행/전체 세 장면 비교의 hash-only 기록입니다. 원본 파생물은 ignored assets/local에만 둡니다.

- [Miner 원본/기준/네이티브 증거](manifests/miner.json): #33의24개 기준 함수·합성/원본 상태·전체 삭제·실제 M키/마우스·DOS 두 실행과 전체 세 장면 비교의 hash-only 기록입니다. 원본 파생물은 ignored assets/local에만 둡니다.

- [Builder 원본/기준/네이티브 증거](manifests/builder.json): #34의28개 기준 함수·합성/원본 상태·경계별 지형 추가/삭제·실제 U키/마우스·DOS 두 실행과 전체 세 장면 비교의 hash-only 기록입니다. 원본 파생물은 ignored assets/local에만 둡니다.

- [Blocker 검증 매니페스트](manifests/blocker.json): #36의37개 기준 함수·합성/원본 객체 맵/캐시/지형·실제 K키/마우스·DOS 두 실행의 전체 세 장면과 검증 자료의 출처/해시를 기록합니다. 확보용 fetch 입력이 아닙니다.

- [Bomber 검증 매니페스트](manifests/bomber.json): #38의43개 기준 함수·카운트다운/폭발/입자·Blocker 해제·원본 네이티브·DOS 두 실행의 전체 여덟 장면과 출처/해시를 기록합니다. 입자 표는 기존 원본 EXE에서 읽으며 원본 파생물은 ignored `assets/local/`에만 둡니다. 확보용 fetch 입력이 아닙니다.

- [Climber 검증 매니페스트](manifests/climber.json): #40의46개 기준 함수·영구 능력/Climbing/Hoisting·원본 네이티브·DOS 두 실행과 기존 여섯 직업 회귀의 출처/해시를 기록합니다. 원본 파생물은 ignored `assets/local/`에만 두며 확보용 fetch 입력이 아닙니다.

- [Floater 검증 매니페스트](manifests/floater.json): #42의48개 원문 함수/16단계 표·두 영구 능력·원본 Fun02·독립 DOS 두 실행과 기존 일곱 직업 회귀의 출처/해시를 기록합니다. 원본 파생물은 ignored 경로에 보관하며 fetch 입력이 아닙니다.

- [Nuke 검증 매니페스트](manifests/nuke.json): #44의50개 원문 함수·순차 타이머/생성 중지·별도 DOS 제거 검사 순서·원본 Fun06·DOS 반복/기존8직업 회귀의 출처·해시를 기록합니다. 원본 파생물은 ignored 경로에 보존하며 fetch 입력이 아닙니다.

- [직업 통합 회귀 매니페스트](manifests/skill_interactions.json): #46의2640할당 행렬·14혼합 사례28재생·50개 기준 함수와 합성 추적/캡처 출처·해시를 기록합니다. 새 DOS 관측은 없으며 fetch 입력이 아닙니다.

- [출구·구출 검증 매니페스트](manifests/exit_rescue.json): #48의51개 기준 함수·합성9사례/18재생·원본 Fun01 구출·새 DOS read-only 관측과 출처·해시를 기록합니다. 확보용 fetch 입력이 아닙니다.

- [위험물·함정 검증 매니페스트](manifests/hazards.json): #50의54개 기준 함수·합성14사례/28재생·원본 세 레벨·새 DOS 물/불/함정 관측과 전체 게임 장면의 출처·해시를 기록합니다. 원본 파생물은 ignored 경로에 두며 fetch 입력이 아닙니다.

[#52 시간·구출률·승패](../mydocs/reports/issue-52_level_outcome.md)의 기준/합성/원본/DOS 결과·출처/해시는 [completion.json](manifests/completion.json)에 기록합니다. 원본 결과 화면·영상은 ignored `assets/local/`에만 보관합니다.

[#56 기술 패널·미니맵](../mydocs/tech_insights/issue-56_skill_panel.md)은 MAIN 구간2의 VGA 패널/폰트39이미지와 전용 UI 팔레트를 추가로 식별합니다. [출처·검증 해시](manifests/skill_panel.json)에 기존200이미지 보존과 원본/합성 검증 범위를 기록합니다.

[#58 포인터 선택·RightClickGlitch](../mydocs/tech_insights/issue-58_pointer_selection.md)의 원문53함수/표·48494개 합성 비교·SDL 입력/재생 출처는 [pointer_selection.json](manifests/pointer_selection.json)에 기록합니다. 새 DOS 실행 증거로 집계하지 않으며 원본 파생 자료는 ignored 경로에만 보관합니다.

[#64 자산 바이트·공통 장면 분리](../mydocs/reports/issue-64_shared_game.md)의 추출 전후 네이티브 재생·원본 파일/메모리 공급 비교는 [shared_game.json](manifests/shared_game.json)에 출처와 해시를 기록합니다. 원본 파생물은 ignored 경로에만 보관합니다.

[#66 JS/WASM·합성 브라우저](../mydocs/reports/issue-66_wasm_browser.md)의 공식 CLI 출처/고정 해시는 [web_tools.json](manifests/web_tools.json), 소스·WASM/JS·실제 브라우저 검증 증거는 [web_browser.json](manifests/web_browser.json)에 기록합니다. 다운로드·생성물은 ignored assets/local에 둡니다.

- [브라우저 입력·자산·전환 검증](manifests/web_input_assets.json): #68의 원본 없는24사례와 선택 원본 Fun01..03 초기 실행/입력/재시작27사례, SDL 대조·CDP19222 실제 조작·출처/해시를 기록합니다. 원본 바이트/추적/화면은 ignored assets/local에만 보관합니다. 원본 한 레벨의 브라우저 통합 완주는 다음 #70 증거로 구분합니다.

- [브라우저 통합 완주 검증](manifests/web_playthrough.json): #70의 합성/원본 네 시도, 매 제어 경계 전체 상태/맵 변형·Canvas, raw 이벤트와 독립 SDL 재생·시간/렌더 분할의 출처/해시입니다. 원본 Fun01 두 번 전원 구출·조기 Nuke 실패·초기화와 Fun02 새 ID0을 검증했고 기존 #60 DOS 증거와 구분합니다. 원본 파생물은 ignored assets/local에만 둡니다.

- [AdLib 자원 목록 검증](manifests/adlib_catalog.json): #72의 합성21곡48채널·원본21곡69채널/18효과음·120레벨 대응, 실제 고정 TS 초기화 함수/기존 목록 대조와 출처/해시입니다. OPL/PCM/DOS 오디오 실행 근거가 아닙니다. 선택 원본 검사는 `python3 tools/check_adlib.py --original-assets assets/local/dos-original --evidence assets/local/adlib-original-check`로 재현하며 Node가 필요합니다. 원본 파생물은 ignored assets/local에만 둡니다.

ADLIB 명령/레지스터 스트림의 합성877호출과 원본 드라이버 직접 실행 검증은 [#74 계약](../mydocs/tech_insights/issue-74_adlib_stream.md)과 [출처/해시](manifests/adlib_stream.json)에 기록합니다. 원본 없는 검사는 `python3 tools/check_adlib_stream.py`이며 `make check-rust`에 포함됩니다. 선택적 원본 비교는 별도 Unicorn2.1.4 환경에서 `python tools/check_adlib_stream.py --original-assets assets/local/dos-original --evidence assets/local/NEW_EMPTY_PATH`로 실행합니다. 실행기/원본·파생물은 ignored assets/local에 보관합니다. audio_step는 드라이버 호출 번호이고 PCM/장치 재생이나 PIT 속도 인증이 아닙니다.

[#76 OPL/PCM 시간 계약](../mydocs/tech_insights/issue-76_opl_pcm.md)과 [출처/해시](manifests/adlib_pcm.json): `python3 tools/check_opl_reference.py`는 고정 LGPL C 소스를 별도 host 프로세스로 빌드해 합성 PCM을 전수 비교합니다(`cc` 필요). `python3 tools/check_audio.py --wasm`는 고정 합성 fixture·native·실제 WASM/두 분할을 비교합니다. 원본 없는 검사는 make check-rust/check-web에 포함됩니다. 선택적 원본 PCM·실제 DOS 관측은 계약의 명령과 새 ignored 경로를 사용합니다. 원본/파생 음원·영상은 배포하지 않습니다. 실제 장치 출력/청취는 후속입니다.

[#78 게임 사운드 계약](../mydocs/tech_insights/issue-78_game_sound.md)·[출처/해시](manifests/game_sound.json): `python3 tools/check_game_sound.py --wasm`는 원본 없는 고정 메타데이터 oracle·합성 게임 경로와 실제 WASM/PCM을 대조합니다. `tools/dos_sound_reference.py`는 선택적 Unicorn2.1.4/검증 원본의 무수정 x86 분기를 실행합니다. 기본 CI에는 원본/Unicorn 의존성을 추가하지 않습니다. 장치 출력은 G10-05입니다.

[#80 출력 계약](../mydocs/tech_insights/issue-80_audio_output.md)·[출처/해시](manifests/audio_output.json): 원본 사운드는 ADLIB.DAT도 공급합니다. 연습 모드는 독립 작성한 음원을 사용합니다. `python3 tools/check_audio_output.py`/`--wasm`는 원본 없는 SDL/실제 Web Audio와 native/WASM PCM을 검사합니다. 선택적 원본/LAN 증거는 새 ignored 경로를 사용합니다. 웹 빌드는 LGPL 전문·대응 소스 묶음/해시를 함께 생성하며 원본/로컬 자료는 제외합니다.

[#89 특수4개 장면](../mydocs/reports/issue-89_special_scenes.md)의 현재 소스 범위·원본4개 SDL/WASM 진입·기존 정적 독립 해시 대조와 합성 UI/오류 보존 증거는 [special_scenes.json](manifests/special_scenes.json)에 기록합니다. 전체120개 완주·성능 인증과 구분하며 원본/파생물은 ignored assets/local에 둡니다.
