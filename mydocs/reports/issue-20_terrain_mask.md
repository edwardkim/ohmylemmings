# G04-01 픽셀 지형 읽기·경계 결과

작업 이슈: [GitHub #20](https://github.com/edwardkim/ohmylemmings/issues/20). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

- [표현·API·경계·합성 전수 대조·재현 명령](../tech_insights/issue-20_terrain_mask.md)
- [진행 기록](../working/issue-20_terrain_mask.md), [작업일지](../orders/20261018.md#issue-20)

## 변경 내용

코어에 호출자 0/1 바이트를 빌리는 읽기 전용 `TerrainMask`를 도입했습니다. 빈 크기·면적 오버플로·정확한 길이·전체 셀 값을 생성 시 검증합니다. 일반 조회는 내부 빈 셀과 외부를 구분하며 부호 있는 지형 조회는 음수·외부를 false로 반환합니다. 원문·행·크기를 복사 없이 제공합니다.

색상·렌더링·리소스 의존성을 추가하지 않았습니다. 지형 크기를 표시 폭으로 고정하지 않으며 외부 조회로 개체 생존/벽 규칙을 결정하지 않습니다. `TerrainMaskError`에 첫 오류 셀과 크기 정보를 보존합니다. `no_std`·무할당/I/O·안전한 Rust와 기존 리소스→코어 의존 하나·외부 패키지 0개를 유지했습니다.

## 검증

| 검사 | 결과 |
|---|---|
| 합성 TDD | 새 12개 Red·기존 코어 2개/리소스 181개 통과 → Green 코어 14개/리소스 181개 |
| `make check` | Rust 단위 195개·문서 예제 15개·Python 8개, 포맷·Clippy·빌드·WASM 컴파일 성공 |
| 경계·손상 | 빈 크기·checked 면적·길이 우선·모든 비이진 값·비대칭 배치·행/열·canary·i32/usize 극단 값·원문 불변 확인 |
| 합성 전수 | 1~3 폭/높이의 모든 이진 마스크 682개·15970개 좌표를 독립 좌표 집합과 대조 |
| 문서·경계 검사 | Rust/TS 재현 명령 그대로 실행, Markdown 75개·로컬 링크 513개·앵커 86개·이슈 문서명 63개·#20 일지·코어 의존성 0개 확인 |
| 실제 TS | 고정 SolidLayer 원문 실행이 같은 좌표 집합과 일치. Rust/TS 조회·행 스트림 서명 `2eeedec0fe771b08` 일치 |

기존 소스 46개의 크기·SHA-256을 검증했습니다. 원본 게임 파일은 필요 없으며 원본·기존 그래픽 API·매니페스트를 변경하지 않았습니다. TS의 임의 비영 값 허용·없는 버퍼 처리와 Rust의 엄격한 생성 검증 차이를 기술 문서에 기록했습니다.

## 게시 상태

구현 커밋 `c81bae0571e3dde0405d0f5b7bdbfde7cac67f9b`를 main에 게시했습니다. [push CI 실행](https://github.com/edwardkim/ohmylemmings/actions/runs/39126059271)의 결론과 세 작업이 모두 success입니다.

| 작업 | 실제 실행 근거 |
|---|---|
| [Python tooling](https://github.com/edwardkim/ohmylemmings/actions/runs/39126059271/job/117671321826) | Python 3.12.15, 테스트 8개 성공 |
| [Rust native](https://github.com/edwardkim/ohmylemmings/actions/runs/39126059271/job/117671321900) | Rust 1.93.1, 포맷·Clippy·빌드·단위 195개·문서 예제 15개·전수 마스크 테스트 성공 |
| [WASM libraries](https://github.com/edwardkim/ohmylemmings/actions/runs/39126059271/job/117671321655) | 고정 Rust·WASM 타깃, 두 라이브러리 컴파일 성공 |

push 이벤트·대상 SHA·실제 로그·테스트 개수·clean-input 성공을 대조했습니다. 원본·기존 target·Cargo 패키지 캐시 없는 러너에서 통과했습니다. 원격 tree API의 파일 125개 모드·blob 해시가 로컬과 일치했고 작업 트리는 깨끗합니다. 원본·참조 소스 사본·빌드 출력을 추적하지 않습니다.

G04-01 범위를 완료했습니다. 최종 문서 게시·CI·상위 #1 갱신·#20 종료는 GitHub 이슈 기록에서 추적합니다.

## 다음 범위

G04-01 읽기·경계를 완료했습니다. G04 전체는 쓰기·추가/삭제 검증까지 진행 중이며 다음 구현은 새 이슈의 **G04-02 지형 쓰기·셀 변경**입니다. 강철·객체·게임 경계/생존 규칙·리소스 연결·게임 틱·실제 DOS/WASM 실행은 후속 범위입니다.
