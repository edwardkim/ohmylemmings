# G03-09 MAIN.DAT 마스크·카운트다운 결과

작업 이슈: [GitHub #18](https://github.com/edwardkim/ohmylemmings/issues/18). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

- [형식·API·비트/숫자/위치 차이·원본 대조·재현 명령](../tech_insights/issue-18_main_masks.md)
- [진행 기록](../working/issue-18_main_masks.md), [작업일지](../orders/20261018.md#issue-18)

## 변경 내용

MAIN.DAT 구간 1의 전체 388바이트를 해석했습니다. 바싱 좌우 각 4개·채굴 좌우 각 2개·폭발 1개·숫자 9→0의 글리프 10개, 총 6개 묶음·23프레임을 무할당 API로 제공합니다. 비트 1이 선택됨을 뜻하며 프레임·방향·행·MSB 순서를 보존합니다.

기존 `MainDatFile`에 선언/출력 길이 검사와 호출자 버퍼 해제를 연결했습니다. `MainMasks`·`MaskFrames`·`MaskBitmap`은 정확한 레코드·프레임·좌표·숫자·boolean 출력 길이를 검사합니다. 숫자 값은 9-value로 저장 프레임에 연결합니다. 크기 오류는 출력을 보존하고 압축 명령 오류는 부분 출력 가능성을 명시합니다. 결과는 출력만 빌립니다.

Lemmix가 표시용으로 읽는 5→1 부분 목록과 TS가 읽는 전체 10개 글리프의 차이를 기록했습니다. TS 원시 -128/0은 비트 1/0으로 정규화했고 반대 의미의 `Mask.at`를 복사하지 않았습니다. 위치·타이밍·지형 적용은 후속 컨텍스트에 두며 기존 로컬 의존 하나·외부 패키지 0개·`no_std`·라이브러리 무할당/I/O 경계를 유지했습니다.

호스트 `inspect_masks`, [검증 매니페스트](../../assets/manifests/main_masks.json), 전체 프레임 미리보기와 재현 명령을 추가했습니다. 원본·생성 출력은 Git 추적에서 제외합니다.

## 검증

| 검사 | 결과 |
|---|---|
| 합성 TDD | Red: 새 14개 실패·기존 리소스 151개 통과 → Green: 리소스 165개 통과 |
| `make check` | Rust 단위 167개·문서 예제 13개·Python 8개, 포맷·Clippy·빌드·WASM 성공 |
| 독립 Python | 원본 DAT 직접 해제·기존 7개 구간의 모든 바이트/해시 유지·고정 레이아웃/5→1 부분 목록 대조 |
| 전체 비트 | 23프레임·3104비트(선택 923개)가 Rust·Python·실제 고정 TS의 정규화 원시 데이터와 일치 |
| 실제 TS | 원본 MaskProvider·MaskList·Mask를 실행해 전체 소비·원시 데이터와 반대 극성의 at 조회 확인, 길이 getter의 ReferenceError 기록 |
| 호스트 오류 | 인자·파일·헤더·후미 손상·구간 수·선언 크기·명령·덮어쓰기 거부 10개 경우 통과, 원본·맵·미리보기 보존 |
| 문서 재현 | 기재한 명령 그대로 실행해 기존 7개 구간·원문·23프레임 비트 해시·메타데이터·입력 해시 통과 |
| 문서·의존·추적 | Markdown 69개·로컬 링크 466개·앵커 78개·이슈 문서명 57개·#18 일지·고정 입력 4개·추가 소스 7개 확인, 기존 로컬 의존 하나·외부 패키지 0개 유지 |

전체 카운트다운 글리프는 9→0이며 Lemmix의 5프레임 부분 목록은 5→1입니다. 지원 비트 데이터는 전수 일치하지만 두 레퍼런스의 조회 의미·숫자 인덱싱·위치·타이밍이 같다고 판정하지 않습니다.

## 게시

구현 커밋 `ec57209d124785f406b5759f8a7063cd397e395a`를 `main`에 게시했습니다. [push CI 실행](https://github.com/edwardkim/ohmylemmings/actions/runs/39123578292)의 결론과 세 작업이 모두 success입니다.

| 작업 | 실제 실행 근거 |
|---|---|
| [Rust native](https://github.com/edwardkim/ohmylemmings/actions/runs/39123578292/job/117663981222) | Rust 1.93.1, 포맷·Clippy·빌드, 단위 167개·문서 예제 13개 성공 |
| [Python tooling](https://github.com/edwardkim/ohmylemmings/actions/runs/39123578292/job/117663981290) | Python 3.12.15, 테스트 8개 성공 |
| [WASM libraries](https://github.com/edwardkim/ohmylemmings/actions/runs/39123578292/job/117663981316) | 고정 Rust·WASM 타깃, 두 라이브러리 컴파일 성공 |

push 이벤트·대상 SHA·세 작업의 실제 로그·테스트 개수·clean-input 성공을 대조했습니다. 원본·기존 target·Cargo 패키지 캐시 없는 러너에서 통과했습니다. 원격 tree API의 파일 115개 모드·blob 해시가 로컬과 모두 일치했으며 작업 트리는 깨끗합니다. 원본·PNG·바이너리·JSONL·참조 소스 사본·빌드 출력은 추적하지 않습니다.

G03-09 마스크·카운트다운 해석 범위를 완료했습니다. 최종 문서 게시·CI와 #18 종료는 GitHub 이슈 기록에서 추적합니다. 상위 #1의 G00·G01·G02만 전체 목표 완료로 유지합니다.

## 남은 범위

지형 적용·강철/충돌·행동 틱·표시 위치·게임 화면·실제 DOS 실행은 후속 범위입니다. G03 전체는 객체 이미지 배치·합성이 남아 진행 중이며 다음은 별도 이슈의 G03-10 레벨 객체 이미지 배치·합성입니다. G04-01도 준비 상태입니다.
