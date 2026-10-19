# G08-04 결과 화면·재시작 계약

이슈: [#54](https://github.com/edwardkim/ohmylemmings/issues/54), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 근거와 지원 경계

고정 [Lemmix Postview](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/GameScreen.Postview.pas)의 `GetScreenText`는 종료 이유·구출/목표 비율을 표시하고 성공할 때 다음 레벨을 선택합니다. 왼쪽 클릭/Enter는 실패 시 재시도, 성공 시 다음 레벨입니다. 원본 DOS 결과 수치는 [#52 관측](issue-52_level_outcome.md)에서 확인했습니다. 이번에는 해당 Pascal UI 전체를 실행하거나 DOS 결과 화면의 픽셀 일치를 주장하지 않습니다.

SDL 어댑터는 자체 320×200 결과 화면을 사용합니다. R은 항상 방금 한 레벨을 새로 시작하는 추가 단축키입니다. 메뉴가 없는 현재 앱에서 Esc/오른쪽 클릭은 종료합니다. 다음 레벨은 기존 데스크톱의 Fun 1..30 범위이며 Fun30 성공 뒤에는 다음 이동을 제공하지 않습니다. 다른 난이도·미리보기/메뉴·접근 코드·원본 640×350 배경/글꼴 조립은 후속입니다. 기존 로더가 지원하지 않는 특수 그래픽 또는 누락/손상 파일은 전환 실패로 표시하고 결과를 보존합니다.

## 세션 교체

`lifecycle::LevelRun`은 자산·Session·View와 현재/다음 번호를 묶습니다. 결과 확정 전에는 전환을 허용하지 않습니다. 로딩과 새 세션 구성이 성공한 뒤 교체하므로 로딩 실패가 확정 결과를 지우지 않습니다.

재구성은 원래 자산에서 시계/시작 지연·개체 ID·지형/벽돌·객체/함정·직업 수량·Nuke·폭발/파티클·결과·입력 순서·벽시계 잔여분을 초기화합니다. View의 카메라/직업 선택/장면 캐시도 새로 만듭니다. main은 남은 catch-up 틱·렌더/캡처 경계·이전 재생 iterator를 폐기합니다. 시뮬레이션은 기존 no_std 코어를 그대로 사용합니다.

## 재생과 캡처

`--steps`는 시도별 상한이며 결과가 나오면 종료합니다. `--result-actions restart,continue`는 최대32개 장면 명령을 실제 SDL 키 이벤트로 결과 화면이 처음 그려진 뒤 주입합니다. 유한 `--steps`가 필요합니다. 결과 전에 상한이 오거나 다음 이동이 불가능하면 실패를 반환합니다. 캡처 유무·렌더1/7에 관계없이 최초 결과 프레임을 그립니다.

기존 `--replay`는 첫 시도에만 적용됩니다. 다음 시도는 수동 입력을 받으며, 결과 뒤의 예약 명령을 다시 실행하지 않습니다. 시도0은 캡처 루트, 이후는 `attempt-0001/`부터 별도 상태/입력/config/화면을 기록합니다. 루트 `transitions.jsonl`은 시도·레벨·종료 control_step·동작·대상을 기록합니다. 각 시도의 `inputs.txt`를 독립 실행에 재생할 수 있습니다. 기존 캡처를 덮어쓰지 않습니다.

합성 `win`은 다음 이동 검사에 필요한 두 번째 `loss` 장면을 제공합니다. 다른 합성 완주 장면은 단일 레벨입니다. 원본 파생 증거는 ignored `assets/local/`에 보관합니다.

## 재현

```sh
make check-desktop
python3 tools/check_results.py --evidence assets/local/my-results --assets assets/local/dos-original
# 전용 X11 display에서:
python3 tools/verify_results.py --xdotool /path/to/xdotool --evidence assets/local/my-results-x11
```

[진행](../working/issue-54_results_restart.md) · [보고서](../reports/issue-54_results_restart.md) · [일지](../orders/20261020.md#issue-54).
