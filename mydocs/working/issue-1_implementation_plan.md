# 전체 구현 계획 수립 진행 기록

## 현재 상태

계획 수립과 [상위 이슈 #1](https://github.com/edwardkim/ohmylemmings/issues/1) 등록을 완료했습니다. G00의 원작 확보(#2)·소스 고정(#3)·비교 계약(#5), [G01-01 / #6](issue-6_rust_workspace.md)의 개발 골격과 [G01-02 / #7](issue-7_continuous_integration.md)의 실제 CI 검증을 마쳤습니다. [G02-01 / #8](issue-8_dat_sections.md) 구간 읽기와 [G02-02 / #9](issue-9_dat_decompression.md) 압축 해제의 검증으로 G02도 완료했습니다. G00·G01·G02는 완료, G03는 [G03-01 / #10](issue-10_level_records.md) 이후 [G03-02 / #11](issue-11_level_catalog.md)도 마쳤습니다. [G03-03 / #12](issue-12_ground_metadata.md)도 합성 테스트·원본 5개 전체 필드 대조·실제 CI를 통과해 완료했습니다. [G03-04 / #13](issue-13_vgagr_images.md)도 합성 테스트·원본 전체 픽셀 대조·실제 CI를 통과해 완료했습니다. [G03-05 / #14](issue-14_vgaspec_backgrounds.md)도 합성 테스트·원본 전체 픽셀 대조·실제 CI를 통과해 완료했습니다. [G03-06 / #15](issue-15_level_terrain.md)도 전체 120개 지형·팔레트 대조·로컬 검사·실제 CI를 통과해 완료했습니다. [G03-07 / #16](issue-16_lemming_sprites.md)도 전체 프레임 대조·로컬 검사·실제 CI를 통과해 완료했습니다. [G03-08 / #17](issue-17_main_ui.md)도 지원 이미지/글리프 전체 대조·로컬 검사·실제 CI를 통과해 완료했습니다. [G03-09 / #18](issue-18_main_masks.md)도 전체 비트 대조·로컬 검사·실제 CI를 통과해 완료했습니다. [G03-10 / #19](../tech_insights/issue-19_level_objects.md)의 객체 합성·전체 대조·게시·실제 CI까지 확인해 G03 전체를 완료했습니다. [G04-01 / #20](../tech_insights/issue-20_terrain_mask.md)의 읽기·경계 검증·게시·실제 CI까지 완료했습니다. [G04-02 / #21](../tech_insights/issue-21_terrain_edits.md)의 셀 쓰기·전수 대조·게시·실제 CI까지 완료했습니다. [G04-03 / #22](issue-22_terrain_bulk.md)의 영역/마스크 변형·전수 대조·게시·실제 CI까지 확인해 G04 전체를 완료했습니다. [G05-01 / #23](issue-23_fixed_ticks.md)의 고정 틱·입력 순서 골격과 전수 재생·게시·실제 CI까지 확인했습니다. [G05-02 / #24](issue-24_startup_schedule.md)의 시작 시계·입구·생성 스케줄과 전수 재생·게시·실제 CI까지 확인했습니다. [G05-03 / #25](issue-25_actor_motion.md)의 기본 개체 이동/제거·전수/통합 재생·게시·실제 CI까지 확인했습니다. G05 전체를 #26의 기준 함수 실행·상태 추적·재생·게시·실제 CI로 완료했습니다.

## 관련 문서

- [작업일지](../orders/20261018.md#issue-1)
- [전체 구현 계획](../plans/issue-1_implementation_plan.md)
- [결과 보고서](../reports/issue-1_implementation_plan.md)

## 진행 내용

- 원본 호환 범위 확정부터 배포까지 13개 목표로 분할했습니다.
- 목표별 선행 조건, 작은 작업, 완료 증거를 정의했습니다.
- 원본 리소스 경로와 합성 데이터 기반 코어 경로의 통합 지점을 G06으로 정했습니다.
- 첫 착수 목표 G00를 세 작업으로 세분화했습니다.
- 문서 링크 36개와 공백, 13개 목표의 의존관계를 검사했습니다.
- 이슈 #1에 13개 구현 목표 체크리스트를 등록했습니다.
- 문서를 `main`에 게시하고 원격 파일 17개의 blob 해시가 로컬과 일치함을 확인했습니다.

## 다음 행동

1. [#26 상태 추적·재생](issue-26_state_replay.md)의 기준 실행/원본 대조·게시·실제 CI로 G05 전체 완료 기준을 충족했습니다. 최종 문서 게시·CI·상위 #1 갱신·#26 종료는 GitHub 이슈 기록에서 추적합니다.
2. 다음 후보는 별도 이슈의 G06-01 네이티브 화면·입력·렌더 주기 연결입니다. 전체 DOS/직업/객체/승패/오디오는 후속 범위로 유지합니다.
