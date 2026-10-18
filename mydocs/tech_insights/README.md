# 기술 분석 문서

Lemmix와 Lemmings.ts 분석, DAT/VGASPEC 구조, 상태 머신과 틱 동작 등 구현에 필요한 도메인 지식을 관리합니다.

- 레퍼런스의 저장소, 버전 또는 커밋, 파일·함수 위치를 기록합니다.
- 확인된 상수·데이터 구조·동작과 아직 검증하지 않은 가정을 구분합니다.
- 담당 이슈, 재현 방법이나 테스트 근거, 관련 계획·작업일지를 연결합니다.

파일명: `issue-<번호>_<주제>.md`. 예: `issue-2_dos_original_baseline.md`.

## 현재 자료

- [DOS 기준판과 원본 리소스](issue-2_dos_original_baseline.md)
- [레퍼런스 코드 버전·파일·함수 지도](issue-3_reference_code_map.md)
- [호환성 범위·입력·상태 비교 기준](issue-5_compatibility_baseline.md)
- [DOS 전체 레벨 지원 목록](issue-5_level_inventory.md)
- [DAT 구간 형식·읽기 API·원본 102개 구간 검증](issue-8_dat_sections.md)
- [DAT 역방향 비트·압축 명령·해제 API와 원본 해시 검증](issue-9_dat_decompression.md)
- [DOS 기본 레벨 헤더·배치·슬롯과 원본 80개 레코드 검증](issue-10_level_records.md)
- [ODDTABLE·120개 표시 카탈로그와 변형·기본 배치 보존](issue-11_level_catalog.md)
- [GROUND 메타데이터·팔레트와 원본 5개 파일 전체 필드 대조](issue-12_ground_metadata.md)
- [VGAGR 비트플레인·마스크·프레임과 원본 전체 픽셀 대조](issue-13_vgagr_images.md)
- [VGASPEC 특수 배경·팔레트·RLE와 원본 4개 전체 픽셀 대조](issue-14_vgaspec_backgrounds.md)
- [초기 지형 합성·플래그 우선순위와 120개 전체 맵 대조](issue-15_level_terrain.md)
- [MAIN.DAT 레밍 스프라이트·방향/기준점과 전체 337프레임 대조](issue-16_lemming_sprites.md)
- [MAIN.DAT UI·폰트·투명도 정책과 200개 이미지/글리프 전체 대조](issue-17_main_ui.md)
- [MAIN.DAT 마스크·카운트다운·극성/숫자 차이와 전체 23프레임 대조](issue-18_main_masks.md)

- [#26 G05-04 상태 추적·재생·기준 실행 대조](issue-26_state_replay.md)

- [#27 G06-01 네이티브 화면·입력 연결](issue-27_native_view.md)

- [#28 실제 DOS 반복 실행·장면 대조](issue-28_dos_comparison.md)
- [#29 DOS 런타임 진입 전제](issue-29_dos_runtime.md)
- [#30 DOS 화면·속도 차이 추적](issue-30_dos_differences.md)

- [#31 G07-01 Digger 계약·원본/기준 실행](issue-31_digger.md)

- [#32 G07-02 Basher](issue-32_basher.md)
