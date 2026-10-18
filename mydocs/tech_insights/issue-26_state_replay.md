# G05-04 상태 추적·재생 계약

작업 이슈: [GitHub #26](https://github.com/edwardkim/ohmylemmings/issues/26). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

- 선행: [#5 비교 계약](issue-5_compatibility_baseline.md), [#23 고정 틱](issue-23_fixed_ticks.md), [#24 시작 스케줄](issue-24_startup_schedule.md), [#25 이동](issue-25_actor_motion.md).
- [진행 기록](../working/issue-26_state_replay.md), [보고서](../reports/issue-26_state_replay.md), [작업일지](../orders/20261019.md#issue-26).

## 구조와 지원 범위

`lemmings-core/examples/trace_replay.rs`는 기존 무할당 no_std 라이브러리를 조합하는 std 개발 어댑터입니다. JSON·파일·할당·해시는 코어 밖에 둡니다. 재생은 초기 상태에서 전체 입력 목록을 다시 적용하며 중간 저장 상태 복원·Restart를 제공하지 않습니다.

`tools/state_replay.py`는 `format=dos-compat-trace-v1`, **`scope=g05-basic-v1`**의 구조화 부분집합을 기록합니다. 전체 #5 필드 계약의 완료를 의미하지 않습니다. 지원 필드는 다음과 같습니다.

- 초기 헤더: 프로필·레벨/설정·17Hz·발 좌표계·정렬된 원본 파일 크기/SHA-256·입력 목록 SHA-256·엔진/컴파일러/어댑터 출처.
- 정적 지형: 폭·높이·행 우선 0/1 바이트의 SHA-256과 base64 전체 마스크. 모든 스냅샷이 같은 지형을 참조하며 변경은 지원하지 않습니다.
- 제어/세션: control_step·iteration·Pause·남은 초·clock_frame·glitch 잔여·RR·생성 countdown·입구 개방·released/alive/removed.
- ID 순 개체: 생성 iteration·발 x/y·방향 -1/0/1·Walking/Jumping/Falling/Splatting·frame/end_of_animation/fallen·제거 이유. 제거 기록을 보존합니다.
- 순서 있는 출력: 입력 수락/거절, 시작 표식, 입구 개방, 생성 요청. 효과음 재생의 검증으로 해석하지 않습니다.

직업·영구 능력·객체/출구·구출·승패·Nuke·음악/오디오·DOS 실행은 헤더의 `unverified`에 명시합니다. 스텁을 0인 실제 관측값으로 기록하지 않습니다.

## 입력·시점·비교

JSON 설정의 `inputs`는 `{control_step, sequence, command}` 목록이며 SetReleaseRate에는 `rate`를 추가합니다. Pause/Resume/SetReleaseRate만 지원합니다. 경계·sequence는 엄격한 증가 순서이고 실행 밖 입력·중복·미지원 명령·잘못된 타입을 거부합니다. 초기 RR 미만이나 100~255인 명령은 실행에서 거절 이벤트를 남깁니다.

초기 스냅샷은 0/0입니다. 경계 s의 입력 → 시작/생성 → ID 순 이동 뒤 **control_step=s+1**을 저장합니다. Pause 중에도 제어 호출/스냅샷이 증가하고 iteration은 고정됩니다. 같은 생성 틱에 새 개체를 갱신합니다. 렌더 주기는 단일 약 1/17초·1초 묶음·불규칙 간격이며 FixedTickClock의 모든 예정 호출을 소비합니다. 지정된 실행 종료 뒤의 시간은 비교 범위 밖입니다.

정규 JSON은 키 정렬·ensure_ascii·공백/끝 개행 없음이며 부동소수점·타임스탬프를 금지합니다. 파일 자체에 자기 해시를 삽입하지 않고 별도 증거에 기록합니다.

- `MATCH_SUPPORTED` / 종료 0: 같은 초기 자료·입력으로 지원 필드의 매 스냅샷이 같습니다. 전체 호환성 PASS가 아닙니다.
- `DIFFERENT` / 종료 1: 최초 다른 control_step·JSON 경로·양쪽 값을 반환합니다. 진단 파일에 앞뒤 상태와 전체 정적 마스크를 보존합니다. 마스크가 다르면 최초 픽셀 좌표와 두 전체 마스크를 남깁니다.
- `UNVERIFIED` / 종료 2: 필드/스냅샷 누락·해시 불일치·잘못된 구조/타입·다른 초기 설정/입력/원본 식별입니다. 일치로 계산하지 않습니다.

## 실제 기준 코드 실행

`tools/lemmix/build_reference.py`는 고정 Lemmix `40e9bc34451f0e9127fd53b290a3877e700d143d`의 Game.pas 전체 SHA-256 `c1ba9ffda9e8e277f5c9b171df2bb611210d3326b9623b28fcd35426f4d6a448`을 먼저 확인합니다. HasPixelAt/ClipY·Transition/TurnAround·HandleLemming·네 이동 함수·상단 경계·제거·IncrementIteration·CheckSpawnLemming·Pause/Resume **15개 함수 원문**을 추출해 Free Pascal 3.2.2로 컴파일했습니다.

함수 식/분기는 번역하지 않습니다. FPC 호환을 위해 IncrementIteration의 for 변수와 BtnPause의 inline 변수를 선언부로 옮기는 두 구문 변환만 허용합니다. 변환 전 함수별·변환 후 전체·컴파일러/실행 파일·어댑터/추출기 해시를 기록합니다. 경계 상수는 원문과 확인하고 애니메이션은 #16의 고정 Styles.Base.pas 해시 및 Walking 8·Jumping 1·Falling 4·Splatting 16/foot(8,10)/Loop·Once 표를 검증합니다.

전체 Windows Lemmix 실행이 아닙니다. `harness.pas`가 최소 자료형·바이너리 지형·입구/개체 목록을 제공하고 입력 → IncrementIteration → CheckSpawnLemming → HandleLemming을 호출합니다. 그래픽·툴바·기록기·음향은 스텁이고 전체 게임 종료·객체·직업은 실행하지 않습니다. 미지원 동작 진입은 가능한 곳에서 예외로 실패합니다. RR 입력은 정규화된 직접 값이며 원본 마우스 누름/UI 증가 처리를 계측한 결과가 아닙니다.

42개 합성 사례는 해당 Pascal 함수를 실제 실행한 결과와 Rust 세 렌더 주기를 비교했습니다. 기록된 SHA-256은 상태 전체의 정규 JSON을 덮습니다. CI에서는 원문/FPC 없이 Rust 결과와 기록된 참조 해시를 대조하고, 어댑터/추출기 변경 시 참조 재실행을 요구합니다. 기존 Python 이동 모델과 별도의 근거입니다.

## 재현 명령

기본 회귀에는 원본·FPC가 필요하지 않습니다.

```sh
make check
```

기준 재실행에는 `reference_assets.py`로 확보한 소스, #16에서 고정한 `assets/local/issue-16-sources/Styles.Base.pas`, FPC가 필요합니다. 원본 사본·추출물·실행 파일·전체 추적은 ignored `assets/local/`에 둡니다. 출력 디렉터리/기록 파일은 기존 결과와 섞이지 않게 새 경로를 사용합니다.

```sh
python3 tools/reference_assets.py verify
python3 tools/reference_assets.py verify --manifest assets/manifests/reference_sources.json
python3 tools/lemmix/build_reference.py assets/local/issue-26-reproduce-reference
python3 tools/check_state_replay.py --reference assets/local/issue-26-reproduce-reference/reference
python3 tools/verify_original_replay.py assets/local/issue-26-reproduce-original --reference assets/local/issue-26-reproduce-reference/reference
```

FPC가 PATH에 없으면 추출기에 `--compiler /절대/경로/ppcx64 --units /절대/경로/units/x86_64-linux/rtl`을 지정합니다. 이 환경에서는 Ubuntu 패키지 `fp-compiler-3.2.2`/`fp-units-rtl-3.2.2`를 apt-get download로 받아 dpkg-deb -x로 로컬에 풀어 실행했습니다. 관리자 권한·시스템 설치는 사용하지 않았습니다.

생성된 설정/마스크로 개별 재생과 진단을 실행할 수 있습니다.

```sh
python3 tools/state_replay.py run assets/local/issue-26-original/F00.config.json assets/local/issue-26-original/f00.mask assets/local/issue-26-original/F00.extra.json --pattern jitter --assets assets/local/dos-original
python3 tools/state_replay.py compare assets/local/issue-26-original/F00.reference.json assets/local/issue-26-original/F00.extra.json --output assets/local/issue-26-original/F00.extra-comparison.json
```

## F00·F01 관측

원본 50개 해시를 검증한 후 기존 리소스 어댑터로 지형을 새로 합성했습니다. Fun01의 마스크는 1584×160, SHA-256 `a5311ef0436ca29a5fe4f6feee89baa0b82afce87baaf799f59c813deb5686d7`입니다. 다섯 원본 시나리오의 초기 포함 341개 상태를 각각 Rust 세 패턴과 대조했습니다. [증거 manifest](../../assets/manifests/state_replay.json)에 입력/참조/상태 해시를 보존합니다.

F00에서 시작 표식 15·입구 소리 표식 34·개방 35·첫 생성 54를 실제 하네스로 관측했습니다. actor 0이 Walking에 처음 진입한 스냅샷의 control_step은 60입니다. 따라서 다음 입력 경계 **60**, sequence 0을 F01 Digger 할당 후보로 고정합니다. Digger 할당 실행/완주는 G07/G08에 남습니다. 실제 DOS 계측 틱으로 표현하지 않습니다.

원본 재현 도구는 게시된 다섯 시나리오의 상태/입력/설정 서명과 F01 보정 경계도 대조합니다. 컴파일러/실행 파일 등 출처가 다르면 전체 추적 파일 해시는 달라질 수 있으며, 논리 스냅샷 서명은 같아야 합니다.
