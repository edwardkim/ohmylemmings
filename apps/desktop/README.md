# Desktop preview

Linux SDL2 어댑터로 원본 **Fun 01 — Just dig!**의 지형·입구·출구 이미지와 생성·걷기·낙하를 표시합니다. 현재 직업 할당·출구 판정·승패·음향은 후속 구현입니다.

## 준비와 실행

Ubuntu/Debian에서 SDL2 개발 라이브러리와 고정 Rust 도구 체인을 준비하고 저장소 루트에서 실행합니다.

```sh
sudo apt-get update
sudo apt-get install --no-install-recommends -y libsdl2-dev
cargo fetch --locked
python3 tools/reference_assets.py fetch
python3 tools/reference_assets.py verify
cargo run --release --locked --offline -p lemmings-desktop -- --assets assets/local/dos-original
```

최초 fetch 이후 Cargo 명령은 오프라인으로 실행합니다. Release 빌드를 권장합니다. 원본 없이 SDL2를 확인하려면 마지막 명령의 앱 인자를 `--synthetic`으로 바꿉니다.

## 조작

| 키 | 동작 |
|---|---|
| Space | 일시정지/재개 |
| Up / Down | 생성 속도 +1 / -1, 레벨 초기값~99 |
| Left / Right | 카메라 16픽셀 이동 |
| Home / End | 월드 양끝 |
| Esc / 창 닫기 | 종료 |

`--help`에서 실행 옵션을 확인합니다. `--steps 340`은 제어 호출 340회 후 종료하며, Pause 중에도 제어 호출은 증가합니다. `--fast`는 `--steps`를 지정한 검사에서만 벽시계 대기를 생략합니다.

## 실행 간격

원본 기본값 `--pacing dos`는 고정 DOSBox/VGA 메뉴 PC2 기준의 `dos-vga-5` 간격(약 14.017호출/초)입니다. 기존 17Hz 실행은 `--pacing reference`로 선택하며 합성 실행의 기본값입니다. 게임 시계는 두 경우 모두 17호출마다 1초입니다. 같은 호출 번호/입력에서는 같은 상태와 그림을 만듭니다. `run.json`에 실제 pacing·분자/분모를 기록합니다. 다른 DOS 판본/하드웨어·-x 경로까지 같은 속도라고 가정하지 않습니다.

[#30 보정·관측 범위](../../mydocs/tech_insights/issue-30_dos_differences.md)를 참고합니다.

## 입력 기록과 화면 캡처

`--capture assets/local/my-native-run`은 새 디렉터리에 초기/매 호출 상태 `trace.jsonl`, 재생할 `inputs.txt`, 실행 정보 `run.json`과 주요 경계/입력 직후의 RGB·SDL readback PPM을 기록합니다. 원본 파생 캡처는 `assets/local/` 아래에 두며 기존 결과를 덮어쓰지 않습니다.

`--replay INPUTS_FILE`은 `CONTROL_STEP SEQUENCE Pause|Resume|SetReleaseRate [RATE]` 형식의 정렬된 입력을 재생합니다. 재생 중에는 카메라와 종료 키를 사용할 수 있습니다. Space·RR은 기록된 입력을 따릅니다.

## 검증

```sh
make check
```

`make check-desktop`은 앱을 오프라인 빌드한 뒤 `target/debug/lemmings-desktop`으로 실제 SDL dummy/software 창·입력 루프·화면 readback을 원본 없이 검사합니다. WASM 검사는 기존 두 라이브러리만 빌드합니다.

[구현 계약·참조·X11 재현](../../mydocs/tech_insights/issue-27_native_view.md), [검증 보고서](../../mydocs/reports/issue-27_native_view.md).
