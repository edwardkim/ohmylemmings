# 작업 결과 보고서

- [#138 Fun 01 HD 아트 디렉션](issue-138_hd_art_direction.md): 새 손그림풍2D 시안·출처/규격·전체 자산 대응과 실제 구현 전 한계.

- [#1 초기 구현·플레이 배포 종료](issue-1_initial_development_closure.md): 메인테이너 지시에 따른 상위 추적 완료, 성능·메모리 #113 독립 후속.

- [#87 네 난이도 선택·재시작·전환 검증](issue-87_rank_selection.md)

- [#85 G11-01 전수 점검표·성능 측정 기준](issue-85_compatibility_validation.md)

완료된 작업의 변경 내용과 검증 결과를 관리합니다.

- 담당 이슈와 관련 계획, 날짜별 작업일지, 진행 기록, 피드백 문서를 연결합니다.
- 구현하거나 변경한 내용과 완료 기준의 충족 여부를 기록합니다.
- 실행한 검증 명령과 결과를 남기고, 미실행 검증은 이유를 명시합니다.
- 남은 문제와 후속 작업을 기록하고 필요한 기술 분석·문제 해결 문서를 연결합니다.

파일명: `issue-<번호>_<주제>.md`. 예: `issue-3_reference_code_map.md`.

- [#26 G05-04 상태 추적·재생·기준 실행 대조](issue-26_state_replay.md)

- [#27 G06-01 네이티브 화면·입력 연결](issue-27_native_view.md)

- [#28 DOS 실행·동일 장면 대조](issue-28_dos_comparison.md)

- [#30 DOS 객체 위상·실행 간격 보정](issue-30_dos_differences.md)

- [#31 G07-01 Digger 검증 결과](issue-31_digger.md)

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

- [#52 시간·구출률·승패 결과](issue-52_level_outcome.md)

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

- [#82 G10-06 통합 사운드 검증](issue-82_audio_playthrough.md)

- [#89 G11-03 특수4개 장면 연결 결과](issue-89_special_scenes.md)

- [#91 G11-04 전체120레벨 초기 재생 결과 기록](issue-91_initial_replay.md)

- [#94 튜토리얼7개 해법 결과](issue-94_solution_replay.md)

- [#96 Fun08~30 해법·반복·재시작 검증](issue-96_fun_solutions.md)

- [#98 Tricky01~30 해법·반복·재시작 검증](issue-98_tricky_solutions.md)

- [#100 Taxing01~30 해법·반복·재시작 검증](issue-100_taxing_solutions.md)

- [#102 Mayhem01~30 해법·반복·재시작 검증](issue-102_mayhem_solutions.md)

- [#104 COMP-01~18 증거 재감사](issue-104_comp_reaudit.md)

- [#106 G11-06 성능 기준](issue-106_performance.md): 로딩/틱/렌더/PCM380행·실제 SDL/Web60초×3·CI 분석, SDL 표시 주기 #107 후속.

- [#107 SDL 표시 주기 보정](issue-107_render_cadence.md): 60Hz 절대 시각·지연/복귀 테스트, 같은 호스트60초3쌍과 동일성 대조.

- [#110 G11 전체 증거·배포 진입 검토](issue-110_g11_closure.md): 기능/네이티브 범위 유효, Web 보완 #111로 G11/G12 미완료.

- [#111 Web 완료 마감·구간 메모리 관측](issue-111_web_deadlines.md): 계측/측정 완료, 무조회 완료 마감 초과2/0/1·#113 후속, G11/G12 미완료.

- [#113 Web 반복 그리기 보정·성능 검증 이월](issue-113_web_dispatch.md): 채택3쌍·trace/입력 진단, 사용자 승인으로 #115 G12 준비 착수.

- [#115 개발용 Linux·Web 패키지](issue-115_release_preparation.md): 허용 목록·대응 소스/체크섬·결정성·실제 압축 해제 실행과 CI artifact, Windows/정식 릴리스 후속.

- [#118 Windows 전체 게임 패키지](issue-118_windows_full_game.md): 원본 데이터 동봉·더블클릭 시작과 Windows 설치 검증.

- [#120 macOS 전체 게임 앱](issue-120_macos_full_game.md): universal2·원본 동봉·실제 ARM/Intel 검사와 서명/공증 조건.

- [#122 Linux·Web 전체 게임 패키지](issue-122_linux_web_full_game.md)

- [#124 버전·태그 관리와 첫 다운로드 릴리즈](issue-124_release_policy.md)

- [#126 GitHub Pages 플레이 배포](issue-126_github_pages.md)

- [#129 MIT 정책·제3자 고지 전달](issue-129_project_license.md)

- [#130 Windows 11·웹브라우저 메인테이너 인수](issue-130_maintainer_acceptance.md): 직접 테스트 성공 판정과 Windows 11 x64 지원·Windows 10 지원 제외 결정.
