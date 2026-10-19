# G07-05 Blocker 계약과 재현

이슈: [#36](https://github.com/edwardkim/ohmylemmings/issues/36), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 고정 기준과 객체 맵

Lemmix `40e9bc34451f0e9127fd53b290a3877e700d143d`의 [Game.pas](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas)에서 기존28함수에 ReadObjectMap·WriteObjectMap·SaveMap·RestoreMap·SetBlockerField·CheckForOverlappingField·AssignBlocker·HandleBlocking·CheckForInteractiveObjects를 더한37함수를 FPC3.2.2로 실행합니다. 기존 두 inline 변수 문법 변환 외에 추출 함수 본문을 바꾸지 않습니다. Blocking은 [Styles.Base.pas](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Styles.Base.pas)의16프레임/FootY10/반복 메타데이터를 따릅니다.

객체 맵은 충돌 지형과 별개인 호출자 소유 바이트 배열입니다. 네이티브 프로필은 원문 초기화 식 `(1647+16) div4`×`(175+16) div4`를 따라415×47, 원점(-16,-16)입니다. 원문 주석의 끝 좌표와 실제 배열 크기를 구분합니다. 음수 좌표는4픽셀 단위로 내림하고 밖은128/쓰기 무시입니다. 알 수 없는 코드도 보존하며 그 효과를 자동 실행하지 않습니다. 코어는 일반 크기/정렬 원점을 지원하며 곱셈 초과·길이·빈 크기·정렬 오류를 거부합니다.

네이티브 초기화는 강철137을 먼저 쓰고 입구를 제외한 활성 객체 목록의 처음16개를 순서대로 덮어씁니다. 객체 위치를4픽셀 단위로 내린 뒤 트리거 사각형을 적용합니다. 효과4는 이 목록의0부터 시작하는 번호, 나머지는128+효과이며 범위를 넘는 효과 바이트는 오류입니다. 음수 패딩·강철 덮어쓰기·입구 제외·압축 목록 번호·17번째 객체 제외를 자체 레벨/GROUND 바이트로 검사합니다.

## 전환과 틱 순서

- 살아 있는 Walking/Shrugging/Building/Bashing/Mining/Digging에 수량이 있으면 할당할 수 있습니다. `(x-4,x,x+4)`×`(y-6,y-2,y+2)`의9칸 중130/138/131이 하나라도 있으면 모든 변경 전에 거부합니다. 성공 때9개의 원래 코드를 저장하고 좌/중/우 열을130/138/131로 쓰며 수량1개를 소비합니다. 할당 시 발판 존재를 추가 검사하지 않습니다.
- Blocking은 이동하지 않고16프레임을 반복합니다. 발의 지형을 상부0으로 자른 좌표에서 검사하며 지지가 없으면9칸을 복원하고 같은 좌표의Walking으로 전환합니다. 이 틱에는 객체를 확인하지 않으며 다음 틱에 Walking의 낙하 판단을 합니다. 저장 영역은Blocking에만 존재하는 복원 상태 불변식입니다.
- 다른 직업은 자신의 기존 `check_objects` 프레임에서만 객체를 확인합니다. 발 코드와 현재 방향의 전방8픽셀/상부8픽셀 코드를 **방향을 바꾸기 전에** 캐시합니다. 발이130/131이고 반대로 향할 때 방향만 뒤집습니다. 위치/애니메이션은 유지합니다. Digger/Basher/Miner의 다음 할당은 최신 맵을 새로 읽지 않고 이 캐시를 사용합니다. Blocker의 맵 덮어쓰기가 강철/일방통행 제한을 바꾸고 복원이 되돌립니다.
- `advance_with_objects`/`step_actors_with_objects`는 기존 지형 변경 경로를 사용합니다. 엄격한 ID 순서를 변경 전에 검사하며 앞 개체의 굴착·영역 복원을 뒤 개체가 같은 틱에 봅니다. 기존 불변/정적 마스크 API는 유지합니다. Blocking을 맵 없는 API로 진행하면 상태를 보존하고 명시적 오류를 반환합니다.

## 네이티브와 재생

K/U/M/B/D는 Blocker/Builder/Miner/Basher/Digger 선택입니다. 왼쪽 클릭은 안정된 개체 ID를 기록합니다. 재생 입력 예는 `80 0 AssignBlocker 0`이며 Pause 중 할당은 상태/수량/영역을 바꾸고 애니메이션은 재개 후 진행합니다.

```sh
source assets/local/dev-tools/env.sh
cargo run --release --locked --offline -p lemmings-desktop -- --assets assets/local/dos-original --level 3
```

`blocker.jsonl`은 수량·개체별 저장9칸·발/전방 캐시·전체 객체 맵 FNV·경계별 순변화 `changed`를 기록합니다. `object-map-initial.bin`에서 순서대로 재생해 매 경계 서명과 `object-map-final.bin`을 검증합니다. 충돌 지형의 추가/삭제 기록과 분리합니다. 순변화는 한 제어 경계 내부의 모든 중간 쓰기 이력을 의미하지 않습니다.

SDL 캡처는 논리 viewport를 잠시 해제하고 전체 출력 버퍼를 읽은 뒤 복원합니다. 검증은 실제 출력 크기의 정수 최근접 배율·중앙 배치·모든 검은 여백 바이트를 검사합니다. 보이는 Xwayland 창은 자신의 PID로 찾고 HUD 클릭으로 초점을 얻은 다음 K키·게임 클릭을 보냅니다. 데스크톱 설정은 변경하지 않습니다.

## 기준 실행과 DOS 재현

```sh
python3 tools/lemmix/build_blocker_reference.py assets/local/my-blocker-reference --compiler /usr/lib/fpc/3.2.2/ppcx64 --units /usr/lib/fpc/3.2.2/units/x86_64-linux/rtl
python3 tools/check_blocker.py --binary target/release/lemmings-desktop --reference assets/local/my-blocker-reference/reference
make check
SDL_VIDEODRIVER=x11 python3 tools/verify_blocker.py assets/local/my-blocker-native --reference assets/local/my-blocker-reference/reference --xdotool /usr/bin/xdotool
```

`make check-desktop`은 release 빌드를 한 번 만든 뒤 기존/새 SDL 사례를 모두 같은 실행 파일로 검사합니다. Rust 단위 테스트는 debug 빌드를 유지합니다. CI는 원본 없이 실제 함수 실행에서 기록한 합성 golden·어댑터 해시를 검사합니다. golden 갱신은 실제 `--reference`가 필수입니다. 기준 호스트는 그림/사운드/툴바/기록기를 스텁 처리하고, 트랩/출구/물/불은 비교 사례 밖으로 제한합니다. 전체 Lemmix 실행을 인증하지 않습니다.

DOS는 원본 Fun03 “Tailor-made for blockers”의 preview(Level3/50명/10%/RR50/5분), scroll512·Blocker10개를 실제 확인합니다. F2 후보 `NJLDLCADCY`는 [레벨 코드 역분석 게시물](https://www.lemmingsforums.net/index.php?topic=7364.0)을 참고해 구했으며 원본 진입 상태로 검증했습니다. EXE offset3456/3542는DS:73 수량,354b는Blocking0x100,3589는개체+0x16부터9칸 저장을 보여 줍니다. 실제 읽기 전용 관측은DS:9e..a6를 보존합니다. DOS 원시 영역 코드는2/10/3, 빈 칸은0이며 Lemmix의130/138/131·128과 구분합니다.

```sh
python3 tools/observe_blocker_dos.py --dosbox /usr/bin/dosbox --xdotool /usr/bin/xdotool --output assets/local/my-blocker-dos --seed assets/local/issue-34-dos-proof-1/bootstrap/drive/RUSSELL.DAT --seed-sha256 04462851c5edc273303b6fd19b8ee3c47ef722b9c1803dfd5adbd0229041e313
python3 tools/observe_blocker_dos.py --dosbox /usr/bin/dosbox --xdotool /usr/bin/xdotool --output assets/local/my-blocker-dos-repeat --seed assets/local/issue-34-dos-proof-1/bootstrap/drive/RUSSELL.DAT --seed-sha256 04462851c5edc273303b6fd19b8ee3c47ef722b9c1803dfd5adbd0229041e313
python3 tools/compare_blocker_dos.py --recording assets/local/my-blocker-dos/run assets/local/my-blocker-dos-repeat/run --native assets/local/my-blocker-native/F03-single --output assets/local/my-blocker-comparison
```

새 환경에서는 이전 생성 파일이 있다고 가정하지 않습니다. `--seed`를 생략해 기존 bootstrap 절차로 생성하고, 반복 실행에서는 실제 생성 파일의 SHA-256을 고정합니다.

이번 실행은 #34의 Arch DOSBox가 생성한256바이트 seed를 재사용하며 offset0만9→0으로 바꿉니다. DOSBox0.74-3·VGA·386 normal/fixed10000cycles·PC2·키보드 F6/space 절차를 고정합니다. 원본50개/EXE·에뮬레이터·게임 메모리는 수정하지 않습니다. `/proc` 관측은 자식의 숫자 필드를 비동기로 읽기만 합니다. 두 실행은 서로 다른 실제 녹화 디렉터리와 같은 프로토콜 해시를 요구합니다. 220/260/340의 전체320×160을 정확한2배 픽셀/스캔라인·RGB565 코드 정규화로 대조하며 커서 마스크·위치 이동·색 허용 오차를 쓰지 않습니다.

전체 결과와 제외한 준비 실행은 [보고서](../reports/issue-36_blocker.md), 원본 파생물 없는 출처/해시는 [manifest](../../assets/manifests/blocker.json)에 기록합니다. Bomber에 의한 영역 해제·트랩/구출/승패·음향·전체 G07/DOS 인증은 후속 범위입니다.
