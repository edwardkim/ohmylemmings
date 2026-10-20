# G10-06 통합 사운드 검증

이슈: [#82](https://github.com/edwardkim/ohmylemmings/issues/82) · 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1)

기준 main `cce167e242c77aa1bcce345a1b4a7d3706569129`. #72~#80의 고정 자원·원본 명령·독립 C PCM·SDL/Web 장치 검증을 재사용한다. 이번에는 실제 DOS 녹음과 원본 레벨의 게임→음향 경로를 연결한다. 범위 밖 기능 추가와 기존 전체 검사의 반복을 피한다.

- [x] #82 등록·번호/URL 확인, main 동기화·작업 브랜치 생성.
- [x] 실제 DOS 자료의 오디오 스트림·원본 해시/설정 확인 및 재현 가능한 PCM/WAV 분석.
- [x] 원본 레벨에서 시작·음악·할당·Nuke/결과·재시작을 포함한 native/actual WASM 통합 비교.
- [x] 청취 자료·재생 절차·관측한 차이·미확인 항목 기록.
- [x] 변경 검사·PR/CI·이슈와 G10 상태 반영.
- [x] 2026-10-21 사용자 사운드 체크 통과 판정 수신.

원본/파생 PCM과 개별 로그는 `assets/local/issue-82-*`에만 둔다. HTTP8080과 CDP19222, 무관한 미추적 경로를 보존한다. 에뮬레이터 간 PCM 차이는 물리 OPL/스피커 청취나 동일 비동기 위상으로 확대 해석하지 않는다. 사용자 청취가 필요하면 준비된 자료를 먼저 제공하고 자동 검증을 계속한다.

합성/원본 각각580경계/1,824,750PCM·18맵/RGB의 native/실제 WASM 일치, 실제 DOS26초·SDL12초 녹음과 DRO4채널272사건/최대4.123ms를 확인했다. 로컬 Rust596/문서29/Python163 및 WASM/Web 검사, 신규 Python8개 TDD와 자체 검토를 통과했다. PR #83의 main 병합과 push·PR·main CI를 확인했다. 2026-10-21 사용자가 “사운드 체크 통과 판정입니다.”라고 확인하여 G10을 완료한다. 다음은 G11-01 전체 호환성·성능 검증 범위 정의다. [보고서](../reports/issue-82_audio_playthrough.md).
