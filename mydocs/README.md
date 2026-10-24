# Hyper-Waterfall 문서 안내

[#208 Native MP3 선택 대화상자·앱 아이콘](reports/issue-208_native_polish.md)을 포함한 **v0.3.1 정식 릴리즈·Pages**를 게시했습니다. [릴리즈](https://github.com/edwardkim/ohmylemmings/releases/tag/v0.3.1)·[웹 플레이](https://edwardkim.github.io/ohmylemmings/)를 제공합니다. 동일 tree CI7개·4플랫폼/12파일과 Windows Shell/Mac ICNS·실제 Native 파일 선택을 검증했습니다. [#202 로컬 MP3](reports/issue-202_local_mp3.md)·[2D HD](plans/issue-137_2d_remaster.md)·[점진적 다운로드/진행률](reports/issue-196_web_progressive.md)도 포함합니다. #202 새 MP3 사람 청취 응답 OPEN·#113 성능 NOT READY를 유지하며 아래 단계별 문구는 당시 기록입니다.

프로젝트 기획은 [전체 기획서 / #1](plans/issue-1_rusty_lemmings.md), 기여 지침은 [AGENTS.md](../AGENTS.md)를 기준으로 합니다.

단계별 구현 목표와 작업 순서는 [전체 구현 계획](plans/issue-1_implementation_plan.md)을 참고합니다. 2026-10-22 메인테이너 지시로 [#1 초기 구현·첫 플레이 배포를 종료](reports/issue-1_initial_development_closure.md)하고, 남은 성능·구간 메모리는 #113에서 계속합니다. 아래 단계별 기록은 각 검증 시점의 상태입니다.

G00-01의 [DOS 기준판 식별·원본 확보](tech_insights/issue-2_dos_original_baseline.md)를 완료했습니다. 로컬 데이터 준비는 [리소스 안내](../assets/README.md)를 참고합니다.

[G00-02 레퍼런스 코드 지도](tech_insights/issue-3_reference_code_map.md)와 [G00-03 호환성 검증 기준](tech_insights/issue-5_compatibility_baseline.md)을 완료했습니다. [G01-01 Rust 개발 골격](plans/issue-6_rust_workspace.md)과 [G01-02 CI·개발 검증](plans/issue-7_continuous_integration.md)을 마쳐 G01을 완료했습니다. [G02-01 DAT 구간 읽기](tech_insights/issue-8_dat_sections.md)와 [G02-02 압축 해제](tech_insights/issue-9_dat_decompression.md)의 테스트·원본 해시·실제 CI 검증으로 G02도 완료했습니다. [G03-01 기본 레벨 레코드](tech_insights/issue-10_level_records.md) 이후 [G03-02 ODDTABLE·카탈로그](tech_insights/issue-11_level_catalog.md)도 마쳤으며 [G03-03 GROUND 메타데이터·팔레트](tech_insights/issue-12_ground_metadata.md)도 합성 테스트·원본 5개 전체 필드 대조·실제 CI를 통과해 완료했습니다. [G03-04 VGAGR 비트플레인·마스크](tech_insights/issue-13_vgagr_images.md)도 합성 테스트·원본 867개 이미지/프레임의 전체 픽셀 대조·실제 CI를 통과해 완료했습니다. [G03-05 VGASPEC 특수 배경](tech_insights/issue-14_vgaspec_backgrounds.md)도 합성 테스트·원본 4개 전체 픽셀 대조·실제 CI를 통과해 완료했습니다. [G03-06 초기 지형 합성](tech_insights/issue-15_level_terrain.md)도 120개 전체 맵·팔레트 대조·로컬 검사·실제 CI를 통과해 완료했습니다. [G03-07 MAIN.DAT 레밍 스프라이트](tech_insights/issue-16_lemming_sprites.md)도 전체 프레임 대조·로컬 검사·실제 CI를 통과해 완료했습니다. [G03-08 MAIN.DAT UI·폰트](tech_insights/issue-17_main_ui.md)도 지원 이미지/글리프 전체 대조·로컬 검사·실제 CI를 통과해 완료했습니다. [G03-09 MAIN.DAT 마스크·카운트다운](tech_insights/issue-18_main_masks.md)도 전체 비트 대조·로컬 검사·실제 CI를 통과해 완료했습니다. G03-01~10의 검증·게시·실제 CI로 G03 전체 완료 기준을 충족했습니다. [G03-10 객체 이미지 배치·합성](tech_insights/issue-19_level_objects.md)도 전체 객체 대조·게시·실제 CI를 통과했습니다. [G04-01 픽셀 지형 읽기·경계](tech_insights/issue-20_terrain_mask.md)도 합성 전수 대조·게시·실제 CI를 통과해 완료했습니다. G04의 읽기·셀 쓰기·영역 변형을 모두 검증했고 [G04-02 지형 쓰기·셀 변경](tech_insights/issue-21_terrain_edits.md)도 전수 대조·게시·실제 CI를 통과해 완료했습니다. [G04-03 영역·마스크 기반 지형 추가/삭제](tech_insights/issue-22_terrain_bulk.md)도 전수 대조·게시·실제 CI를 통과했습니다. G04 전체 완료 기준을 충족했습니다. [G05-01 고정 틱·입력 순서 골격](tech_insights/issue-23_fixed_ticks.md)도 합성 분할/재생·게시·실제 CI를 통과해 완료했습니다. [G05-02 시작 시계·입구·생성 스케줄](tech_insights/issue-24_startup_schedule.md)도 합성 경계/재생·게시·실제 CI를 통과해 완료했습니다. [G05-03 개체 이동·제거](tech_insights/issue-25_actor_motion.md)도 전수/통합 재생·게시·실제 CI를 통과해 완료했습니다. [#26 상태 추적·재생·기준 실행 대조](tech_insights/issue-26_state_replay.md)의 실제 기준 함수 실행·합성/원본 대조·게시·CI를 확인해 G05 전체를 완료했습니다. 이후 G06 네이티브 화면·DOS 대조/보정도 완료했습니다. 전체 게임 호환성은 후속 목표에서 검증합니다.

[#27 G06-01 네이티브 화면·입력](tech_insights/issue-27_native_view.md)의 실제 X11/키/화면·게시·CI를 확인해 완료했습니다. 후속 #28의 DOS 실행·#30의 전체 장면/속도 보정을 거쳐 G06 전체를 완료했습니다.

## 폴더 구조

- [plans/](plans/README.md): 마일스톤 및 아키텍처 계획
- [orders/](orders/README.md): 날짜별 작업일지와 이슈별 작업 목록
- [working/](working/README.md): 진행 중 작업 상태와 이어서 작업할 컨텍스트
- [reports/](reports/README.md): 작업 결과와 검증 보고서
- [releases/](releases/README.md): 버전별 플레이 배포 변경 내역·실행 안내
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

[#28 G06-02 실제 DOS 세 실행·장면 대조](reports/issue-28_dos_comparison.md)를 완료했습니다. 상부 일치에서 발견한 전체 화면·속도 차이는 [#30 보정](tech_insights/issue-30_dos_differences.md)과 [게시·CI](reports/issue-30_dos_differences.md)로 마무리했습니다. G00~G06과 Digger·Basher·Miner를 완료했으며 [G07-04 Builder / #34](reports/issue-34_builder.md)의 병합·CI도 완료했습니다. [G07-05 Blocker / #36](reports/issue-36_blocker.md)의 PR #37 병합·main CI·종료를 확인했습니다. [G07-06 Bomber / #38](reports/issue-38_bomber.md)의 PR #39 병합·main CI·종료와 사용자 E키 확인을 마쳤습니다. [G07-07 Climber / #40](reports/issue-40_climber.md)의 PR #41 병합·main CI·종료를 확인했습니다. [G07-08 Floater / #42](reports/issue-42_floater.md)의 PR #43 병합·main CI·종료를 확인했습니다. [G07-09 Nuke / #44](reports/issue-44_nuke.md)의 PR #45 병합·main CI·종료를 확인했습니다. [G07-10 / #46](reports/issue-46_skill_interactions.md)의 할당/혼합 회귀도 완료해 G07 직업·지형 목표의 기준을 충족했습니다. [G08-01 / #48](reports/issue-48_exit_rescue.md)의 출구·구출을 연결했습니다. [G08-02 / #50](reports/issue-50_hazards.md)의 물·불·함정도 연결했습니다. [G08-03 / #52](reports/issue-52_level_outcome.md)의 시간·구출률·승패 판정을 연결했습니다. [G08-04 / #54](reports/issue-54_results_restart.md)에서 결과 화면·동일 레벨 재시작·다음 레벨 전환을 연결했습니다. [#56 기술 패널·미니맵](reports/issue-56_skill_panel.md)을 통합했습니다. [#58 포인터 선택·RightClickGlitch](reports/issue-58_pointer_selection.md)의 병합·실제 CI를 확인했습니다. [#60 한 레벨 통합 완주](reports/issue-60_native_playthrough.md)의 실제 SDL 성공·실패·재시작/전환·독립 재생과 DOS 결과/장면 비교를 검증했습니다. 정확한 게시 CI·G08 완료 처리는 #60에서 추적하며 다음은 G09 웹의 동일 코어 경계입니다.

[#31 Digger](reports/issue-31_digger.md)의 구현·기준/원본 검증·게시/실제 CI를 완료했습니다. Basher와 Miner도 완료했으며 [G07-04 Builder / #34](reports/issue-34_builder.md)의 병합·CI도 완료했습니다. [G07-05 Blocker / #36](reports/issue-36_blocker.md)의 PR #37 병합·main CI·종료를 확인했습니다. [G07-06 Bomber / #38](reports/issue-38_bomber.md)의 PR #39 병합·main CI·종료와 사용자 E키 확인을 마쳤습니다. [G07-07 Climber / #40](reports/issue-40_climber.md)의 PR #41 병합·main CI·종료를 확인했습니다. [G07-08 Floater / #42](reports/issue-42_floater.md)의 PR #43 병합·main CI·종료를 확인했습니다. [G07-09 Nuke / #44](reports/issue-44_nuke.md)의 PR #45 병합·main CI·종료를 확인했습니다. [G07-10 / #46](reports/issue-46_skill_interactions.md)의 할당/혼합 회귀도 완료해 G07 직업·지형 목표의 기준을 충족했습니다. [G08-01 / #48](reports/issue-48_exit_rescue.md)의 출구·구출을 연결했습니다. [G08-02 / #50](reports/issue-50_hazards.md)의 물·불·함정도 연결했습니다. [G08-03 / #52](reports/issue-52_level_outcome.md)의 시간·구출률·승패 판정을 연결했습니다. [G08-04 / #54](reports/issue-54_results_restart.md)에서 결과 화면·동일 레벨 재시작·다음 레벨 전환을 연결했습니다. [#56 기술 패널·미니맵](reports/issue-56_skill_panel.md)을 통합했습니다. [#58 포인터 선택·RightClickGlitch](reports/issue-58_pointer_selection.md)의 병합·실제 CI를 확인했습니다. [#60 한 레벨 통합 완주](reports/issue-60_native_playthrough.md)의 실제 SDL 성공·실패·재시작/전환·독립 재생과 DOS 결과/장면 비교를 검증했습니다. 정확한 게시 CI·G08 완료 처리는 #60에서 추적하며 다음은 G09 웹의 동일 코어 경계입니다.

[#32 Basher](reports/issue-32_basher.md)의 구현·기준/원본/네이티브·전체 회귀·게시/실제 CI를 완료했습니다. G07 직업·지형 회귀를 완료했으며 G08 객체·완주는 후속입니다.

[#33 Miner](reports/issue-33_miner.md)의 구현·기준/원본/네이티브·전체 회귀·게시/실제 CI를 완료했습니다. G07 직업·지형 회귀를 완료했으며 G08 객체·완주는 후속입니다.

G08은 #60/PR #61과 main CI까지 확인해 완료했습니다. [G09-01 / #62 공유 세션·웹 이식 경계](tech_insights/issue-62_shared_session.md)에서 후속 웹 작업을 분할합니다.

[G09-02 / #64 자산 바이트·공통 장면과 패널](tech_insights/issue-64_shared_game.md)을 진행합니다. JS/WASM·Canvas·브라우저 완주는 후속 단계입니다.

[G09-03 / #66 JS/WASM·합성 브라우저](tech_insights/issue-66_wasm_browser.md)에서 원본 없는 실제 WASM과 네이티브/브라우저 상태·픽셀 대조를 연결합니다. 원본 공급/입력·같은 레벨 브라우저 완주는 G09-04/05 후속입니다.

[G09-04 / #68 브라우저 입력·원본 자산·레벨 전환](tech_insights/issue-68_web_input_assets.md)을 연결합니다. 브라우저의 원본 한 레벨 통합 완주는 G09-05에서 검증합니다.

[G09-05 / #70 브라우저 통합 완주](reports/issue-70_web_playthrough.md)에서 원본 Fun01 성공 두 번·Nuke 실패·재시작/다음 Fun02를 매 제어 경계/맵/Canvas 및 독립 네이티브 재생과 대조했습니다. G09 기능 기준을 충족했으며 정확한 게시 CI·G09 마감은 #70/상위 #1에서 추적합니다. 다음은 G10 사운드 통합입니다.

[G10-01 / #72 AdLib 자원·통합 계약](tech_insights/issue-72_adlib_catalog.md): no_std21곡/18효과음과120레벨 대응·합성/원본 전체 목록을 검증하고, 명령 스트림·합성기·게임 이벤트·SDL/Web 출력을 후속 절편으로 분리합니다. G10 전체는 진행 중이며 다음은 G10-02입니다.

[G10-02 / #74 ADLIB 명령·레지스터 스트림](tech_insights/issue-74_adlib_stream.md): 원본 드라이버 직접 실행과21곡/18효과음·반복/정지/재시작을 호출 단위로 대조합니다. 다음은 G10-03 OPL/PCM이며 G10 전체는 진행 중입니다.

[G10-03 / #76 OPL·결정론 PCM](reports/issue-76_opl_pcm.md): 엔진/라이선스 고정·원본 호출 시계·유리수 샘플/청크·native/WASM 및 독립 C 비교를 검증합니다. 다음 G10-04 게임 이벤트/DOS ID, G10 전체는 진행 중입니다.

[G10-04 / #78 게임 사운드·DOS ID](reports/issue-78_game_sound.md): typed 사건·시작/종료/음악 cursor·함정 raw ID·세대 큐와 native/실제 WASM PCM 연결을 검증합니다. 다음은 G10-05 장치 출력이며 G10 전체는 진행 중입니다.

[G10-05 / #80 SDL/Web 오디오 출력](reports/issue-80_audio_output.md): 장치·사용자 활성화·음소거/볼륨·숨김/재개·원본/연습 음원과 LAN HTTP, 라이선스/재빌드 소스 배포를 연결합니다. 다음은 G10-06 통합 대조·청취/마감입니다.

[G10-06 / #82 통합 사운드](reports/issue-82_audio_playthrough.md): 실제 DOS 녹음/OPL·SDL 출력과 원본 native/WASM PCM 대조, PR #83 병합·CI와 2026-10-21 사용자 사운드 체크 통과 판정으로 G10을 완료했습니다. 다음은 G11 전체 호환성·성능 검증의 범위 정의입니다.

[G11-01 / #85 전수 점검·성능 기준](plans/issue-85_compatibility_validation.md): 120레벨·8직업·21곡/18효과음·COMP-01~18의 기존 증거·소스 제한·미검증과 측정 절차를 구분했습니다. G11 전수 실행은 후속이며 다음 구현은 G11-02 난이도/레벨 ID 연결입니다.

[G11-02 / #87 네 난이도 선택·전환](tech_insights/issue-87_rank_selection.md): 공통 로더·SDL/Web에 LevelId를 연결하고 Fun 호환 입력을 유지합니다. 일반116개는 소스 범위이며 특수4장면과 전수 완주/성능 검증은 후속입니다. 현재 검증·게시 상태는 #87에서 추적합니다.

[G11-03 / #89 특수4개 장면](reports/issue-89_special_scenes.md): VGASPEC을 공통 로더·SDL/Web에 연결하고 특수4개 호스트/정적 증거를 대조했습니다. 전체120개 초기 재생·완주·성능은 G11 후속입니다. 게시/병합 CI·종료는 #89에서 추적합니다.

[G11-04 / #91 전체120레벨 초기 재생](reports/issue-91_initial_replay.md): 실제 SDL·WASM의340경계·3회/시간 분할·맵/전체 프레임과 독립 초기 자원을 대조합니다. 전수 실행·정확한 CI/마감 상태는 #91에 기록하며 유효 해법 완주·성능은 후속입니다.

[G11-05 #93](working/issue-93_solution_validation.md)의 첫 절편 [#94](reports/issue-94_solution_replay.md)은 Fun01~07 유효 입력·실제 SDL/WASM 반복/재시작을 검증합니다. 전수120개·COMP 마감은 #93에서 계속하며 최종 실행/CI는 각 이슈 기록을 따릅니다.

[G11-05b #96 Fun08~30](reports/issue-96_fun_solutions.md)은 성공23개/실패23개 저장 입력과 실제 SDL/WASM 반복·Restart를 검증합니다. 최종 실행/CI/병합은 #96, 다른90개와 COMP/성능은 #93에서 추적합니다.

[G11-05c #98 Tricky01~30](reports/issue-98_tricky_solutions.md)은 성공30개/실패30개 입력(자체 작성·공개 리플레이 변환 출처 구분)과 실제 SDL/WASM 반복·Restart를 검증합니다. 최종 실행/CI/병합은 #98, 다른60개와 COMP/성능은 #93에서 추적합니다.

[G11-05d #100 Taxing01~30](reports/issue-100_taxing_solutions.md)은 성공30개/실패30개 입력(자체7·공개 변환23)과 실제 SDL/WASM 반복·Restart를 검증합니다. 최종 실행/CI/병합은 #100, Mayhem30개·COMP/성능은 #93에서 추적합니다.

[G11-05e #102 Mayhem01~30](reports/issue-102_mayhem_solutions.md)은 성공30개/실패30개 입력(자체8·공개 변환22)과 실제 SDL/WASM 반복·Restart를 검증합니다. 최종 판정은 #102, COMP 전체 재감사·성능/G11/G12는 #93에서 추적합니다.

[G11-05f #104 COMP 재감사](reports/issue-104_comp_reaudit.md)는 기존 경계 증거의 현재 소스 연결·빠진 원시 자료 보충·초기120개와 성공/실패240사례의 집계를 수행합니다. 최종 판정·정확한 CI/병합은 #104 기록을 따르며 성능은 G11-06 후속입니다.

[G11-06 #106 성능 기준 측정](reports/issue-106_performance.md)은 알고리즘380행과 실제 SDL/Web 각60초×3을 기록했습니다. SDL 약53.2fps의60Hz 목표 미달은 [#107](https://github.com/edwardkim/ohmylemmings/issues/107)에서 보정하며 G11 전체 마감/G12는 후속입니다.

[#107 SDL 표시 주기 보정](reports/issue-107_render_cadence.md)은 렌더 처리 시간을 다음 대기에 더하던 방식을60Hz 절대 시각으로 바꿉니다. 실제 전후60초3쌍과 동일성·CI 결과는 보고서/#107에 고정하며 G11 전체/G12 마감은 후속입니다.

[#110 전체 검토](reports/issue-110_g11_closure.md)에서120개/COMP·Native 보정 증거를 재확인했습니다. Web 콜백 예산 초과3건·완료 마감/구간 메모리 증거 부족은 [#111](https://github.com/edwardkim/ohmylemmings/issues/111)에서 보완하며 G11과G12는 미완료입니다.

[#111 Web 관측 보완](reports/issue-111_web_deadlines.md)에서 rAF 시각/완료와 구간 메모리 표본을 확보했습니다. 무조회60초3회의 마감 초과2/0/1과 추가 draw는 [#113](https://github.com/edwardkim/ohmylemmings/issues/113)에서 진단하며 G11/G12는 미완료입니다.

현재 [#113 Web 보정·측정](reports/issue-113_web_dispatch.md)은 남은 성능 검증을 더 좋은 호스트에 이월했다. 사용자 승인으로 [#115 배포 준비](plans/issue-115_release_preparation.md)와 v0.1.0/Pages 플레이 배포를 완료했다. 성능 합격이나 G12 전체 완료를 뜻하지 않는다.

[#115 배포 준비](reports/issue-115_release_preparation.md)는 Linux/Web 개발 패키지부터 Windows/macOS/Linux/Web 전체 게임·대응 소스/해시·공개 릴리즈와 Pages까지 완료했다. [#129](reports/issue-129_project_license.md)에서 자체 코드 MIT·제3자 고지 정책을 적용했다. [#130](reports/issue-130_maintainer_acceptance.md)은 메인테이너의 Windows 11·웹브라우저 직접 테스트 성공 판정을 받았다. Windows 지원 대상은 Windows 11 x64이며 Windows 10은 지원하지 않는다. 남은 고성능 호스트 성능은 #113에서 추적한다.

버전·태그·게시 운영은 [#124 릴리즈 정책](plans/issue-124_release_policy.md)을 따릅니다.

[#147 HD-03b 직업·시설·패널 자산](reports/issue-147_hd_actions.md)에서 새65캐릭터 프레임·첫 테마 입출구11프레임·13조작 아이콘과 typed 팩을 준비했다. 현재 후보는7atlas106자산/81캐릭터 프레임이며 실제 HD 게임/GPU·Fun01 통합은 [#151](reports/issue-151_hd_integration.md), 전체 자산 확장은 #137 후속이다.

[#154 HD-05a 캐릭터 전체 프레임](reports/issue-154_hd_actors.md)에서 남은21동작256포즈를 연결해 전체28/337 캐릭터 키와 Native/Web 표시를 검증하고 PR #155·CI7개 SUCCESS·실제 병합으로 완료했다. 다른 테마/시설/효과/글꼴과 지원 HD 배포는 후속이다.

[#157 HD-05b 일반 지형](reports/issue-157_hd_terrain.md)을 PR #158·CI7개 SUCCESS·실제 병합으로 완료했다. 일반5재질·원본 일방통행 화살표를 연결하고 팩을119.5→109.5MB로 정리했다.120초기 장면/선택1,920틱·Native/Web 동일성을 검증했으며 특수4장면/시설/효과/전체 UI·지원 HD 배포는 후속이다.

[#175 물·불 HD](reports/issue-175_hd_hazards.md)는8PNG·9객체84프레임, 원본 판정/주기 보존과 실제 Native/Web 검증·PR #176 CI7개·병합을 완료했습니다. 다음은 기계 함정·나머지 효과/글꼴/전체 UI와 HD 배포/인수입니다.

[#178 기계 함정 HD](reports/issue-178_hd_traps.md)는9PNG·10종159프레임과 원본 발동 상태 보존·Native/Web 검증·PR #179 CI7개·실제 병합을 완료했습니다. 다음은 횃불/나머지 장식·글꼴/전체 UI와 지원 HD 배포/인수입니다.

[#181 깃발·일방통행 표시 HD](reports/issue-181_hd_decor.md)는2PNG/20종212키·원본 방향 제한/지형 보존과 Native/Web 검증·PR #182 CI7개·실제 병합을 완료했습니다. 원본54객체/540키를 갖췄으며, 다음은 글꼴/전체 UI와 지원 HD 배포/인수입니다. BGM #165는 리마스터 이후입니다.

[#200 사용자 K-pop BGM 교체 검토](reports/issue-200_custom_bgm_review.md): Web/Native에서 외부 음악 교체 가능. 첫 구현은 사용자 로컬 MP3 선택을 권고하며, 원본 BGM/효과음 분리·음악 버퍼링/UX와 공개 음원 사용 권한을 검토했다. 제품 구현은 후속 작업이다.

[#202 로컬 MP3 BGM](reports/issue-202_local_mp3.md)은 main 개발 버전에 구현·CI7개·병합을 완료했습니다. Web 파일 선택과 Native 드롭/실행 옵션, 효과음 유지/개별 음량·반복·오류/재시도를 제공합니다. 제공받은 두 파일의 실제 디지털 출력을 확인했으며 메인테이너의 새 MP3 청취 응답을 기다립니다. [v0.3.0/Pages 게시](reports/issue-205_release_v030_completion.md)까지 완료했습니다.

[#205 v0.3.0 정식 릴리즈·Pages](reports/issue-205_release_v030_completion.md)를 완료했습니다. [버전별 노트](releases/issue-205_v0.3.0.md)는 사용자 로컬 MP3와 실행/인수 범위를 안내합니다.

[#208 v0.3.1 Native 편의 개선·배포](reports/issue-208_native_polish.md)를 완료했습니다. F4/Ctrl+O/Cmd+O 선택·취소/모달 처리·Windows EXE/Mac/SDL 아이콘을 제공합니다.
