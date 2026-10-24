# Rusty Lemmings

2026-10-24 개발 상태: G00~G10과 [배포 준비 #115](mydocs/reports/issue-115_release_preparation.md)를 완료했고, [v0.2.0 2D 리마스터](https://github.com/edwardkim/ohmylemmings/releases/tag/v0.2.0)의 Windows/macOS/Linux/Web 다운로드와 [GitHub Pages 플레이](https://edwardkim.github.io/ohmylemmings/)를 제공합니다. G11 성능 재검증은 더 좋은 호스트의 [#113](https://github.com/edwardkim/ohmylemmings/issues/113)에 이월했습니다. 자체 코드에는 [#129의 MIT 정책](mydocs/reports/issue-129_project_license.md)을 적용했고, [#130](mydocs/reports/issue-130_maintainer_acceptance.md)은 메인테이너의 Windows 11·웹브라우저 직접 테스트 성공 판정을 받았습니다. Windows 지원 대상은 **Windows 11 x64**이며 **Windows 10은 지원하지 않습니다**. [#1은 초기 구현·첫 플레이 배포의 상위 추적을 종료](mydocs/reports/issue-1_initial_development_closure.md)하며, 남은 성능·메모리는 #113에서 계속합니다.

[![CI](https://github.com/edwardkim/ohmylemmings/actions/workflows/ci.yml/badge.svg)](https://github.com/edwardkim/ohmylemmings/actions/workflows/ci.yml)
[![Play on GitHub Pages](https://github.com/edwardkim/ohmylemmings/actions/workflows/pages.yml/badge.svg)](https://edwardkim.github.io/ohmylemmings/)

[2D HD 리마스터 #137](mydocs/plans/issue-137_2d_remaster.md)는 [#187 전체 인수](mydocs/reports/issue-187_hd_release.md)를 거쳐 **v0.2.0**으로 게시했습니다. 캐릭터337·객체540·5재질·특수4장면·13아이콘·HD 글꼴/전체 UI를 포함하고 HD로 시작하며, 원본 화면과 규칙/조작도 유지합니다. Web은 첫 Fun01 HD 약19.8MB를 우선 준비하고 플레이 중 다음 레벨을 받습니다. 실제 용량·진행률·재시도와 캐시 재사용을 제공하며 엔진/게임 데이터는 별도입니다. [BGM #165 지속 재생 수정](mydocs/reports/issue-165_bgm.md)도 포함합니다. [릴리즈 노트](mydocs/releases/issue-191_v0.2.0.md)·[게시/공개 검증](mydocs/reports/issue-194_release_v020_completion.md)을 참고하세요.

프로젝트 자체 작성 코드는 [MIT](LICENSE)로 제공합니다. [제3자 고지 전문](THIRD_PARTY_NOTICES.txt)과 [적용 범위](third_party/README.md)를 함께 확인하세요. OPL 합성기는 LGPL-2.1-or-later를 유지하며, 원본 DOS 그래픽·레벨·음악/효과음·상표에는 프로젝트 MIT 허가가 적용되지 않습니다.

고전 DOS 게임 레밍즈(Lemmings)의 데이터와 게임 동작을 Rust로 재현하는 프로젝트입니다. 렌더링·입출력과 분리된 코어 엔진을 만들고, SDL2 기반 데스크톱 환경과 WebAssembly 기반 웹 환경을 지원하는 것이 목표입니다.

## 현재 상태

G00~G09를 완료했습니다. [G10-01 / #72](mydocs/reports/issue-72_adlib_catalog.md)에서 ADLIB21곡·18효과음의 no_std 자원 해석과 합성/원본 전체 목록 대조, 사운드 통합 계약을 연결합니다. 실제 명령 해석·OPL/PCM·네이티브/웹 오디오 출력은 G10 후속 절편입니다.

G00 기준 정의·G01 개발 기반·G02 DAT 언패킹을 완료했습니다. `no_std` 코어 자료형과 DAT 구간 검증·압축 해제를 도입했습니다. 원작 파일 50개·소스 46개와 120개 레벨·8직업·음악 21개·효과음 18개의 검증 범위를 고정했습니다. [G02-02 압축 해제](mydocs/tech_insights/issue-9_dat_decompression.md)는 합성 테스트, 원본 21개 파일·102개 구간의 해시 대조와 실제 Rust·Python·WASM CI를 통과했습니다.

G03 리소스 해석을 완료했습니다. [기본 레벨 레코드](mydocs/tech_insights/issue-10_level_records.md), [ODDTABLE·120개 카탈로그](mydocs/tech_insights/issue-11_level_catalog.md), [GROUND 메타데이터·팔레트](mydocs/tech_insights/issue-12_ground_metadata.md), [VGAGR 이미지·마스크](mydocs/tech_insights/issue-13_vgagr_images.md)의 원본 대조와 실제 CI를 통과했습니다.

[G03-05 VGASPEC 특수 배경](mydocs/tech_insights/issue-14_vgaspec_backgrounds.md)을 완료했으며 [G03-06 초기 레벨 지형](mydocs/tech_insights/issue-15_level_terrain.md)도 전체 맵 대조·로컬 검사·실제 CI를 통과해 완료했습니다. 120개 진입점·30412800픽셀·팔레트를 독립 Python과 대조했고 실제 TS 렌더링 코드와의 규칙 차이도 기록했습니다. [G03-07 MAIN.DAT 레밍 스프라이트](mydocs/tech_insights/issue-16_lemming_sprites.md)도 337프레임·63488픽셀의 Rust·Python·TS 대조·로컬 검사·실제 CI를 통과해 완료했습니다. [G03-08 MAIN.DAT UI·폰트](mydocs/tech_insights/issue-17_main_ui.md)도 식별된 19개 묶음·200개 이미지/글리프 전체 픽셀 대조·로컬 검사·실제 CI를 통과해 완료했습니다. [G03-09 MAIN.DAT 마스크·카운트다운](mydocs/tech_insights/issue-18_main_masks.md)도 전체 23프레임·3104비트 대조·로컬 검사·실제 CI를 통과해 완료했습니다. G03-01~10의 검증·게시·실제 CI로 G03 전체 완료 기준을 충족했습니다. [G03-10 객체 이미지 배치·합성](mydocs/tech_insights/issue-19_level_objects.md)도 160개 기본 결과·240개 표시 결과의 독립 대조·게시·실제 CI를 통과했습니다. [G04-01 픽셀 지형 읽기·경계](mydocs/tech_insights/issue-20_terrain_mask.md)도 합성 전수 대조·게시·실제 CI를 통과해 완료했습니다. G04의 읽기·셀 쓰기·영역 변형을 모두 검증했으며 [G04-02 지형 쓰기·셀 변경](mydocs/tech_insights/issue-21_terrain_edits.md)도 전수 대조·게시·실제 CI를 통과해 완료했습니다. [G04-03 영역·마스크 기반 지형 추가/삭제](mydocs/tech_insights/issue-22_terrain_bulk.md)도 전수 대조·게시·실제 CI를 통과했습니다. G04 전체 완료 기준을 충족했습니다. [G05-01 고정 틱·입력 순서 골격](mydocs/tech_insights/issue-23_fixed_ticks.md)도 합성 분할/재생·게시·실제 CI를 통과해 완료했습니다. [G05-02 시작 시계·입구·생성 스케줄](mydocs/tech_insights/issue-24_startup_schedule.md)도 합성 경계/재생·게시·실제 CI를 통과해 완료했습니다. [G05-03 개체 이동·제거](mydocs/tech_insights/issue-25_actor_motion.md)도 전수/통합 재생·게시·실제 CI를 통과해 완료했습니다. [#26 상태 추적·재생·기준 실행 대조](mydocs/tech_insights/issue-26_state_replay.md)의 실제 기준 함수 실행·합성/원본 대조·게시·CI를 확인해 G05 전체를 완료했습니다. 이후 G06 화면·실제 DOS 대조/보정도 완료했습니다. 전체 DOS 호환성 판정은 후속 목표에서 수행합니다.

## Rust 개발

Rustup을 설치한 뒤 저장소 루트에서 실행합니다. [rust-toolchain.toml](rust-toolchain.toml)이 Rust 1.93.1과 rustfmt·Clippy를 선택합니다. 두 no_std 라이브러리는 외부 패키지 의존이 없습니다. SDL2 데스크톱 앱의 고정 의존을 최초 fetch한 뒤 오프라인으로 빌드합니다.

전체 개발 검증에는 GNU Make, Python 3.12, SDL2 개발 라이브러리와 WASM 타깃, 웹 검사에 사용할 Chrome/Chromium을 준비합니다. Linux SDK 설치는 [데스크톱 실행 안내](apps/desktop/README.md)를 따릅니다.

```sh
rustup target add wasm32-unknown-unknown
cargo fetch --locked
python3 tools/web_build.py fetch
make check
```

`make check-rust`, `make check-python`, `make check-wasm`, `make check-desktop`, `make check-web`으로 각각 실행할 수도 있습니다. [CI·검증 안내](mydocs/plans/issue-7_continuous_integration.md)와 [Makefile](Makefile)에 세부 명령을 정리합니다. 같은 명령이 [GitHub Actions](.github/workflows/ci.yml)의 push·PR·수동 실행에서 사용됩니다.

- [crates/lemmings-core](crates/lemmings-core/src/lib.rs): 레벨 식별 자료형·0/1 지형 읽기/변형·호출 경계 입력·17호출 게임 시계·시작/생성·기본 개체 이동/제거
- [crates/lemmings-resources](crates/lemmings-resources/src/lib.rs): DAT·기본 레벨·ODDTABLE·120개 카탈로그·GROUND·VGAGR·VGASPEC·초기 지형·객체 합성·MAIN 레밍/UI/폰트·마스크·숫자와 팔레트, 코어 ID를 사용하는 로컬 의존
- [crates/lemmings-session](crates/lemmings-session/README.md): 공유 결정적 게임 세션·호스트 시간 전달
- [crates/lemmings-game](crates/lemmings-game/README.md): 이름 기반 자산 바이트 조립·CPU 화면·패널·장면 전환
- [개발 골격 설계](mydocs/plans/issue-6_rust_workspace.md): 모듈 경계·API·개별/WASM 빌드 명령

- [apps/web](apps/web/README.md): JS/WASM 합성 브라우저 연습장·명시 틱/시간·Canvas 표시, `make web` / `make serve-web`
- [apps/desktop](apps/desktop/README.md): Linux SDL2 창·입력·원본 Fun01 지형/스프라이트·고정 틱 세션·캡처/재생

원본을 준비한 뒤 저장소 루트에서 네이티브 화면을 실행합니다.

```sh
cargo run --release --locked --offline -p lemmings-desktop -- --assets assets/local/dos-original
```

F/C/E/K/U/M/B/D로 Floater/Climber/Bomber/Blocker/Builder/Miner/Basher/Digger를 선택하고 레밍을 클릭합니다. `--level 6`은 Bomber 원본 Fun06을 엽니다. Space로 일시정지/재개, 위/아래 화살표로 생성 속도, 좌/우 화살표로 카메라를 조작하고 Esc로 종료합니다. 출구·구출·승패와 결과 화면의 재시작/다음 레벨 이동을 지원합니다. 음향은 후속입니다. [G06-01 검증 보고서](mydocs/reports/issue-27_native_view.md)에 X11 실행과 화면 근거를 기록합니다. [#28 DOS 세 실행·동일 장면 대조](mydocs/reports/issue-28_dos_comparison.md)를 완료했습니다. [#30 보정](mydocs/reports/issue-30_dos_differences.md)으로 일곱 기준 장면의 전체 320×160 영역이 원본 세 실행과 같고 기본 실행 간격도 DOS 관측에 맞췄습니다. 게시·실제 CI를 확인해 G06 전체를 완료했습니다. G07-01 Digger와 G07-02 Basher를 완료했으며 G07-03 Miner도 완료했으며 [G07-04 Builder / #34](mydocs/reports/issue-34_builder.md)의 병합·CI도 완료했습니다. [G07-05 Blocker / #36](mydocs/reports/issue-36_blocker.md)의 PR #37 병합·main CI·종료를 확인했습니다. [G07-06 Bomber / #38](mydocs/reports/issue-38_bomber.md)의 PR #39 병합·main CI·종료와 사용자 E키 확인을 마쳤습니다. [G07-07 Climber / #40](mydocs/reports/issue-40_climber.md)의 PR #41 병합·main CI·종료를 확인했습니다. [G07-08 Floater / #42](mydocs/reports/issue-42_floater.md)의 PR #43 병합·main CI·종료를 확인했습니다. [G07-09 Nuke / #44](mydocs/reports/issue-44_nuke.md)의 PR #45 병합·main CI·종료를 확인했습니다. [G07-10 / #46](mydocs/reports/issue-46_skill_interactions.md)의 할당/혼합 회귀도 완료해 G07 직업·지형 목표의 기준을 충족했습니다. [G08-01 / #48](mydocs/reports/issue-48_exit_rescue.md)의 출구·구출을 연결했습니다. [G08-02 / #50](mydocs/reports/issue-50_hazards.md)의 물·불·함정도 연결했습니다. [G08-03 / #52](mydocs/reports/issue-52_level_outcome.md)의 시간·구출률·승패 판정을 연결했습니다. [G08-04 / #54](mydocs/reports/issue-54_results_restart.md)에서 결과 화면·동일 레벨 재시작·다음 레벨 전환을 연결했습니다. [#56 원본 기술 패널·미니맵](mydocs/reports/issue-56_skill_panel.md)을 통합했습니다. [#58 포인터 선택·RightClickGlitch](mydocs/reports/issue-58_pointer_selection.md)의 병합·실제 CI를 확인했습니다. [#60 한 레벨 통합 완주](mydocs/reports/issue-60_native_playthrough.md)의 실제 SDL 성공·실패·재시작/전환·독립 재생과 DOS 결과/장면 비교를 검증했습니다. 정확한 게시 CI·G08 완료 처리는 #60에서 추적하며 다음은 G09 웹의 동일 코어 경계입니다.

## 원본 데이터 준비

저장소 루트에서 Python 3으로 실행합니다.

```sh
python3 tools/reference_assets.py fetch
python3 tools/reference_assets.py verify
python3 -m unittest discover -s tools/tests -v
```

원본 파일은 `assets/local/dos-original/`에 저장됩니다. [확보 안내](assets/README.md)와 [기준판 분석](mydocs/tech_insights/issue-2_dos_original_baseline.md)에서 출처와 파일 식별 기준을 확인할 수 있습니다.

후속 구현에서 참고할 파일·함수와 차이는 [레퍼런스 코드 지도](mydocs/tech_insights/issue-3_reference_code_map.md)에 정리합니다.

[호환성 검증 기준](mydocs/tech_insights/issue-5_compatibility_baseline.md)과 [전체 레벨 목록](mydocs/tech_insights/issue-5_level_inventory.md)은 후속 구현의 비교 입력·판정 방법을 정의합니다.

## 구현 목표

- 원본 DAT/VGASPEC 리소스 파싱
- 고정 틱 기반의 결정론적 게임 루프
- 레밍의 행동 상태 머신과 중력·충돌 처리
- 파기와 폭발에 따른 픽셀 단위 지형 변경
- 데스크톱과 웹에서 동작하는 플랫폼 어댑터

전체 설계와 단계별 작업은 [프로젝트 기획서](mydocs/plans/issue-1_rusty_lemmings.md)를 참고하세요.

구현 순서와 목표별 완료 기준은 [분할정복 기반 전체 구현 계획](mydocs/plans/issue-1_implementation_plan.md)에 정리합니다. G00~G06과 G07-01 Digger·G07-02 Basher·G07-03 Miner를 완료했으며 [G07-04 Builder / #34](mydocs/reports/issue-34_builder.md)의 병합·CI도 완료했습니다. [G07-05 Blocker / #36](mydocs/reports/issue-36_blocker.md)의 PR #37 병합·main CI·종료를 확인했습니다. [G07-06 Bomber / #38](mydocs/reports/issue-38_bomber.md)의 PR #39 병합·main CI·종료와 사용자 E키 확인을 마쳤습니다. [G07-07 Climber / #40](mydocs/reports/issue-40_climber.md)의 PR #41 병합·main CI·종료를 확인했습니다. [G07-08 Floater / #42](mydocs/reports/issue-42_floater.md)의 PR #43 병합·main CI·종료를 확인했습니다. [G07-09 Nuke / #44](mydocs/reports/issue-44_nuke.md)의 PR #45 병합·main CI·종료를 확인했습니다. [G07-10 / #46](mydocs/reports/issue-46_skill_interactions.md)의 할당/혼합 회귀도 완료해 G07 직업·지형 목표의 기준을 충족했습니다. [G08-01 / #48](mydocs/reports/issue-48_exit_rescue.md)의 출구·구출을 연결했습니다. [G08-02 / #50](mydocs/reports/issue-50_hazards.md)의 물·불·함정도 연결했습니다. [G08-03 / #52](mydocs/reports/issue-52_level_outcome.md)의 시간·구출률·승패 판정을 연결했습니다. [G08-04 / #54](mydocs/reports/issue-54_results_restart.md)에서 결과 화면·동일 레벨 재시작·다음 레벨 전환을 연결했습니다. [#56 원본 기술 패널·미니맵](mydocs/reports/issue-56_skill_panel.md)을 통합했습니다. [#58 포인터 선택·RightClickGlitch](mydocs/reports/issue-58_pointer_selection.md)의 병합·실제 CI를 확인했습니다. [#60 한 레벨 통합 완주](mydocs/reports/issue-60_native_playthrough.md)의 실제 SDL 성공·실패·재시작/전환·독립 재생과 DOS 결과/장면 비교를 검증했습니다. 정확한 게시 CI·G08 완료 처리는 #60에서 추적하며 다음은 G09 웹의 동일 코어 경계입니다.

진행 상황은 [GitHub 이슈 #1](https://github.com/edwardkim/ohmylemmings/issues/1)에서 추적합니다.

## Hyper-Waterfall 개발 방법론

이 프로젝트는 관리자가 정립한 Hyper-Waterfall을 채택합니다. 구현 전에 관련 문서로 컨텍스트를 맞추고, 작은 모듈부터 TDD로 구현하며, 피드백과 기술 지식·문제 해결 이력을 문서에 반영합니다.

모든 작업은 GitHub 이슈를 먼저 등록하고 번호·URL을 확인한 뒤 진행합니다. 날짜별 작업일지·진행 기록·결과 보고서에 이슈를 연결하고, 검증·보고 후 완료 처리합니다.

`mydocs/` 작업 문서는 `issue-<번호>_<주제>.md`로 추적합니다. `orders/`는 `yyyymmdd.md` 안에 이슈별 작업 목록을 기록하고, 폴더 안내용 `README.md`는 유지합니다.

```text
mydocs/
├── plans/             # 계획 및 아키텍처
├── orders/            # 날짜별 작업일지와 이슈별 목록
├── working/           # 진행 중 작업 상태와 컨텍스트
├── reports/           # 작업 결과와 검증 보고서
├── feedbacks/         # 피드백과 반영 결과
├── tech_insights/     # 기술 분석 및 도메인 지식
└── troubleshootings/  # 문제 해결 이력
```

## 참여 방법

작업을 시작하기 전에 [기여 지침](AGENTS.md)과 [문서 안내](mydocs/README.md)를 읽어주세요. PR에는 변경 목적, 관련 작업, 검증 결과를 기록하고 계획된 기능과 구현된 동작을 구분합니다.

[G07-01 Digger](mydocs/reports/issue-31_digger.md)의 구현·검증·게시/실제 CI를 완료했습니다. Digger는 아래, Basher는 수평, Miner는 대각선으로 굴착합니다. M/B/D 키로 선택한 뒤 클릭하며 정수 ID 입력으로 재생할 수 있습니다. 구출·승패·결과/재시작·원본 기술 패널과 포인터를 연결했고 [#60](mydocs/reports/issue-60_native_playthrough.md)에서 한 레벨 완주를 통합 검증했습니다.

[G07-02 Basher](mydocs/reports/issue-32_basher.md)의 구현·기준/원본 검증·게시/실제 CI도 완료했습니다. B/D 키로 Basher/Digger를 선택하고 클릭합니다. `--level 5`로 Basher 원본 레벨을 선택할 수 있습니다.

[G07-03 Miner](mydocs/reports/issue-33_miner.md)의 구현·기준/원본 검증·게시/실제 CI도 완료했습니다. M/B/D로 직업을 선택하고 클릭합니다. `--level 4`로 원본 Miner 레벨을 선택합니다.

[G07-04 Builder](mydocs/reports/issue-34_builder.md)는 계단을 만듭니다. U/M/B/D로 직업을 선택한 뒤 클릭하며 `--level 7`로 원본 Builder 레벨을 선택합니다. 구현·검증·self-review·병합·main CI를 완료했습니다.

[G07-05 Blocker](mydocs/reports/issue-36_blocker.md)는 다른 레밍의 방향을 바꿉니다. K/U/M/B/D로 직업을 선택한 뒤 클릭하며 `--level 3`으로 원본 Blocker 레벨을 선택합니다. self-review와 통합은 [#36](https://github.com/edwardkim/ohmylemmings/issues/36)에서 추적합니다.

[G07-08 Floater](mydocs/reports/issue-42_floater.md)는 영구 우산 능력을 부여합니다. F키로 선택한 뒤 클릭하며 `--level 2`로 원본 Fun02를 엽니다. 높은 곳에서 우산을 펼치고 안전하게 착지합니다. 재생 예는 `54 0 AssignFloater 0`이며 직업 상호작용은 #46, 출구·구출은 #48에서 연결했으며 승패/음향은 후속입니다.

[G07-09 Nuke](mydocs/reports/issue-44_nuke.md)는 N키로 작동합니다. 이후 생성을 중단하고 기존 레밍에 하나씩 카운트다운을 붙입니다. 일시정지 중에는 실행하지 않으며 Bomber 수량을 소비하지 않습니다. 재생 예는 `120 0 Nuke`이고 `--synthetic-nuke`로 원본 없이 검사합니다. 직업 상호작용은 #46, 출구·구출은 #48에서 연결했으며 승패·원본 하단 패널·음향은 후속입니다.

[G10-02 ADLIB 명령·레지스터 스트림 / #74](mydocs/tech_insights/issue-74_adlib_stream.md)에서 no_std9채널 해석기·오류 보존·합성/원본 직접 대조를 연결합니다. 다음은 OPL/PCM이며 G10 전체 사운드 출력은 진행 중입니다.

[G10-03 OPL·결정론 PCM / #76](mydocs/reports/issue-76_opl_pcm.md)에서 no_std 정수 합성·유리수 오디오 시계·native/WASM PCM과 독립 C 전체 샘플 비교를 연결했습니다. 엔진은 [LGPL-2.1-or-later 고지/소스·재링크 안내](third_party/README.md)를 포함합니다. 다음은 게임 이벤트/DOS ID이며 SDL/Web 실제 출력은 후속입니다.

[G10-04 게임 사운드·DOS ID / #78](mydocs/reports/issue-78_game_sound.md)에서 논리 사건·원본 효과음/음악 선택·공유 큐/세대·native/actual WASM 명령→PCM을 연결합니다. 다음은 G10-05 SDL/Web 실제 출력입니다.

[G10-05 SDL/Web 오디오 출력 / #80](mydocs/reports/issue-80_audio_output.md)에서 원본/연습 음원·장치 출력·활성화/음소거/볼륨·숨김/세대와 LAN HTTP를 연결합니다. 웹의 소리 켜기 버튼 또는 S, 양쪽 PageUp/PageDown으로 조절합니다. LGPL 라이선스와 재빌드 소스 묶음을 웹 footer에서 제공합니다. 다음은 G10-06 통합 대조·청취와 마감입니다.

사용자 승인으로 [G12 배포 준비 / #115](mydocs/plans/issue-115_release_preparation.md)를 완료했습니다. [#113 성능 결과](mydocs/reports/issue-113_web_dispatch.md)의 남은60fps 검증은 더 좋은 호스트에 이월하며, 현재 장비에서 추가 조정·반복하지 않습니다.

Linux·Web 전체 게임 archive는 원본28개(그래픽·120레벨·음원)와 대응 소스·라이선스·해시를 포함합니다. `python3 tools/reference_assets.py fetch` 후 `make package-linux` 또는 `make package-web`으로 만들고 압축 해제 실행까지 검사합니다. Linux는 실행 파일 옆 data를 사용하고 Web은 HTTP에서 자동으로 Fun01을 엽니다. [사용·검증 안내](mydocs/reports/issue-122_linux_web_full_game.md). MIT·제3자 고지는 이후 빌드부터 포함하며 기존 공개 v0.1.0 파일에는 소급 적용하지 않습니다.

Windows 전체 게임 ZIP은 [#118 패키징 안내](mydocs/reports/issue-118_windows_full_game.md)를 따릅니다. 원본 그래픽·전체120레벨·음원을 포함하며 압축 해제 후 `OhMyLemmings.exe`를 더블클릭합니다. GitHub Actions의 `Windows full game` 검사는 실제 Windows에서 이 패키지의 첫 실행·120개 선택·음향·재시작을 검증하고 `development-windows-x86_64` artifact를 제공합니다.

**브라우저에서 플레이:** [Oh My Lemmings](https://edwardkim.github.io/ohmylemmings/). 최신 정식 릴리즈의 원본 그래픽·120레벨·음원이 포함되어 있습니다. 소리는 화면의 «소리 켜기»를 눌러 활성화하세요.

**게임 다운로드:** [Releases](https://github.com/edwardkim/ohmylemmings/releases). 운영 기준은 [버전·태그 정책](mydocs/plans/issue-124_release_policy.md), 버전별 안내는 [릴리즈 기록](mydocs/releases/README.md)을 참고하세요.

macOS 전체 게임 앱은 [#120 배포 안내](mydocs/reports/issue-120_macos_full_game.md)를 따릅니다. 원본120개 레벨·음원을 포함한 Apple Silicon/Intel universal2 `.app` ZIP과 실제 양쪽 Mac CI 검증 결과를 제공합니다. 현재 ad-hoc 서명이며 Developer ID/Apple 공증은 자격 확보 후의 별도 단계입니다.

[#160 HD 입출구](mydocs/reports/issue-160_hd_portals.md)에서 전체5테마 입출구와 출구 불꽃을 Native/Web 개발 미리보기에 연결합니다. [생성 자산·정확한 프롬프트](assets/remaster/README.md)를 참고하세요.

[#163 팩 정리](mydocs/reports/issue-163_hd_pack.md): 원본32PNG·440자산을 유지하고 런타임27atlas/131,699,864바이트로 정리했습니다. 필터링에 민감한 페이지는 기존 배치를 유지하며 Native/Canvas 화면 동일성을 검증했습니다. 전체 리마스터·성능 인수는 후속입니다.

[#169 테마별 HD 로딩](mydocs/reports/issue-169_hd_loading.md): 공통 자산과 현재 테마/특수 장면을 선택합니다. 현재 선택 팩은 최대26atlas/약124.7MB이며, HD UI에는 별도 약4.7MB의 RGBA가 추가됩니다. `make hd-assets`/`make web`으로 카탈로그를 갱신하며 Native는 `--hd-pack FILE_OR_DIRECTORY`도 지원합니다. 기존 v1 팩과 원본 모드는 유지합니다.
