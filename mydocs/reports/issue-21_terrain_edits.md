# G04-02 지형 쓰기·셀 변경 결과

작업 이슈: [GitHub #21](https://github.com/edwardkim/ohmylemmings/issues/21). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

- [쓰기·결과·대여 계약·전수 대조·재현 명령](../tech_insights/issue-21_terrain_edits.md)
- [진행 기록](../working/issue-21_terrain_edits.md), [작업일지](../orders/20261018.md#issue-21)

## 변경 내용

코어 `TerrainMaskMut`가 기존 읽기 전용 마스크의 생성 검증을 재사용하며 호출자 버퍼를 독점 대여합니다. bool 기반 `set_pixel`·`set_solid`는 변경/유지/외부를 구분하고 범위 밖 쓰기는 모든 셀을 보존합니다. 원시 가변 바이트를 노출하지 않아 0/1 불변식을 유지합니다.

`as_read_only`는 짧은 읽기 재대여, `into_read_only`는 소비 후 원래 버퍼 수명의 읽기 전환입니다. 읽기 뷰 사용 중 쓰기 금지는 compile-fail 문서 테스트로 검증합니다. 기존 읽기 API·오류 계약을 유지하고 코어 의존성·할당/I/O·unsafe·렌더링/리소스 결합을 추가하지 않았습니다.

## 검증

| 검사 | 결과 |
|---|---|
| 합성 TDD | 새 12개 Red·기존 코어 14개/리소스 181개 통과 → Green 코어 26개/리소스 181개 |
| `make check` | Rust 단위 207개·문서 테스트 17개(compile-fail 1개 포함)·Python 8개, 포맷·Clippy·빌드·WASM 컴파일 성공 |
| 합성 전수 | 682개 마스크·79850번 시도(유효 쓰기 27530번)의 결과·전체 버퍼·복원을 독립 좌표 집합과 대조 |
| 실제 TS | 유효 좌표 set/clear 원문 실행 + 외부 거부 정책 스트림이 Rust와 일치. 서명 03ec6f2a34e44882 |
| 문서·의존 경계 | Rust/TS 명령 그대로 실행, Markdown 78개·로컬 링크 534개·앵커 90개·이슈 문서명 66개·#21 일지·코어 의존성 0개 확인 |
| 경계·대여 | 극단/음수/행 경계·canary·실패 후 버퍼 재사용·독립 하위 슬라이스·읽기 전환/별칭 거부 확인 |

기존 소스 46개와 SolidLayer 크기·SHA-256·git blob을 확인했습니다. TS의 `(3,0)`·`(-1,1)` 외부 쓰기가 3×2 버퍼의 다른 행을 바꾸는 차이를 기록했으며 Rust는 둘 다 거부합니다. 원본 게임 파일은 필요 없고 기존 리소스 코드·입력 매니페스트·의존 방향은 변경하지 않았습니다.

## 게시 상태

구현 커밋 `3972d3f18e87f6c65c374a38936c44b753fee581`을 main에 게시했습니다. [push CI 실행](https://github.com/edwardkim/ohmylemmings/actions/runs/39127650752)의 결론과 세 작업이 모두 success입니다.

| 작업 | 실제 실행 근거 |
|---|---|
| [Python tooling](https://github.com/edwardkim/ohmylemmings/actions/runs/39127650752/job/117676054612) | Python 3.12.15, 테스트 8개 성공 |
| [Rust native](https://github.com/edwardkim/ohmylemmings/actions/runs/39127650752/job/117676054905) | Rust 1.93.1, 포맷·Clippy·빌드·단위 207개·문서 테스트 17개·전수 변경/별칭 거부 성공 |
| [WASM libraries](https://github.com/edwardkim/ohmylemmings/actions/runs/39127650752/job/117676054892) | 고정 Rust·WASM 타깃, 두 라이브러리 컴파일 성공 |

push 이벤트·대상 SHA·실제 로그·테스트 개수·전수/compile-fail·clean-input 성공을 대조했습니다. 원본·기존 target·Cargo 패키지 캐시 없는 러너에서 통과했습니다. 원격 tree API의 파일 128개 모드·blob 해시가 로컬과 일치했고 작업 트리는 깨끗합니다. 원본·참조 소스 사본·빌드 출력을 추적하지 않습니다.

G04-02 범위를 완료했습니다. 최종 문서 게시·CI·상위 #1 갱신·#21 종료는 GitHub 이슈 기록에서 추적합니다.

## 다음 범위

G04-02 단일 셀 쓰기를 완료했습니다. G04 전체는 일괄 변형 검증까지 진행 중이며 다음 구현은 새 이슈의 **G04-03 영역·마스크 기반 지형 추가/삭제**입니다. 클리핑·강철/객체·게임 경계/생존 규칙·리소스 연결·틱·실제 DOS/WASM 실행은 후속 범위입니다.
