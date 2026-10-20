# G10-03 OPL·결정론 PCM 결과

이슈: [#76](https://github.com/edwardkim/ohmylemmings/issues/76) · 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1)

원본 드라이버 쓰기를 정수 PCM으로 변환하는 no_std·무할당 라이브러리 두 개와 별도 native/WASM 검증 어댑터를 추가했습니다. [시간/API/선택 근거](../tech_insights/issue-76_opl_pcm.md) · [출처/해시](../../assets/manifests/adlib_pcm.json).

LGPL-2.1-or-later Nuked-OPL3 Rust 코어를 버전/소스 해시로 고정하고 OPL2 인터페이스로 제한했습니다. 전체 라이선스·저작권·수정 patch·선호 소스·재링크 설명을 포함합니다. 정수 합성 순서는 독립 C1.8과 샘플 전체 대조했습니다. 먼저 평가한 oplon은 원본 음악의 음량 차이를 확인해 선택에서 제외했으며 탐색 결과를 성공 증거로 집계하지 않았습니다.

| 검사 | 범위 | 결과 |
| --- | --- | --- |
| Rust 신규 회귀13개 | OPL2개·PCM10개·무출력 VM1개 | 샘플률/유리수 위상·임의 청크·후반 오류 원자성·새 초기화·경계 제어·헤더 factor 독립 |
| Python 신규 회귀13개 | PCM/브라우저7·원본 시계3·C 기준3 | 손상/누락/순서/타입·원본 경로·카운터 wrap/게임 budget reset·기준 소스 해시 |
| 합성 native+actual WASM | 6샘플률×2시계×1/8음성=24개, 1,225,880프레임 | 고정 합성 fixture/direct/스트리밍2분할/브라우저2분할 SHA 일치 |
| 선택적 원본 native+WASM | 합성24+21곡/18효과음 각3초22050Hz=63개, 3,805,730프레임 | native189회 전달과 WASM126회 전달 일치 |
| 독립 C1.8 | 4waveform×6샘플률+21곡/18효과음 각2초44100Hz=63개, 6,942,328프레임 | Rust/C PCM 모든 바이트 일치 |
| 원본 IRQ dispatch 직접 실행 | 5모드 각512IRQ | AdLib mask1/2/3 각512, speaker256, muted0 |
| 실제 DOSBox VGA+AdLib | 1659RAM샘플·19.889초·1390IRQ | 약69.887511Hz, VGA2190197/31250, AdLib선택FF |

원본50파일은 각 원본 검사 전후 동일합니다. 원본·파생 PCM·영상·런타임 seed·선택적 에뮬레이터는 ignored assets/local에 보관하며 공개 자료에는 출처·해시·검증 수치만 기록합니다. C 기준 소스는 별도의 LGPL 공개 소스이며 검증 전용 subprocess입니다. 기존 desktop/web 제품·게임 이벤트·시뮬레이션/렌더/입력 흐름은 바꾸지 않았습니다.

전체 make check 종료0: Rust563/문서29/Python141·포맷/Clippy·오프라인 빌드·합성 독립 C/PCM·WASM·전체 SDL/기술/판정/패널/포인터·기존 브라우저7742상태/4456Canvas·통합 여정656상태/57Canvas·독립12재생 및 WASM12재생1968상태/127RGB·actual audio WASM을 통과했습니다. 로그 SHA-256 `d396c858500e4f0c6eb4c84812d62e8436205c519beff0b342cdf5944f4fe54e`. self-review에서 엔진/라이선스·원본 시간/카운터·정수 경계/오류 원자성·독립 기준·비공개 보관·CI 재현성을 확인했고 미해결 항목은 없습니다.

정확한 push/PR/main 네 작업 CI·ZIP 전체 로그/checkout·같은 트리/파일 검증과 이슈 마감은 #76에서 추적합니다. UI 변경이 없어 새 UI 스크린샷은 없습니다. 원격 HTTP8080/방화벽 허용과 CDP19222 loopback을 유지합니다.

G10-03 다음은 G10-04 게임 이벤트·DOS 숫자ID 연결입니다. G10 전체는 진행 중이며 실제 SDL/Web 오디오 장치·활성화/음소거·청취와 통합 검증은 G10-05/06에 남습니다.
