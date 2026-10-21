# G11-05a 저장 해법·결과·재시작 비교 계약

이슈: [#94](https://github.com/edwardkim/ohmylemmings/issues/94) · 상위 [#93](https://github.com/edwardkim/ohmylemmings/issues/93) · [G11 계획](../plans/issue-85_compatibility_validation.md).

## 입력과 판정

[공개 목록](../../assets/manifests/solution_replay.json)은 이번 작업에서 작성한 안정된 actor ID 입력14개를 담는다. 원본 바이너리/이미지/맵/음원과 실행 원시 자료는 포함하지 않는다. Fun01~07 원본 수량·재고·구출 목표를 유지하고 Pause/Resume/RR/Nuke와8직업의 실제 게임 명령만 허용한다. 엄격한 `(control_step, sequence)` 순서·동일 경계0부터 연속 순서·정수/인수·LF 직렬화·관측 끝 이전 입력을 확인한다. 목표나 결과를 설정하는 명령은 없다.

성공은 로더가 제공한 원본 `total/required`가 모든 경계에서 보존되고 실제 결과가 Won이며 구출 수가 원본 목표 이상일 때만 인정한다. 실패는 실제 Lost와 명시한 사유, 목표 미달을 요구한다. Nuke 분모 감소는 원래 계약으로 허용하지만 원본 목표를 낮춰 성공시키지 않는다. 한 번도 종료하지 않은 재생, 결과 뒤에 남은 미소비 입력, 3회 중 상태/맵/화면 차이는 실패다.

## 실행

[기존 초기 재생 도구](../../tools/initial_replay.py)의 실제 SDL 스트림·전체 v1 진단/actor 필드·지형/brick/raw object map 복원·전체 physical readback 검증을 재사용한다. 선택적 저장 입력을 받아 dense 렌더1의 매 경계 전체 RGB와 sparse7/13을 대조하며 초기 CLI의 무입력 계약은 유지한다. 각 사례의 종료 뒤4개 guarded probe도 관측한다. 새 [도구](../../tools/solution_replay.py)는 각 레벨 성공/실패 짝을 요구하고 SHA/출처와 최종 수치를 기록한다.

동일 입력을 SDL에서 렌더1/7/13·시간30/60/144Hz로3회 실행한다. 실제 WASM은 같은 조합으로 FramePump 예산3·정수 마이크로초·숨김/1시간 경과/복귀를 사용한다. 매 경계 전체 상태/3개 맵의 SHA, 선택한 전체320×200 RGB와 Canvas RGBA readback이 SDL dense 기준과 일치해야 한다. 실제 WASM 모듈과 Canvas를 사용하지만 실시간 프레임 마감 계측은 아니다.

SDL은 성공/실패 결과에서 실제 Restart 경로를 각각3회(렌더37/101/257) 실행한다. 기존 제품 계약에 따라 이전 입력 로그를 폐기하고 입력 없는 새 세션을 진행한다. ID/전환1회·경계0 전체 상태·초기 재고/수치·자원/steel/trap/지형/raw object map·brick 초기화·이전 입력 비전파를 대조하며3회 새 세션의 최대341경계 prefix가 같아야 한다. 첫 결과까지의 모든 상태도 앞서 고정한 입력 재생과 대조한다. 전체 readback은 확인 후 파일을 제거하고 크기/SHA를 보존한다.

WASM은 각 결과에서 `transition('restart')`로 같은 레벨을 다시 시작한 뒤 **같은 입력을 다시 완주**한다. 경계0/전체 후속 상태·맵·화면을 다시 SDL 기준과 대조하며 음원 세대+1·queued0·sequence0·reset을 확인한다. 따라서 기본3회와 재시작 뒤3회가 있다. SDL의 재시작 뒤 입력 없는 경로와 구분해 집계한다.

## 재현

`source assets/local/dev-tools/env.sh` 이후 `python3 tools/solution_replay.py --smoke`는 독립 작성한 byte bank의 성공/실패와 실제 SDL/WASM 재시작을 검사한다. `make check-web`에도 포함되며 원본을 요구하지 않는다. Python 단위 검사는 `make check-python`에 포함한다.

원본 검증은 공개 목록을 ignored 입력 파일로 풀어 다음처럼 실행한다. 기존 증거 디렉터리를 덮어쓰지 않는다.

```sh
python3 - <<'PY'
import json
from pathlib import Path
p=Path('assets/local/solution-inputs');p.mkdir(exist_ok=False)
m=json.loads(Path('assets/manifests/solution_replay.json').read_text());cases=[]
for c in m['cases']:
    name=c['id']+'-'+c['phase'].lower()+'.txt'
    (p/name).write_text(c['input_text'])
    cases.append({k:c[k] for k in ['id','phase','steps']}|{'inputs':name})
(p/'cases.json').write_text(json.dumps(cases))
PY
python3 tools/solution_replay.py --assets assets/local/dos-original \
  --cases assets/local/solution-inputs/cases.json \
  --evidence assets/local/solution-run --workers 3
```

## 증거 범위

최종 원본 실행·각 사례 상태/입력/전체 프레임 체인 해시·수치와 정확한 push/PR/main CI ZIP·checkout SHA는 [GitHub #94](https://github.com/edwardkim/ohmylemmings/issues/94)에 게시한다. 저장소 목록/보고서는 코드 검토 단계의 고정 계약이고 `review_stage`를 최종 런타임 PASS로 읽지 않는다.

COMP-07의8직업 등장과 COMP-17/18의 **첫7개 호스트 입력 묶음**을 보강한다. 다른113레벨, 전체 COMP-01~18의 재감사, 모든 입력의 DOS 내부 상태 동일성, 새로운7개 DOS 영상 관측, 실시간 성능 인증은 남아 있다. 원작 관측은 기존 [#60](../reports/issue-60_native_playthrough.md)/[#70](../reports/issue-70_web_playthrough.md)의 Fun01 범위만 재사용한다. 이번 새 입력의 안정된 actor ID 재생은 실제 DOM 포인터·키 조작 인증과 구분한다.
