# Hyper-Waterfall 문서 안내

프로젝트 기획은 [전체 기획서 / #1](plans/issue-1_rusty_lemmings.md), 기여 지침은 [AGENTS.md](../AGENTS.md)를 기준으로 합니다.

단계별 구현 목표와 작업 순서는 [전체 구현 계획](plans/issue-1_implementation_plan.md)을 참고합니다.

G00-01의 [DOS 기준판 식별·원본 확보](tech_insights/issue-2_dos_original_baseline.md)를 완료했습니다. 로컬 데이터 준비는 [리소스 안내](../assets/README.md)를 참고합니다.

[G00-02 레퍼런스 코드 지도](tech_insights/issue-3_reference_code_map.md)와 [G00-03 호환성 검증 기준](tech_insights/issue-5_compatibility_baseline.md)을 완료했습니다. [G01-01 Rust 개발 골격](plans/issue-6_rust_workspace.md)과 [G01-02 CI·개발 검증](plans/issue-7_continuous_integration.md)을 마쳐 G01을 완료했습니다. [G02-01 DAT 구간 읽기](tech_insights/issue-8_dat_sections.md)와 [G02-02 압축 해제](tech_insights/issue-9_dat_decompression.md)의 테스트·원본 해시·실제 CI 검증으로 G02도 완료했습니다. [G03-01 기본 레벨 레코드](tech_insights/issue-10_level_records.md) 이후 [G03-02 ODDTABLE·카탈로그](tech_insights/issue-11_level_catalog.md)도 마쳤으며 [G03-03 GROUND 메타데이터·팔레트](tech_insights/issue-12_ground_metadata.md)도 합성 테스트·원본 5개 전체 필드 대조·실제 CI를 통과해 완료했습니다. [G03-04 VGAGR 비트플레인·마스크](tech_insights/issue-13_vgagr_images.md)도 합성 테스트·원본 867개 이미지/프레임의 전체 픽셀 대조·실제 CI를 통과해 완료했습니다. [G03-05 VGASPEC 특수 배경](tech_insights/issue-14_vgaspec_backgrounds.md)도 합성 테스트·원본 4개 전체 픽셀 대조·실제 CI를 통과해 완료했습니다. [G03-06 초기 지형 합성](tech_insights/issue-15_level_terrain.md)도 120개 전체 맵·팔레트 대조·로컬 검사·실제 CI를 통과해 완료했습니다. [G03-07 MAIN.DAT 레밍 스프라이트](tech_insights/issue-16_lemming_sprites.md)도 전체 프레임 대조·로컬 검사·실제 CI를 통과해 완료했습니다. [G03-08 MAIN.DAT UI·폰트](tech_insights/issue-17_main_ui.md)도 지원 이미지/글리프 전체 대조·로컬 검사·실제 CI를 통과해 완료했습니다. [G03-09 MAIN.DAT 마스크·카운트다운](tech_insights/issue-18_main_masks.md)도 전체 비트 대조·로컬 검사·실제 CI를 통과해 완료했습니다. G03-01~10의 검증·게시·실제 CI로 G03 전체 완료 기준을 충족했습니다. [G03-10 객체 이미지 배치·합성](tech_insights/issue-19_level_objects.md)도 전체 객체 대조·게시·실제 CI를 통과했습니다. [G04-01 픽셀 지형 읽기·경계](tech_insights/issue-20_terrain_mask.md)도 합성 전수 대조·게시·실제 CI를 통과해 완료했습니다. G04의 읽기·셀 쓰기·영역 변형을 모두 검증했고 [G04-02 지형 쓰기·셀 변경](tech_insights/issue-21_terrain_edits.md)도 전수 대조·게시·실제 CI를 통과해 완료했습니다. [G04-03 영역·마스크 기반 지형 추가/삭제](tech_insights/issue-22_terrain_bulk.md)도 전수 대조·게시·실제 CI를 통과했습니다. G04 전체 완료 기준을 충족했습니다. [G05-01 고정 틱·입력 순서 골격](tech_insights/issue-23_fixed_ticks.md)도 합성 분할/재생·게시·실제 CI를 통과해 완료했습니다. [G05-02 시작 시계·입구·생성 스케줄](tech_insights/issue-24_startup_schedule.md)도 합성 경계/재생·게시·실제 CI를 통과해 완료했습니다. [G05-03 개체 이동·제거](tech_insights/issue-25_actor_motion.md)도 전수/통합 재생·게시·실제 CI를 통과해 완료했습니다. [#26 상태 추적·재생·기준 실행 대조](tech_insights/issue-26_state_replay.md)의 실제 기준 함수 실행·합성/원본 대조·게시·CI를 확인해 G05 전체를 완료했습니다. 이후 G06 네이티브 화면·DOS 대조/보정도 완료했습니다. 전체 게임 호환성은 후속 목표에서 검증합니다.

[#27 G06-01 네이티브 화면·입력](tech_insights/issue-27_native_view.md)의 실제 X11/키/화면·게시·CI를 확인해 완료했습니다. 후속 #28의 DOS 실행·#30의 전체 장면/속도 보정을 거쳐 G06 전체를 완료했습니다.

## 폴더 구조

- [plans/](plans/README.md): 마일스톤 및 아키텍처 계획
- [orders/](orders/README.md): 날짜별 작업일지와 이슈별 작업 목록
- [working/](working/README.md): 진행 중 작업 상태와 이어서 작업할 컨텍스트
- [reports/](reports/README.md): 작업 결과와 검증 보고서
- [feedbacks/](feedbacks/README.md): 사용자 및 리뷰어의 피드백과 반영 결과
- [tech_insights/](tech_insights/README.md): 레퍼런스 분석과 확인된 기술 지식
- [troubleshootings/](troubleshootings/README.md): 문제 원인과 해결 이력

## 문서명과 이슈 추적

- 작업 문서는 `issue-<번호>_<주제>.md`로 작성합니다. 예: `tech_insights/issue-3_reference_code_map.md`.
- 문서 상단에 담당 GitHub 이슈의 번호·URL을 연결합니다. 여러 작업에 걸친 기획서는 상위 추적 이슈를 사용합니다.
- `orders/`는 날짜별 일지이므로 `yyyymmdd.md`를 사용합니다. Asia/Seoul 작업일자를 기준으로 하루 한 파일에 이슈 번호·작업·결과·다음 행동을 목록으로 기록합니다.
- 폴더 안내용 `README.md`는 공통 색인이므로 이름을 유지합니다. 문서를 이동하면 저장소와 GitHub 이슈 본문의 링크도 갱신합니다.

[20261018 작업일지](orders/20261018.md)에서 #1~#25와 이슈 도입 전의 공개 기록을 확인할 수 있습니다. 문서 운영 규칙의 적용은 [#4](https://github.com/edwardkim/ohmylemmings/issues/4)에서 추적합니다.

## 작업 흐름

1. GitHub 이슈를 먼저 등록하고 생성된 번호·URL을 조회해 확인합니다.
2. `orders/yyyymmdd.md`의 이슈별 목록과 `working/issue-<번호>_<주제>.md`에 작업을 기록하고 관련 계획·피드백·기술 분석 문서를 읽습니다.
3. 작은 모듈부터 TDD로 구현하고 검증하며, 진행 상황과 다음 작업을 `working/`에 기록합니다.
4. 완료된 작업의 변경 내용과 검증 결과를 `reports/`에 정리합니다.
5. 발견한 지식, 피드백 반영 결과, 문제 해결 이력을 기록하고 계획의 진행 상태를 갱신한 뒤 이슈를 완료 처리합니다.

문서에서는 예정된 작업과 구현·검증이 완료된 내용을 구분합니다.

[#28 G06-02 실제 DOS 세 실행·장면 대조](reports/issue-28_dos_comparison.md)를 완료했습니다. 상부 일치에서 발견한 전체 화면·속도 차이는 [#30 보정](tech_insights/issue-30_dos_differences.md)과 [게시·CI](reports/issue-30_dos_differences.md)로 마무리했습니다. G00~G06과 Digger·Basher·Miner를 완료했으며 현재 [G07-04 Builder / #34](working/issue-34_builder.md)의 구현·검증·자체 검토를 마쳤습니다. 최종 통합 기록은 #34에서 추적합니다.

[#31 Digger](reports/issue-31_digger.md)의 구현·기준/원본 검증·게시/실제 CI를 완료했습니다. Basher와 Miner도 완료했으며 현재 [G07-04 Builder / #34](working/issue-34_builder.md)의 구현·검증·자체 검토를 마쳤습니다. 최종 통합 기록은 #34에서 추적합니다. G07 전체는 진행 중입니다.

[#32 Basher](reports/issue-32_basher.md)의 구현·기준/원본/네이티브·전체 회귀·게시/실제 CI를 완료했습니다. G07 전체는 진행 중입니다.

[#33 Miner](reports/issue-33_miner.md)의 구현·기준/원본/네이티브·전체 회귀·게시/실제 CI를 완료했습니다. G07 전체는 진행 중입니다.
