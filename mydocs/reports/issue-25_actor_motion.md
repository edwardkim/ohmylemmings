# G05-03 개체 이동·제거 결과

작업 이슈: [GitHub #25](https://github.com/edwardkim/ohmylemmings/issues/25). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

- [계약·순서·경계·독립 모델·재현 명령](../tech_insights/issue-25_actor_motion.md)
- [진행 기록](../working/issue-25_actor_motion.md), [작업일지](../orders/20261018.md#issue-25)

## 변경 내용

직업이 없는 Actor의 Walking/Jumping/Falling/Splatting·프레임·발 좌표·방향 전환·제거를 구현했습니다. 생성 요청을 입구+(24,14)의 오른쪽 Faller로 연결하고 같은 생성 틱에 갱신합니다. fallen 60/61 경계와 16프레임 후 충돌 사망 제거를 보존하며 객체 검사 가능 여부를 반환합니다.

호출자 소유 actor 슬라이스는 엄격한 ID 순서로 처리하고 제거된 기록을 보존합니다. 전체 사전 검사 후 갱신해 잘못된 순서·좌표/누적값 오류에서 배치와 콜백을 보존합니다. 지형은 읽기 전용입니다. 코어 no_std·무할당/I/O·외부 의존 0개와 리소스→코어 로컬 의존 하나를 유지했습니다.

## 검증

| 검사 | 결과 |
|---|---|
| 합성 TDD | 새 19개 모두 Red·기존 260개 통과 → Green 코어 98개/리소스 181개 |
| make check | Rust 단위 279개·문서 테스트 27개(기존 compile-fail 2개 포함)·Python 8개/독립 모델 golden 검사·포맷·Clippy·빌드·WASM 성공 |
| 이동 전수 | 322560전이의 전체 이동 상태/결과를 직렬화, 1024그룹 독립 golden·전체 서명 a04dc778204e5a15 일치 |
| 생성/이동 통합 | 36사례·세 패턴 108재생·36720제어 호출, 전체 이벤트/상태와 Python 서명 4821a442b91d9b8b 일치 |
| 생성 순서 | 생성 54의 같은 틱에서 y14→17·fallen3→6·frame0→1 확인 |
| 문서·의존 경계 | Markdown 90개·로컬 링크 630개·앵커 108개·이슈 문서명 78개·#25 일지, 코어 의존 0개/로컬 의존 하나 확인 |
| 재현·CI 연결 | 세 sh 블록 그대로 실행, 고정 소스 46개/추가 메타데이터 검증·Python CI 명령의 unittest 8개와 독립 golden 통과 |
| 예외/경계 | 턱/점프/빈틈·60/61·ClipY/상하좌우·최대 좌표/누적값·복원·배치 오류 보존·지연 제거/중복 방지 |

고정 Lemmix 이동·전환·메타데이터와 TS 차이를 정적으로 분석했습니다. 독립 합성 모델을 대조했으며 실제 레퍼런스 엔진/DOS·출구/직업·플랫폼 동작을 인증하지 않습니다. Actor/시작 상태의 테스트 바이트는 전체 trace-v1 구현이 아닙니다.

## 게시 상태와 다음 범위

구현 커밋 `e21bd56803f92f27610754e8e3c988fdc05c9a51`을 main에 게시했습니다. [push CI 실행](https://github.com/edwardkim/ohmylemmings/actions/runs/39133341616)의 결론과 세 작업이 모두 success입니다.

| 작업 | 실제 실행 근거 |
|---|---|
| [Python tooling](https://github.com/edwardkim/ohmylemmings/actions/runs/39133341616/job/117692800495) | Python 3.12.15, 도구 unittest 8개와 독립 모델의 전체 golden/통합 서명 성공 |
| [Rust native](https://github.com/edwardkim/ohmylemmings/actions/runs/39133341616/job/117692800546) | Rust 1.93.1, 포맷·Clippy·빌드·단위 279개·문서 27개·전수/통합·생성 틱/오류 보존·기존 별칭 거부 성공 |
| [WASM libraries](https://github.com/edwardkim/ohmylemmings/actions/runs/39133341616/job/117692800324) | 고정 Rust·WASM 타깃, 두 라이브러리 컴파일 성공 |

push 이벤트·대상 SHA·실제 로그·테스트 개수·두 이동/통합 서명·기존 compile-fail·clean-input 성공을 대조했습니다. 원본·기존 target·Cargo 패키지 캐시 없는 러너에서 통과했습니다. 원격 tree API의 파일 151개 모드·blob 해시가 로컬과 일치했고 작업 트리는 깨끗합니다. 원본·참조 소스 사본·빌드 출력을 추적하지 않습니다.

G05-03 범위를 완료했습니다. 최종 문서 게시·CI·상위 #1 갱신·#25 종료는 GitHub 이슈 기록에서 추적합니다.

G05 전체·COMP-06은 상태 추적/기준 실행 대조 등이 남아 진행 중이며 다음은 별도 이슈의 **G05-04 상태 추적·재생·기준 실행 대조**입니다.
