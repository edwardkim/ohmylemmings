# G08-03 시간·구출률·승패 계약

이슈: [#52](https://github.com/edwardkim/ohmylemmings/issues/52), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1). 선행 [#48](issue-48_exit_rescue.md)·[#50](issue-50_hazards.md).

## 기준과 순서

고정 [Game.pas](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas)의 `IncrementIteration`·`CheckForGameFinished`·`SetGameResult`와 확보한 DOS `VGALEMMI.EXE`를 기준으로 합니다. 시간은 시작 대기부터17논리틱마다1초 감소하고0에서 포화합니다. Pause는 시간·개체·함정·파티클을 진행하지 않으며 시작 PauseGlitch는 #24의 기존 동작을 유지합니다.

DOS 주소는 MZ 헤더288바이트를 제외한 **CS 오프셋**입니다. 기존 문서의 파일 오프셋과는288 차이입니다. CS:01ad~01bf의 생성·개체/파티클 갱신 뒤01e7~0209가 생성 수/표시 중 개체 수와 기존 시각의0 여부를 검사하고,0213~0246이 시각을 감소시킵니다. 따라서 `CompletionProgress.seconds_before_tick`에는 해당 호출 전 시각을 넣습니다. 남은 시간이0이 되는 틱 뒤에도 다음 경계의 마지막 개체 갱신/구출이 판정에 포함됩니다. 전체 제거가 시간0과 겹치면 `PopulationExhausted`를 먼저 선택합니다.

원본은 폭발 표시가 끝날 때까지 DS:45의 표시 중 개체로 셉니다. 정규화 코어는 폭발 즉시 제거하므로 남아 있는 개체별 `particle_timer`의 최댓값으로 정상 종료를 늦춥니다. 시간 만료는 이 대기를 통과합니다. 필요한 구출률을 충족했다는 사실만으로는 종료하지 않습니다.

## 정수 백분율과 결과

- `target = floor(required * 100 / original_total)`
- `done = floor(rescued * 100 / current_max)`
- `success = done >= target`

DOS CS:3ace~3af4는 DS:46의 구출 수와 DS:47의 현재 최대 인원으로 백분율을 구합니다. CS:4591~459e는 원래 레벨 인원/필요 구출 수로 목표를 계산하고,4720~4724가 결과를 비교합니다. Nuke는 #44처럼 현재 최대 인원을 생성된 수로 줄입니다. 예를 들어 원래80/필요40인 레벨에서2마리 생성 후 Nuke·1마리 구출이면50%/50%로 성공합니다. 절대 필요 수만 비교하면 이 동작을 놓칩니다.

양쪽 백분율을 각각 버린 뒤 비교합니다. 코어는 `u64` 곱셈으로 overflow를 피하며0 분모는0%로 정의합니다. `required > original_total`과 잘못된 구출⊆제거⊆생성⊆총원, 잘못된 Nuke 분모·파티클 범위는 오류입니다. 이는 손상/합성 입력에 대한 명시적 계약이며 DOS의 모든 잘못된 입력 동작을 재현한다는 뜻은 아닙니다.

## 코어·세션·추적

`CompletionRules`는 I/O·할당 없이 순수 판정을 수행합니다. `StartupScheduler`가 시간을 맡고 데스크톱 `Session`은 개체 갱신 뒤 최초 `LevelResult`를 보존합니다. 종료 후 틱·게임 명령·수량·지형이 바뀌지 않습니다. 화면에는 `WON`/`LOST`와 현재/목표 백분율을 표시합니다. 카메라와 Esc는 사용할 수 있습니다.

원본 레벨은 자동으로 판정을 사용합니다. 기존 합성 개체 전용 fixture는 고정된 관측 구간을 유지하며, 완주 회귀는 `--synthetic-outcome win|loss|clock|particles|zero|mixed`로 실행합니다. 유한 `--steps` 재생은 종료 시 마지막 상태/화면을 기록하고 끝납니다. 종료 뒤 예약된 입력은 적용하지 않습니다. 일반 실행은 종료 상태를 표시한 채 Esc를 기다립니다. `completion.jsonl`에는 `Playing/Won/Lost`, 이유, 원래/현재 총원·필요/구출 수와 두 백분율을 기록합니다.

## 실제 기준 실행과 재현

`build_completion_reference.py`는 기존54함수/표에 변경하지 않은 `CheckForGameFinished`·`SetGameResult`를 더해56함수/표를 FPC로 실행합니다. 자체 호스트는 메타데이터·입력·추적과 `Finish` 연결을 맡습니다. **원문 `InternalUpdate` 전체 실행은 아닙니다.** 원문은 호출 시작에서 종료를 검사하고 파티클 대기를 시간보다 앞에 둡니다. DOS에 맞춘 자체 경계 어댑터가 개체 갱신 후의 수량과 갱신 전 시각을 넘기고, 시간0일 때만 종료 대기를0으로 전달합니다. 원문 함수 본문은 그대로 유지하고 이 차이를 숨기지 않습니다.

```sh
python3 tools/lemmix/build_completion_reference.py assets/local/my-completion-reference --compiler /path/to/ppcx64 --units /path/to/rtl
python3 tools/check_completion.py --reference assets/local/my-completion-reference/reference --record --evidence assets/local/my-completion-synthetic
python3 tools/verify_completion.py assets/local/my-completion-original --reference assets/local/my-completion-reference/reference
python3 tools/check_completion.py  # 원본/FPC 없는 CI 회귀
```

DOS 검증은 별도 X11 display에서 `observe_completion_dos.py --profile win|loss|timeout`으로 정상 키·read-only 자식 메모리만 사용합니다. win/loss는 Fun01, timeout은 [고정 legacy 생성기](https://github.com/tomsoftware/lemmings.js/blob/82fb648e70595c5ee8fafce10a2133397eeacddb/codeGenerator.js)의 후보 코드로 진입하는 Mayhem16이며 실제 레벨 인덱스105·1분 시각을 확인해야 인정합니다. 원본50개·EXE는 수정하지 않고 문서화한 별도 RUSSELL seed를 사용합니다. 결과 화면 PNG/짧은 영상·비동기 수량/시각/결과 표지를 보존합니다. 결과 아트워크를 네이티브와 동일하게 렌더링하거나 원자적 전체 DOS 추적을 검증하는 범위는 아닙니다.

결과 화면/재시작·원본 패널/미니맵은 후속입니다. [진행 기록](../working/issue-52_level_outcome.md)에서 실행 결과와 게시 상태를 추적합니다.
