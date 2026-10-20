# G10-04 게임 사운드 이벤트·DOS 숫자 ID

이슈: [#78](https://github.com/edwardkim/ohmylemmings/issues/78) · 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1)

게임 상태 전환·할당 피드백·시작/종료와 DOS 숫자 ID를 연결했습니다. SDL/Web 입력은 같은 기술 선택 helper를 사용하며 호스트가 여러 틱을 한 번에 실행해도 각 경계의 사건을 수집합니다. 음악 cursor·함정 raw ID·레벨 세대·큐 초과/오류 진단을 공유합니다.

[구현 계약·주소·재현 절차](../tech_insights/issue-78_game_sound.md), [출처/해시](../../assets/manifests/game_sound.json)를 참고합니다.

## 검증

- TDD: 이벤트 부재, 잘못된 수동 ID 가정, 실제 음악 retry 동작, UI burst 제한, 일방향 벽 오발음을 각각 실패로 확인한 뒤 수정했습니다. 보존된 실패 로그는 최종 통과 결과와 구별합니다.
- 원본 무수정 x86 분기97개·음악170조합을 직접 실행해 대조했습니다. Nuke OhNo·8개 기술 공통 ID4·일방향 벽 무음·전방 강철 거절18·frame10 경고를 포함합니다.
- 합성 게임98경로 native/실제 WASM trace와 두 배달 분할이 동일합니다. 6개 명령→PCM 229665프레임을 직접 스케줄·typed 적용·두 청크/실제 WASM으로 대조했습니다. trace SHA-256 `7fd0d4d1c324d868387b77e03c65b87e4d0dea0d22113527eb033cadb854b64c`.
- 실제 DOSBox 첫 음악 RAM ID6, 1712샘플/1435IRQ/20.532초, 69.889897Hz 관측. 원본50개 전후 해시 동일, EXE/DAT 수정 없음. 이는 장치 PCM/청취 비교가 아닙니다.
- actual WebGame WASM은 startup15/35/55·drain 시 상태/RGB 불변·repeat/focus/일시정지·1100요청→1024보관/76손실·종료Stop·성공/재시작 세대를 검사합니다.

전체 make check 종료0: Rust587/문서29/Python146, 포맷/Clippy·오프라인 빌드·독립 C/PCM·WASM·기존 전체 SDL/직업/판정/패널/포인터·Web7742상태/4456Canvas·통합656상태/57Canvas 및 독립12재생·WASM12재생1968상태/127RGB·actual game/audio WASM 통과. 로그 SHA-256 `672b01c42c290cc82d7d5be6227ae75c4101b8d08094223a7221e69ca624ca49`. 추가 원본 Fun01~03 Web 회귀는8648상태/4978Canvas를 통과했습니다. 신규 Rust24/Python5 회귀이며 자체 검토 미해결0입니다.

## 범위와 후속

이번 단계는 게임→숫자 명령→기존 PCM 연결입니다. SDL/Web 실제 장치 출력, 볼륨/음소거·사용자 활성화·숨김·LAN HTTP는 G10-05이며 실제 DOS/native/browser 통합 청취와 G10 마감은 G10-06입니다. 기존 OPL 엔진/외부 패키지/라이선스는 유지합니다. 임의 원본 완전 호환이나 비동기 IRQ 위상 일치를 주장하지 않습니다. 정확한 push/PR/main 네 CI 작업·ZIP checkout 로그·동일 트리 확인 후 이슈를 마감합니다.
