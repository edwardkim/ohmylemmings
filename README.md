# Rusty Lemmings

[![CI](https://github.com/edwardkim/ohmylemmings/actions/workflows/ci.yml/badge.svg)](https://github.com/edwardkim/ohmylemmings/actions/workflows/ci.yml)

고전 DOS 게임 레밍즈(Lemmings)의 데이터와 게임 동작을 Rust로 재현하는 프로젝트입니다. 렌더링·입출력과 분리된 코어 엔진을 만들고, SDL2 기반 데스크톱 환경과 WebAssembly 기반 웹 환경을 지원하는 것이 목표입니다.

## 현재 상태

G00 기준 정의·G01 개발 기반·G02 DAT 언패킹을 완료했습니다. `no_std` 코어 자료형과 DAT 구간 검증·압축 해제를 도입했습니다. 원작 파일 50개·소스 46개와 120개 레벨·8직업·음악 21개·효과음 18개의 검증 범위를 고정했습니다. [G02-02 압축 해제](mydocs/tech_insights/issue-9_dat_decompression.md)는 합성 테스트, 원본 21개 파일·102개 구간의 해시 대조와 실제 Rust·Python·WASM CI를 통과했습니다.

G03 리소스 해석을 완료했습니다. [기본 레벨 레코드](mydocs/tech_insights/issue-10_level_records.md), [ODDTABLE·120개 카탈로그](mydocs/tech_insights/issue-11_level_catalog.md), [GROUND 메타데이터·팔레트](mydocs/tech_insights/issue-12_ground_metadata.md), [VGAGR 이미지·마스크](mydocs/tech_insights/issue-13_vgagr_images.md)의 원본 대조와 실제 CI를 통과했습니다.

[G03-05 VGASPEC 특수 배경](mydocs/tech_insights/issue-14_vgaspec_backgrounds.md)을 완료했으며 [G03-06 초기 레벨 지형](mydocs/tech_insights/issue-15_level_terrain.md)도 전체 맵 대조·로컬 검사·실제 CI를 통과해 완료했습니다. 120개 진입점·30412800픽셀·팔레트를 독립 Python과 대조했고 실제 TS 렌더링 코드와의 규칙 차이도 기록했습니다. [G03-07 MAIN.DAT 레밍 스프라이트](mydocs/tech_insights/issue-16_lemming_sprites.md)도 337프레임·63488픽셀의 Rust·Python·TS 대조·로컬 검사·실제 CI를 통과해 완료했습니다. [G03-08 MAIN.DAT UI·폰트](mydocs/tech_insights/issue-17_main_ui.md)도 식별된 19개 묶음·200개 이미지/글리프 전체 픽셀 대조·로컬 검사·실제 CI를 통과해 완료했습니다. [G03-09 MAIN.DAT 마스크·카운트다운](mydocs/tech_insights/issue-18_main_masks.md)도 전체 23프레임·3104비트 대조·로컬 검사·실제 CI를 통과해 완료했습니다. G03-01~10의 검증·게시·실제 CI로 G03 전체 완료 기준을 충족했습니다. [G03-10 객체 이미지 배치·합성](mydocs/tech_insights/issue-19_level_objects.md)도 160개 기본 결과·240개 표시 결과의 독립 대조·게시·실제 CI를 통과했습니다. [G04-01 픽셀 지형 읽기·경계](mydocs/tech_insights/issue-20_terrain_mask.md)도 합성 전수 대조·게시·실제 CI를 통과해 완료했습니다. G04의 읽기·셀 쓰기·영역 변형을 모두 검증했으며 [G04-02 지형 쓰기·셀 변경](mydocs/tech_insights/issue-21_terrain_edits.md)도 전수 대조·게시·실제 CI를 통과해 완료했습니다. [G04-03 영역·마스크 기반 지형 추가/삭제](mydocs/tech_insights/issue-22_terrain_bulk.md)도 전수 대조·게시·실제 CI를 통과했습니다. G04 전체 완료 기준을 충족했습니다. [G05-01 고정 틱·입력 순서 골격](mydocs/tech_insights/issue-23_fixed_ticks.md)도 합성 분할/재생·게시·실제 CI를 통과해 완료했습니다. G05 전체는 진행 중이고 다음 후보는 G05-02 시작 시계·입구·생성 스케줄입니다. 실제 게임 실행과 DOS 호환성 판정은 후속 목표에서 수행합니다.

## Rust 개발

Rustup을 설치한 뒤 저장소 루트에서 실행합니다. [rust-toolchain.toml](rust-toolchain.toml)이 Rust 1.93.1과 rustfmt·Clippy를 선택합니다. 도구 체인 설치 후 현재 패키지는 외부 의존성 없이 오프라인으로 빌드됩니다.

전체 개발 검증에는 GNU Make, Python 3.12와 WASM 타깃도 준비합니다.

```sh
rustup target add wasm32-unknown-unknown
make check
```

`make check-rust`, `make check-python`, `make check-wasm`으로 각각 실행할 수도 있습니다. [CI·검증 안내](mydocs/plans/issue-7_continuous_integration.md)와 [Makefile](Makefile)에 세부 명령을 정리합니다. 같은 명령이 [GitHub Actions](.github/workflows/ci.yml)의 push·PR·수동 실행에서 사용됩니다.

- [crates/lemmings-core](crates/lemmings-core/src/lib.rs): 레벨 식별 자료형·0/1 지형 읽기/변형·17Hz 정수 누적·경계 입력/논리 진행 골격
- [crates/lemmings-resources](crates/lemmings-resources/src/lib.rs): DAT·기본 레벨·ODDTABLE·120개 카탈로그·GROUND·VGAGR·VGASPEC·초기 지형·객체 합성·MAIN 레밍/UI/폰트·마스크·숫자와 팔레트, 코어 ID를 사용하는 로컬 의존
- [개발 골격 설계](mydocs/plans/issue-6_rust_workspace.md): 모듈 경계·API·개별/WASM 빌드 명령

현재 빌드 대상은 라이브러리 2개이며, 게임 실행 명령은 G06의 네이티브 어댑터에서 도입합니다.

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

구현 순서와 목표별 완료 기준은 [분할정복 기반 전체 구현 계획](mydocs/plans/issue-1_implementation_plan.md)에 정리합니다. G00~G04를 완료했으며 G05 시뮬레이션을 작은 이슈로 나눠 진행합니다.

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
