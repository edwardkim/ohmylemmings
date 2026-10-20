# G10-06 통합 사운드 검증 결과

이슈: [#82](https://github.com/edwardkim/ohmylemmings/issues/82) · 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1)

기준 main `cce167e242c77aa1bcce345a1b4a7d3706569129`의 게임·OPL·장치 경로를 새 도구로 실행했다. 원본/PCM/영상·개별 로그는 ignored 경로에, 공개 자료는 합성 코드·수치·해시에 보관한다.

| 검사 | 결과 |
| --- | --- |
| 합성·원본 통합 | 각각580경계/1,824,750PCM프레임·18개 맵/RGB. 모든 게임/사건/오디오 상태와 두 청크 분할 일치 |
| 원본 lifecycle | 시작·음악6·선택·Digger·Nuke OhNo/폭발·Lost/Stop·세대1→2 재시작 |
| 실제 DOS | 50원본 불변, AdLibFF·음악6·약69.890621IRQ/s, 게임 녹음25.967982초 nonzero |
| 실제 DRO | 23.099초/1,434쓰기.4채널272사건 일치, 최대 상대 오차4.122914ms |
| 실제 SDL | 소유 스트림의 전용 sink monitor.11.888617초/524,288프레임 중419,057nonzero. 임시 장치 정리 |
| TDD | 신규 Python8개: PCM/WAV·좌채널 보존·시계/세대/손실·DRO 손상/지연/주파수 불일치. Red→Green |
| 로컬 검사 | `make check-rust check-python check-wasm check-web` 종료0. Rust596/문서29; DRO 추가 뒤 Python163 별도 재확인 |

변경 없는 SDL 전체 회귀는 #80 main CI를 재사용하고 게시 CI에서 다시 실행한다. 로컬 전체 `make check` 실행으로 집계하지 않는다. 이전 전곡/효과음·독립 C·장치/활성화·LAN·라이선스 근거도 재사용한다. [로그/선정 파일 해시](../../assets/manifests/audio_playthrough.json), [재현과 계약](../tech_insights/issue-82_audio_playthrough.md). 자체 검토·정확한 게시/병합 CI는 #82에서 갱신한다.

DOSBox DBOPL/믹서와 Nuked의 음량·음색/파형 차이는 남아 있다. RMS 비교를 동등성으로 승인하지 않았으며 실제 OPL로 첫 음악의 주파수·템포를 따로 검증했다. [청취 페이지](http://192.168.2.146:8080/listening/)를 제공한 뒤 **2026-10-21 사용자의 “사운드 체크 통과 판정입니다.” 확인을 수신했다.** 자동 검증과 사용자 청취 검증을 통과하여 #82와 G10을 마감한다. 다음은 G11 전체 호환성·성능 검증의 범위 정의다. 이 청취 판정을 물리 DAC 계측·전곡 전구간 검사·DOS와의 파형 또는 절대 위상 동일성으로 확대하지 않는다.
