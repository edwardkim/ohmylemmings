# G11-05f COMP-01~18 증거 재감사

이슈: [#104](https://github.com/edwardkim/ohmylemmings/issues/104), 상위 [#93](https://github.com/edwardkim/ohmylemmings/issues/93) · [계획](../plans/issue-85_compatibility_validation.md) · [진행](../working/issue-104_comp_reaudit.md).

## 판정 범위

기준 main `64dc383ad5da11eb9576aab7d2a3fe8391ee1e7f`의 DOS Original VGA/AdLib 계약에 대해 COMP-01~18을 요구별로 재감사한다. 제품·의존성·게임 규칙은 변경하지 않는다. 새 감사 도구는 증거 파일의 크기/SHA-256, 현재 소스·테스트 함수 연결, 120개 초기 실행과 240개 성공/실패 실행의 입력·원래 목표·결과·렌더/시간 분할·재시작 기록을 검사한다. 이 도구의 PASS는 저장된 증거의 무결성과 범위 판정이며 게임을 새로 실행했다는 뜻은 아니다.

[#85 당시 계획 manifest](../../assets/manifests/compatibility_plan.json)의 미완료 수치와 과거 범위는 보존한다. 현재 감사는 [compatibility_audit.json](../../assets/manifests/compatibility_audit.json), 이번에 다시 실행한 경계는 [compatibility_boundaries.json](../../assets/manifests/compatibility_boundaries.json)에 별도로 고정한다. 상세 요구·현재 테스트 함수·제품 경로·증거 연결은 감사 manifest의 `requirements`에 있다.

## 요구별 근거

| COMP | 재감사한 세부 조건 | 판단 근거 |
|---|---|---|
| 01 | DAT 전체 구간·체크섬·절단·잘못된 복사 | 고정 원본 목록/구간 해시, 현재 DAT/압축 해제 손상·원자성 테스트 |
| 02 | 120개 레벨·ODD40·초기 수치·특수4개·잘못된 로딩 | 카탈로그/ODD 테스트, #89 특수 자산, #91 실제120개 초기 상태·독립 자원 대조, 손상 로딩 원자성 |
| 03 | 지형 덮어쓰기/삭제·팔레트·마스크·추가/삭제 경계 | 전체120개 지형30,412,800픽셀·객체240장면60,825,600픽셀의 독립 정적 증거, 현재 지형/객체 테스트, #91 초기 맵/viewport |
| 04 | 시계·F00 입구 열림·첫 생성 | #26 고정 함수 추적, 현재 clock/startup 테스트와 독립 모델, #91 모든 초기 재생 |
| 05 | 최저 RR/98/99·기존 카운트다운·0~4입구·총원 상한 | 42합성 사례의 기존 기준 golden과 현재 회귀, RR 전수·입구 순서·최대 ID 테스트 |
| 06 | 걷기·반전·턱·낙하/충돌 한계·세계 경계 | motion/climber/floater 경계 테스트, 고정 함수 golden, #91과 유효 완주에서 현재 호스트 상태·맵 대조 |
| 07 | 8기술 정상·거부·재고0·영구 능력/타이머 | 이번 실제 Pascal/Rust 2,640할당, 제거48조합 Rust 회귀, 기술별 경계 테스트 |
| 08 | 같은 경계 다중 할당·지형 공유·Blocker 해제·폭발 | 이번 혼합14사례/28 SDL 재생의 전 경계·맵·입력 수락·재고/능력/타이머 비교와 도달 조건 |
| 09 | 강철·좌우 일방향·Miner 원작 버그·Splat 출구 | 할당 행렬·Miner 진리표/원문 golden·이번 출구 기준 실행; 원본의 superlemming은 전부0으로 옵션 전체를 추정하지 않음 |
| 10 | 0기준 객체 슬롯15/16·함정 경쟁/재사용·물/불·출구 우선순위 | 현재 자원 로더 슬롯 테스트, 이번 출구9/위험물14사례 기준 실행, terminal hazard/폭발 선행·trap busy 경계 |
| 11 | 시간0·목표±1·정수 백분율·최종 결과 고정·재시작 | `level_outcome`의 작은 분모 전수와 넓은 곱셈 경계, 이번 승패11사례 기준 실행, #52 DOS 최종 수치의 역사적 관측, 전체120개 양 플랫폼 결과/Restart |
| 12 | Pause 중 시각·입구·PauseGlitch 양 설정 | startup/stepper 전수 모델 및 기존42사례 기준 golden, 이번 혼합/출구/승패의 실제 Pause 경계 |
| 13 | Nuke 분모·중복·제거/타이머/개별 순서·생성 중단·NukeGlitch 양 설정 | 현재 nuke/level_outcome 테스트와 기존 기준 golden, 이번 혼합/승패 함수 실행, 전체120개 실패/성공의 원래 목표와 분모 구분 |
| 14 | 겹침 우선순위·우클릭 원격·함정 제거 대상·원시 호스트 입력 | 이번 48,494행 실제 기준 probe, 현재 SDL raw pointer 테스트, #58 X11·#60 SDL·#70 실제 DOM와 현재 Web journey 회귀; 안정 ID 입력과 별도 집계 |
| 15 | 같은 입력30/60/144Hz·실제 Canvas/SDL 읽기·숨김/복귀·원자적 실패 | #91의120개 초기3회, #94/#96/#98/#100/#102의240사례 각3회와 실제 Restart·숨김 복귀; 현재 DOM/FramePump 회귀 |
| 16 | 음악21/효과음18·원본 driver·PCM·동시 사건·정지/세대·장치·특수/순차 음악 | #72~82의 원본 x86 227,615호출·독립 C/native/WASM PCM·580경계 통합과 사용자 청취 통과, 현재 rank/special/Continue·device 회귀 |
| 17 | Fun/Tricky/Taxing/Mayhem 각30개 로딩·성공·실패·재시작 | 초기120·성공120·실패120의 중복/누락0 집계, 각 입력 SHA·카탈로그 원래 총원/필요 수·실제 결과 연결 |
| 18 | F01 실제 완주·동일 입력3회·출처/소스·재현성 | #60/#70의 원시 입력 한 레벨 경로와 전체240 안정 ID 경로의 실제 SDL/WASM 기록을 구분, 현재 소스 일치·원본/입력/증거 해시·3분할 결과 비교 |

## 변경 영향과 증거 보충

#91 검증 head `24dae2076581e03bf6adcc43fc58dd3e6cc53e61`부터 현재 기준까지 `apps/`, `crates/`, `Cargo.toml`, `Cargo.lock`의 제품 바이트 변경은 없다. 각 완주 batch의 캡처 시점 제품 소스도 현재 바이트와 대조한다. #96의 브라우저 장시간 실행/heartbeat 도구 변경은 각 batch가 기록한 버전을 유지하며 비교 판정을 완화하지 않는다. 따라서 통과한120레벨 완주를 다시 전부 실행하지 않는다.

이전 PC에서 만든 일부 #31/#32/#33/#46/#48/#50/#52 ignored 원시 파일은 현재 PC에 없다. 이 파일들을 새로 해시 검증했다고 주장하지 않는다. 공개 manifest의 과거 관측과 한계는 그대로 남기고, 현재 소스에 필요한 할당·혼합·출구·위험물·승패 경계는 고정 원문의 실제 함수 실행으로 보충한다. #70 manifest가 가리키는 `assets/local/web/pkg/` 네 파일은 이후 빌드로 교체되는 작업 경로이므로 과거 해시와 현재 바이트가 다른 사실도 구분한다. 현재 WASM은 완주 batch와 정확한 CI의 해시를 사용한다. 외부 레퍼런스의 저장소 내부 경로도 현재 프로젝트 경로의 누락으로 계산하지 않는다.

이번 실행은 기존 입력·어댑터·golden을 변경하지 않고 `--reference`로 수행한다. `--record`로 새 결과에 맞춰 기대값을 덮어쓰지 않는다. 새 실행의 명령·레퍼런스 함수 해시·요약·개별 원시는 ignored `assets/local/issue-104-review/`에 보존한다. 과거 DOS 영상/비동기 RAM 결과가 현재 PC에서 없는 경우에는 역사적 관측으로만 인용하며 새 DOS 관측 횟수에 더하지 않는다.

## 현재 실행 결과

- 18개 COMP의 **70개 세부 요구**, 현재 소스/fixture338개와 테스트 함수107연결(고유100개)을 검사했다.
- 초기120개 실제 SDL/WASM 각360회와 성공/실패240사례를 중복·누락 없이 집계했다. 완주/실패는 플랫폼별720회와 Restart720회이며 기준 함수318,574상태, SDL958,602probe/WASM1,917,204probe다. SDL stream495,960·Restart196,380·Canvas780,916readback 기록을 연결했다.
- 이번 보충은 할당2,640행·포인터48,494행과 **104 SDL 재생·40,736상태·5,698정확 캡처**를 통과했다. 기존 golden/비교 조건은 유지했다.
- 새 감사 도구의 TDD5개와 전체 Python183개, 독립 motion fixture를 통과했다. 누락/중복 요구·변경된 증거/소스·빠진 레벨/반복·입력/목표/결과 변조·잘못된 Restart 대상·두 번째 재완주 차이·집계 오류를 거부한다.
- 보존된 완주 원시 **64,080파일**의 크기/SHA를 재확인했다. 이전 검증 후 제거한 readback은 과거 기록으로만 남기며, 이번 새 기준/SDL 원시 **14,972파일**의 별도 해시 목록을 고정했다.
- 제품은 그대로이므로 이미 통과한 기준 main의 Rust611·문서29·SDL·WASM·실제 Web/audio 회귀를 재사용한다. 새 감사 도구를 포함한 게시 커밋의 전체 CI는 별도로 확인한다.

## 재현과 제한

```sh
source assets/local/dev-tools/env.sh
python3 tools/compatibility_audit.py
python3 tools/compatibility_audit.py --local
python3 -m unittest tools.tests.test_compatibility_audit -v
make check-python
```

기본 명령과 단위 검사는 원본/FPC 없이 공개 입력·증거·소스만 사용한다. `--local`은 manifest에 고정한 ignored 파일도 요구한다. 경계 실제 재실행은 `compatibility_boundaries.json`의 각 `commands`를 새 ignored 출력 경로로 실행한다. FPC3.2.2·고정 원문과 릴리스 SDL 실행 파일이 필요하다. 전체120개 재현은 #91/#94~102의 기존 명령/입력과 원본 자산을 따른다.

새 레퍼런스 실행은 고정 Lemmix의 추출 함수/표와 문서화한 호스트 어댑터의 지원 필드만 비교한다. 전체 DOS 내부 상태의 원자적 추적이나 모든 레벨의 새 DOS 플레이 관측이 아니다. 원시 DOM/SDL 포인터 사례와 안정 actor ID 리플레이를 같은 범위로 세지 않는다. SDL Restart는 초기 자원·이전 입력 폐기·입력 없는 prefix이고 WASM Restart는 같은 입력의 실제 재완주다. PCM 비교는 고정 Nuked 엔진 범위이며 DBOPL 파형과 모든 물리 DAC/브라우저/장치를 인증하지 않는다.

성능 측정은 G11-06이다. CI/캡처/파일 쓰기 벽시계 시간을 게임 FPS로 보고하지 않는다. G11 전체 마감과 G12 배포는 아직 후속이다. 정확한 최종 실행 집계·self-review·게시 head/병합 트리·push/PR/main CI·마감은 [#104](https://github.com/edwardkim/ohmylemmings/issues/104)의 최종 기록에 고정한다.
