# G07-06 Bomber 계약과 재현

이슈: [#38](https://github.com/edwardkim/ohmylemmings/issues/38), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 고정 기준과 순서

Lemmix `40e9bc34451f0e9127fd53b290a3877e700d143d`의 [Game.pas](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Game.pas) 기존37함수에 AssignBomber·UpdateExplosionTimer·HandleOhNoing·HandleExploding·ApplyExplosionMask·CheckLemmings를 더한43함수를 실제 FPC3.2.2로 실행합니다. 기존 두 inline 변수 문법 변환 외에 원문 본문을 바꾸지 않습니다. OhNo16프레임/FootY10·Explosion1프레임/FootY25·16×22 마스크와52틱 입자 카운터를 고정합니다.

- 살아 있는 개체에 중복 카운트다운이 없고 Splatting/OhNo/Exploding이 아니면 수량1개를 소비하고 타이머79로 시작합니다. 직업/좌표/애니메이션은 바꾸지 않으므로 기존 다섯 직업과 병행합니다. 이후 다른 직업 할당도 타이머를 유지합니다.
- 매 진행 틱은 입자 카운터·제거 가드·폭발 타이머·행동·객체 확인 순서입니다. 타이머가0이 된 틱은 이동/객체 확인을 건너뛰고, Falling이면 곧바로Exploding, 다른 구현 행동이면OhNo로 전환합니다. Splatting 진입은 타이머를 취소합니다. 숫자는65..79=5,49..64=4,33..48=3,17..32=2,1..16=1입니다. 이 카운터를 벽시계5초로 바꾸지 않습니다.
- OhNo는16프레임의 끝에서Exploding으로 전환합니다. 그 전에는 최대3픽셀 내려오고 발 지형을 순서대로 검사합니다. 하부163을 넘으면 제거하며 지상 프레임만 객체 확인을 허용합니다. 원문의 음수 Y 클리핑·ID 순서를 유지합니다.
- Exploding 다음 틱에 저장된 Blocker9칸을 **현재 좌표에서** 먼저 복원하고 발 코드를 다시 읽습니다. 강철137/물133이면 지형을 지우지 않습니다. 다른 경우 `(x-8,y-14)`의 MSB-first16×22 마스크로 실제 고체만 지웁니다. 주변 강철 셀을 픽셀별로 추가 보호하지 않습니다. 제거·입자52/frame0으로 전환하고 이후 제거된 개체도 입자 카운터를 진행합니다. 프레임은50에서 멈추며 타이머가0이면 표시하지 않습니다.
- Blocker의 저장 영역은Blocking/OhNo/Exploding에서 보존합니다. OhNo 도중 지지 소실로 좌표가 바뀌면 원문처럼 현재 좌표에서 복원하며 새로운 원점 보정을 추가하지 않습니다. 원문의 이 순서와 전체 객체/완주 인증은 구분합니다.

`BomberContext`와 `advance_with_bomber`/`step_actors_with_bomber`는 호출자 소유 지형·객체 맵·폭발 마스크를 받습니다. ID 오류는 모든 변경 전에 거부하고 한 개체의 산술 오류는 그 개체/지형/맵을 보존합니다. 기존 API는 활성 Bomber 문맥이 필요하면 명시적 오류를 반환합니다. 코어에는 리소스/렌더링/I/O 의존성을 추가하지 않습니다.

## SDL과 원본 표시 보정

E/K/U/M/B/D는 Bomber/Blocker/Builder/Miner/Basher/Digger 선택입니다. 클릭은 안정된 ID를 기록합니다. 예시는 `80 0 AssignBlocker 0`과 `100 0 AssignBomber 0`이며 Pause 중 할당은 가능하지만 타이머/행동/입자는 재개 후 진행합니다.

```sh
source assets/local/dev-tools/env.sh
cargo run --release --locked --offline -p lemmings-desktop -- --assets assets/local/dos-original --level 6
```

실제 DOS 전체 장면 대조에서 Lemmix의 숫자 표시 위치와 차이를 발견했습니다. DOS EXE7213..7272는 현재 애니메이션 상단을0으로 자른 뒤8픽셀 위, 발의X에 표시합니다. Lemmix의 `x-1,y-FootY-12`와 비교하면 일반 지상 위치에서 오른쪽1/아래4픽셀입니다. MAIN 카운트다운은9..0 저장 순서에서 해당 값을 그대로 선택하고 palette3으로 표시합니다.

폭발 첫 입자 틱에는 실제 DOS의 지형 표시가 삭제보다 한 프레임 늦습니다. 코어 `MotionOutcome.explosion_erased`는 실제 고체→빈 칸 변화만22행/행당16비트로 보고합니다. SDL 세션은 그 픽셀의 이전 지형 재질을 해당 틱의 표시용 마스크에만 보존하며, 시뮬레이션/trace의 지형은 즉시 삭제합니다. 다음 진행 틱에 표시도 갱신합니다. Pause 중에는 이 한 프레임을 유지하고 렌더 빈도로 수명을 결정하지 않습니다. 숫자/입자/클리핑·Pause 표시 지연을 자체 fixture로 검사합니다.

입자 표는 원본 `VGALEMMI.EXE`의 offset49976부터8160바이트이며 고정 [Particles.dat](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Data/Particles/Particles.dat) 전체와 같습니다(SHA-256 `a466969e1fe00205ef0db0d0dfb0fa337f9d05156b9b4dc88c31aeb5468b838f`).51×80개의 signed(dx,dy)이며 어느 성분이든-128이면 미표시합니다. 원본50개를 확보한 환경에서는 기존 EXE에서 읽으므로 추가 다운로드가 필요 없습니다. EXE가 없는 엔진 리소스 구성에서는 입자 그림을 생략하며, EXE가 있지만 크기/MZ 프로필이 다르면 오류입니다.8160바이트 뷰는 no_std 리소스 계층에 있고 화면/팔레트/파일 읽기는 데스크톱이 담당합니다. 원본 파생 데이터는 Git에 넣지 않습니다.

## 재현과 관측 범위

```sh
python3 tools/lemmix/build_bomber_reference.py assets/local/my-bomber-reference --compiler /usr/lib/fpc/3.2.2/ppcx64 --units /usr/lib/fpc/3.2.2/units/x86_64-linux/rtl
python3 tools/check_bomber.py --binary target/release/lemmings-desktop --reference assets/local/my-bomber-reference/reference
make check
SDL_VIDEODRIVER=x11 python3 tools/verify_bomber.py assets/local/my-bomber-native --reference assets/local/my-bomber-reference/reference --xdotool /usr/bin/xdotool
```

`bomber.jsonl`은 수량과 각 ID의 폭발 타이머/입자 타이머/입자 프레임을 기록합니다. 기존 지형 추가/삭제·객체 맵 변경/캐시 기록과 함께 기준 함수 실행에 대조합니다. CI는 실제 실행에서 기록한 합성 golden과 어댑터 해시를 검사하며 golden 갱신에는 실제 `--reference`가 필요합니다. 하네스는 그래픽/사운드/툴바/미니맵/기록기를 스텁 처리하고 전체 Lemmix 실행을 인증하지 않습니다.

원본 Fun06 “A task for blockers and bombers”의50명/20%/RR50/5분·Blocker5/Bomber5·scroll592를 실제 preview/숫자로 확인합니다. F2 후보 `DLCIJNLGCT`는 [직접 역분석한 레벨 코드 설명](https://www.lemmingsforums.net/index.php?topic=7364.0)을 참고해 구한 뒤 원본 진입 상태로 검증합니다. Blocker는DOS counter81의Walking(663,84)에, Bomber는counter101의같은Blocker에 키보드로 할당합니다. 네이티브 입력80/100·테두리16픽셀을 뺀(647,68)과 대응합니다. F5는 선택값4를 관측할 때까지 누른 뒤 놓아 짧은 입력 누락을 피합니다.

```sh
python3 tools/observe_bomber_dos.py --dosbox /usr/bin/dosbox --xdotool /usr/bin/xdotool --output assets/local/my-bomber-dos --seed assets/local/issue-34-dos-proof-1/bootstrap/drive/RUSSELL.DAT --seed-sha256 04462851c5edc273303b6fd19b8ee3c47ef722b9c1803dfd5adbd0229041e313
python3 tools/observe_bomber_dos.py --dosbox /usr/bin/dosbox --xdotool /usr/bin/xdotool --output assets/local/my-bomber-dos-repeat --seed assets/local/issue-34-dos-proof-1/bootstrap/drive/RUSSELL.DAT --seed-sha256 04462851c5edc273303b6fd19b8ee3c47ef722b9c1803dfd5adbd0229041e313
python3 tools/compare_bomber_dos.py --recording assets/local/my-bomber-dos/run assets/local/my-bomber-dos-repeat/run --native assets/local/my-bomber-native/F06-single --output assets/local/my-bomber-comparison
```

32f8에서DS:70 수량을 감소시키고3301에서개체+0x20(DS:a8)에79를 씁니다. Blocker를 유지한 실제 원시 상태는0x8100(OhNo)/0x8102(폭발·입자)이며 제거 바이트255는 입자 단계 끝에 관측됩니다. 이 저장 플래그와 코어의 논리적 폭발 제거/입자 카운터를 동일한 원자적 상태라고 간주하지 않습니다. 관측은 `/proc` 자식 메모리 읽기만 수행합니다. DOSBox0.74-3·VGA/PC2·fixed10000cycles·같은 seed/정규화 설정/mapper/컨트롤러와 서로 다른 녹화 디렉터리를 요구합니다. seed는 #34의 실제 생성 파일 offset0만9→0으로 변경하고 원본50개/EXE·에뮬레이터·게임 메모리는 수정하지 않습니다.

전체320×160·정확한2배 픽셀/스캔라인·RGB565 코드 정규화만 사용합니다. 커서 마스크·위치 이동·색 허용 오차는 없습니다. 비동기 숫자 폴링과 화면 앵커를 전체 DOS 상태/입력 틱/전체 HUD·속도 인증으로 확대하지 않습니다. [보고서](../reports/issue-38_bomber.md)와 [manifest](../../assets/manifests/bomber.json)에 최종 실행/제외 목록과 출처·해시를 기록합니다. Climber/Floater·Nuke·전체 객체/구출/승패·음향·G07 전체는 후속입니다.
