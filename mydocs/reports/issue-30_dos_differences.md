# G06-03 DOS 객체 위상·실행 간격 보정 결과

이슈: [#30](https://github.com/edwardkim/ohmylemmings/issues/30), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

[기술 계약/재현](../tech_insights/issue-30_dos_differences.md) · [진행](../working/issue-30_dos_differences.md) · [일지](../orders/20261019.md#issue-30) · [증거 manifest](../../assets/manifests/dos_calibration.json).

## 결과

출구 장식의 위상을 보정하고 실제 시간의 호출 간격을 DOS 기준에 맞췄습니다. 코어의 17호출 게임 시계와 논리 상태/입력 규칙은 유지했습니다. 원본 EXE/50개·에뮬레이터·게임 메모리를 변경하지 않았으며 #29의 별도 런타임 seed 조건입니다.

| 검증 | 결과 |
|---|---|
| 원본 세 영상 전체 화면 | 0/35/54/60/100/170/340 일곱 장면 × 3실행, 전체 320×160에서 차이 0픽셀 |
| 객체 위상 | 초기 1회 증가와 DOS counter/native label 차이 1을 원본 코드·읽기 전용 폴링으로 확인, 연속 객체 +2 보정 |
| 타이머 | GUI PC2와 -x 경로 분리. 추가 8 DOS 실행, 제한된 메모리 관측 4개 |
| CPU 경계 | 7실행은 해당 경로의 영상 양자화 범위와 일치. GUI/5000 마지막 장면의 영상 1프레임 지연은 DIFFERENT로 보존 |
| 원본 기본 실행 | DOS pacing 340호출 24.433초/1174렌더, reference 17Hz는 20.228초/857렌더 |
| 시간/렌더 독립성 | 두 pacing의 모든 341상태와 기준 이미지 10개 동일. 렌더 변경·RR/일시정지·실제 X11 키 재생 회귀 성공 |
| 실제 SDL | 6실행/1780상태/63픽셀 캡처 + reference 실행 341상태/10픽셀 캡처 |
| TDD | 기존 행동 5 Red→데스크톱 22 Green; Python RAM 위치/분기/범위 판정 8개 추가, 전체 34 Green |
| 전체 검사 | Rust 단위/예제 304·문서 27·Python 34·기존 모델/참조 서명·SDL 4재생/1124상태/40캡처·WASM·포맷/Clippy 성공 |

F00 상태 SHA-256은 기존 `3e56870904094448fdeab59715ce9c7ad96ec25deb9ff34017cd24058d74d4fe`와 같습니다. 17Hz 참조 방식은 --pacing reference로 재현하며 원본 기본값은 dos-vga-5입니다. 전체 비교 그림: assets/local/issue-30-full-comparison-verified/run-1/pair-000170.png.

## 완료 범위

G06-03의 차이 원인 분리·보정·합성/원본 회귀를 완료했습니다. 게시·실제 CI 확인으로 G06-01~03과 **G06 전체를 완료**했습니다. 다음 후보는 G07-01 Digger입니다. 먼저 이슈를 등록하고 F01의 할당 경계/지형 변형·정상/실패·경계 사례를 검증합니다.

전체 DOS 내부 상태/입력 틱·다른 판본/하드웨어·키 디스크 무설정 기동·직업·트리거/구출/승패·음향·Windows 인증은 남습니다. 5000 cycles 관측 차이와 -x 경로를 기본 pacing의 보장 범위에 포함하지 않습니다.

## 게시 검증

구현 `becbbddd9f178a33c5e2792ee3aee77e7baaa4e6`를 main에 게시했습니다. [실제 push CI](https://github.com/edwardkim/ohmylemmings/actions/runs/39158078764)의 정확한 SHA·이벤트·세 작업/초기 clean-input·SDK/잠금 의존 준비와 실제 로그/서명을 확인했습니다.

- Rust native: Rust 단위/예제 304·문서 27, 기존 참조/모델·실제 SDL 4재생/1124상태/40캡처 success.
- Python tooling: Python 3.12.15, unittest 34·독립 모델 두 서명 success. 원본/DOSBox/PyAV 없이 동작합니다.
- WASM libraries: 기존 두 no_std 라이브러리 잠금 의존 준비·오프라인 컴파일 success.

원격 197개 파일 모드/blob이 로컬과 일치하고 작업 트리는 깨끗했습니다. 원본/다운로드 바이너리/원본 파생 영상·화면·전체 폴링/추적은 추적하지 않습니다. 471개 증거 파일 해시와 문서/링크/일지 규칙도 확인했습니다. 최종 문서 게시·CI·상위 #1 G06 체크·#30 종료는 GitHub 기록에서 추적합니다.
