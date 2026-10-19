# G07-04 Builder 계약과 검증

이슈: [#34](https://github.com/edwardkim/ohmylemmings/issues/34), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 고정 기준

Lemmix `40e9bc34451f0e9127fd53b290a3877e700d143d`의 [Game.pas](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas)에서 AssignBuilder·LayBrick·HandleBuilding·HandleShrugging을 기존24개에 더한28개 함수를 실제 FPC3.2.2로 실행합니다. 기존 두 inline 변수 문법 변환 외에 원문 함수 본문을 바꾸지 않습니다. [Styles.Base.pas](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Styles.Base.pas)의 Building16프레임/FootY13·Shrugging8프레임/FootY10과 벽돌색 palette7을 확인합니다.

## 코어 동작

- 살아 있는 Walking/Digging/Bashing/Mining/Shrugging 개체에 안정된 ID로 할당합니다. 현재 애니메이션 발 기준점의 `y-FootY >= -5`와 수량을 검사하고 성공 때만 Builder1개를 소비합니다. 좌표를 움직이지 않고 Building/frame0·벽돌12개로 시작합니다. 이미 Building인 개체에는 중복 할당하지 않습니다.
- Building은 먼저 프레임을16으로 순환시킵니다. 프레임9, 그리고 벽돌9개일 때의 프레임10에 오른쪽은`x..x+5`, 왼쪽은`x-4..x+1`의`y-1`에 지형을 추가합니다. 기존 고체는 보존하고 맵 밖은 자릅니다. 이 두 종류의 벽돌 배치 프레임에서는 객체 확인을 허용하지 않습니다.
- 프레임0에서 첫 수평1픽셀·위로1픽셀을 먼저 옮기고`x<=0 || x>1647` 또는 새 발의`y-1` 충돌에서 Walking으로 뒤돕니다. 다음 수평1픽셀 뒤 같은 발 충돌을 확인합니다. 이어 벽돌을1개 소비하며0이면 Shrugging으로 전환합니다. 이 소진 판단은 전방 머리와 두 번째 이동 뒤의 경계 판단보다 먼저입니다.
- 벽돌이 남으면`(x+방향*2, y-9)`의 머리 또는 월드 경계를 검사해 Walking으로 뒤돕니다. 너무 높으면 먼저 Walking으로 전환한 뒤 새 발 기준점으로 상부 보정을 하므로 일부 높이에서는 방향이 유지되는 원작 순서를 보존합니다. 지지 지형이 사라져도 Building/Shrugging 안에서 별도의 낙하 검사를 추가하지 않습니다.
- Shrugging은8프레임의 끝에서 Walking으로 바뀝니다. 양쪽 상태에서 기존 세 직업으로 교체할 수 있고 벽돌 수는0으로 초기화합니다. 복원 상태는 Building에서1..12개, 다른 상태에서0개만 허용합니다.
- `advance_with_jobs`/`step_actors_with_jobs`에서 처리합니다. 한 개체의 산술 오류는 그 개체와 픽셀을 보존하고 ID 순서 오류는 모든 변경 전에 거부합니다. 앞 개체가 만든 지형을 뒤 개체가 같은 틱에 봅니다. `MotionOutcome.added_pixels`에는 실제 빈 곳에서 고체로 바뀐 최대6개 좌표만 기록합니다.

## 네이티브 연결과 재생

U/M/B/D로 Builder/Miner/Basher/Digger를 선택하고 왼쪽 클릭으로 할당합니다. U는 기존 B/Basher 키와 충돌하지 않게 선택했습니다. 원본 Fun07 “Builders will help you here”는 Builder20개입니다.

```sh
source assets/local/dev-tools/env.sh
cargo run --release --locked --offline -p lemmings-desktop -- --assets assets/local/dos-original --level 7
```

재생 입력은`80 0 AssignBuilder 0`입니다. Pause 중 할당은 수량/상태만 바꾸며 진행을 재개한 뒤 지형을 추가합니다. 새 벽돌 재질을 별도로 보존해 원래 지형을 지운 뒤 같은 픽셀에 쌓아도 결합 palette7로 보입니다. 이 색은 첫 custom 색을 복사한 것이므로, 지형 내부 인덱스는0이며 합성 뒤8번/공유7번이 됩니다. 초기 내부 인덱스7 구현의47픽셀 차이를 실제 DOS 대조로 찾아 오프라인 합성 GROUND 회귀 테스트를 Red→Green으로 보정했습니다. 객체를 합성하기 전에 편집된 지형을 반영합니다.

`builder.jsonl`에는 매 경계의 수량·개체별 남은 벽돌·`filled`를, 기존`digger.jsonl`에는`cleared`와 전체 지형 서명을 기록합니다. `filled`/`cleared`는 경계 전후의 순변화이며 한 경계 안의 모든 중간 변경 이력은 아닙니다. 두 목록으로 초기 마스크를 재생해 전체 최종 마스크를 검사합니다. 최종 벽돌 재질은`brick-final.bin`에 기록합니다. 기존 직업만 실행한 경우의 상태/지형 서명을 유지합니다.

## 재현

준비 환경의 FPC 경로 예입니다. 추출물·실행 파일·원본 파생 결과는 ignored `assets/local/`의 새 디렉터리를 사용합니다.

```sh
python3 tools/lemmix/build_builder_reference.py assets/local/my-builder-reference --compiler /usr/lib/fpc/3.2.2/ppcx64 --units /usr/lib/fpc/3.2.2/units/x86_64-linux/rtl
python3 tools/check_builder.py --reference assets/local/my-builder-reference/reference
make check
```

기준 호스트의 Safe PixelS는 범위를 자르고 빈 픽셀에만 쓰는 원문 LayBrick을 실행합니다. 색상은0/ALPHA_TERRAIN1로 두어 충돌 마스크로 정규화합니다. RGB·GUI·사운드·툴바·미니맵/기록기는 별도 어댑터이며 전체 Lemmix 실행 인증은 아닙니다. CI는 원본/참조 소스 없이 실제 함수 실행으로 기록한 합성 golden과 어댑터 해시를 검사합니다. golden 갱신에는 실제`--reference`가 필수입니다.

```sh
SDL_VIDEODRIVER=x11 xvfb-run -a python3 tools/verify_builder.py assets/local/my-builder-native --reference assets/local/my-builder-reference/reference --xdotool /usr/bin/xdotool
SDL_VIDEODRIVER=x11 xvfb-run -a python3 tools/observe_builder_dos.py --dosbox /usr/bin/dosbox --xdotool /usr/bin/xdotool --output assets/local/my-builder-dos
```

DOS 관측은 새 복사본에서 #29의256바이트 RUSSELL.DAT 생성 단계를 재현하고 offset0만9→0으로 seed합니다. 이번 Arch DOSBox가 생성한 나머지 바이트는 이전 Ubuntu 기록과 달랐습니다. 생성 해시는`04462851c5edc273303b6fd19b8ee3c47ef722b9c1803dfd5adbd0229041e313`이며 이전 파일과 같다고 간주하지 않습니다. 새 생성 단계·입력/설정·해시를 보존하고 반복 실행은`--seed`와`--seed-sha256`으로 같은 파일을 고정합니다. fresh 생성의 길이256/카운터9와 재사용 해시, 변경 offset0만을 테스트했습니다.

원본 내장 F2의`LCANLLDHCO` 후보로 진입한 뒤 실제 preview의Level7·제목·50명/50%/RR50과 수량20·scroll464를 확인했습니다. 원본 EXE offset33dd/33f6의DS:76 수량,33ff의Building0x80,3412의개체+21(DS:a9) 벽돌12개를 정적 분석해 읽기 전용 관측 필드를 고정했습니다. 원본50개/EXE·에뮬레이터/게임 메모리는 변경하지 않습니다. 전용 X11에서 DOS 실행을 순서대로 수행합니다.

## 현재 근거와 한계

코어 새12개·네이티브 새5개·Python 지형/필드 검사를 Red→Green으로 확인했습니다. 실제 Pascal28함수의 합성8사례/16재생/6736상태/718개 SDL readback과 모든 경계 추가/삭제가 일치합니다. 원본 Fun07 네이티브4실행/1244상태/139캡처·실제 U키/클릭이 기준 함수와 일치합니다. 입력80의 세 재생 서명은`985c867c7211450e7fb36b116c0d686e29b11c35c439306cce56e3ce4afd3cc0`입니다.

최종 DOS 두 실행2/3의1303개 숫자 샘플·4760게임 영상 프레임에서 수량20→19·Building0x80·벽돌12개 진입을 관측했습니다. counter81의Walking(567,144)/frame7→할당은 native 입력80의(551,128)과16픽셀 테두리를 뺀 값입니다. 220·260·340 전체320×160 세 장면×2실행이0픽셀 차이입니다. 정확한2배 픽셀/스캔라인·RGB565 코드 대조를 유지하며 커서 마스크·위치 이동·색 허용 오차를 추가하지 않았습니다. 실제 원본/실행 복사본·바이너리/seed·정규화한 설정·mapper/입력 절차의 동일성을 재검증합니다. 반복 판정은 같은 프로토콜로 실행한 서로 다른 녹화 디렉터리 두 개 이상을 요구합니다. self-review에서 같은 폴더를 두 번 세는 경우를 실패 테스트로 확인하고 정규화 경로의 중복·상대 경로/심볼릭 링크 별칭·빈 경로를 거부하도록 보완했습니다. 기존 두 녹화의 재대조 결과는 동일합니다.

초기 native-proof-1과dos-compare-1의 색 차이 및runtime guard에서 종료된dos-proof-1은 최종 실행 횟수에서 제외합니다. bootstrap의 실제 생성 RUSSELL.DAT은 seed 출처로만 보존합니다. 전체 회귀·게시/실제 CI는 [결과 보고서](../reports/issue-34_builder.md)와 #34에서 추적합니다. 제한된 비동기 숫자 폴링은 원자적 전체 DOS 상태가 아니며 전체 직업·객체/구출/승패·음향·전체 DOS 인증은 후속 범위입니다.

비교는 위 절차로 만든 두 실행과 네이티브의 새 출력 경로를 지정합니다. PyAV14.2.0이 설치된 준비 환경을 사용합니다.

```sh
python3 tools/compare_builder_dos.py --recording assets/local/my-builder-dos/run assets/local/my-builder-dos-repeat/run --native assets/local/my-builder-native/F07-single --output assets/local/my-builder-comparison
```

두 번째 관측 실행은 첫 실행의`bootstrap/drive/RUSSELL.DAT`을`--seed`로 지정하고 해당 파일의 SHA-256을`--seed-sha256`으로 명시합니다. 각 실행은 별도 새 출력 경로를 사용합니다.
