# #151 HD-04 실제 Fun01 Native/Web 통합

이슈: https://github.com/edwardkim/ohmylemmings/issues/151 · 상위 [#137](issue-137_2d_remaster.md)

main f5cefc9와 이슈 OPEN/번호·URL을 확인하고 별도 worktree에서 착수했다. [아트 계약](issue-138_hd_art_contract.md), [표시 계약](issue-140_hd_presentation.md), [팩 결과](../reports/issue-147_hd_actions.md)를 따른다.

1. 공통 game HD 모듈에 자산/원본 fallback atlas·텍스처 revision·지형 chunk cache·사각형/텍스트/복사 명령을 구현한다. 실제 visible mask/brick·시설 draw flags/phase를 사용하고 큰 HD 지형은 변경 chunk만 다시 만든다. snapshot과 동일한 읽기 전용 경계로 simulation/시계/사운드를 바꾸지 않는다.
2. Native SDL과 Web Canvas 호스트가 같은 명령/텍스처를 표시한다.1920×1080 비율 유지·게임1920×960/패널120, 원본 모드와 명시적 HD 미리보기 선택·전체화면/resize를 연결한다. 원본 모드 경로/기본 캡처·벤치마크 계약을 보존한다.
3. 공통 HD hit→기존 논리 좌표/패널 의미의 변환으로8직업·속도/Pause/Nuke/미니맵/재시작을 연결한다. 기존 held rate/두 번 클릭/우클릭/재생 guard를 재사용하고 여백/잘못된 좌표·모드 전환 시 held 입력을 검사한다.
4. HD PNG가 없는 상태는 원본 스프라이트를 명시적으로 사용하고 coverage/진단과 UI 안내에 드러낸다. 새 자산이 없는 것을 박스로 대체하지 않는다. 실제 Fun01/합성 저장 입력의 걷기·굴착·계단·폭발·입출구·패널/원본모드 전환을 Native/Web에서 확인한다. 원본/HD 논리·mask·입력·사운드가 동일한지 검증한다.
5. 실패 테스트 뒤 구현하고 실제 화면/입력·변경 구간 캐시·누락/손상과 로컬/CI7개·self-review·병합/문서 근거를 기록한다. Web/Native 개발 빌드에 팩 준비 명령과 파일을 연결하지만 공개 v0.1.0/Pages를 배포하지 않는다.

저사양 호스트의60fps 장기 반복은 하지 않는다. #113 이월/NOT READY·#142 Chrome156 진단·#1 CLOSED와 Windows11 지원 정책을 유지한다. 전체337프레임/5테마/특수 장면·효과·새 공개 패키지/전체화면 시각 인수는 별도 범위를 정확히 기록한다.
