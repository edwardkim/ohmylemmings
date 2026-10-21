# 기술 분석 문서

Lemmix와 Lemmings.ts 분석, DAT/VGASPEC 구조, 상태 머신과 틱 동작 등 구현에 필요한 도메인 지식을 관리합니다.

- 레퍼런스의 저장소, 버전 또는 커밋, 파일·함수 위치를 기록합니다.
- 확인된 상수·데이터 구조·동작과 아직 검증하지 않은 가정을 구분합니다.
- 담당 이슈, 재현 방법이나 테스트 근거, 관련 계획·작업일지를 연결합니다.

파일명: `issue-<번호>_<주제>.md`. 예: `issue-2_dos_original_baseline.md`.

## 현재 자료

- [#87 네 난이도 ID·선택/전환·재생 호환 계약](issue-87_rank_selection.md)

- [#85 G11 전체120레벨 지원 범위·실행 증거 점검표](issue-85_compatibility_matrix.md)

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

- [#33 G07-03 Miner](issue-33_miner.md)

- [#34 G07-04 Builder](issue-34_builder.md)

- [#36 G07-05 Blocker](issue-36_blocker.md)

- [#38 G07-06 Bomber](issue-38_bomber.md)

- [#40 G07-07 Climber](issue-40_climber.md)

- [#42 G07-08 Floater](issue-42_floater.md)

- [#44 G07-09 Nuke](issue-44_nuke.md)

- [#46 G07-10 직업 전환·혼합 상호작용](issue-46_skill_interactions.md)

- [#48 G08-01 객체·출구/구출](issue-48_exit_rescue.md)

- [#50 G08-02 위험물·함정](issue-50_hazards.md)

- [#52 시간·구출률·승패 계약](issue-52_level_outcome.md)

- [#54 결과 화면·재시작](issue-54_results_restart.md)

- [#56 원본 기술 패널·미니맵](issue-56_skill_panel.md)

- [#58 포인터 선택·RightClickGlitch](issue-58_pointer_selection.md)

- [#60 네이티브 한 레벨 통합 완주](issue-60_native_playthrough.md)

- [#62 공유 세션·웹 이식 경계](issue-62_shared_session.md)

- [#64 자산 바이트 공급·공통 장면과 패널](issue-64_shared_game.md)

- [#66 JS/WASM 바인딩·합성 브라우저 진입](issue-66_wasm_browser.md)

- [#68 G09-04 브라우저 입력·원본 자산·레벨 전환](issue-68_web_input_assets.md)

- [#70 G09-05 브라우저 통합 완주](issue-70_web_playthrough.md)

- [#72 AdLib 자원·통합 계약](issue-72_adlib_catalog.md)

- [#74 ADLIB 명령·레지스터 스트림](issue-74_adlib_stream.md)

- [#76 OPL 합성기·결정론 PCM](issue-76_opl_pcm.md)

- [#78 게임 사운드 이벤트·DOS 숫자 ID](issue-78_game_sound.md)

- [#80 SDL/Web 오디오 출력](issue-80_audio_output.md)

- [#82 실제 DOS OPL·SDL·WASM 통합 사운드](issue-82_audio_playthrough.md)

- [#89 G11-03 특수4개 장면 연결 계약](issue-89_special_scenes.md)

- [#91 G11-04 전체120레벨 초기 재생 계약](issue-91_initial_replay.md)

- [#94 튜토리얼7개 해법 비교 계약](issue-94_solution_replay.md)
