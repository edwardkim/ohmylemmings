# G04-03 영역·마스크 기반 지형 추가/삭제 결과

작업 이슈: [GitHub #22](https://github.com/edwardkim/ohmylemmings/issues/22). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

- [API·클리핑·변경 경계·전수 대조·재현 명령](../tech_insights/issue-22_terrain_bulk.md)
- [진행 기록](../working/issue-22_terrain_bulk.md), [작업일지](../orders/20261018.md#issue-22)

## 변경 내용

코어 가변 마스크에 `fill_rect`·`apply_mask`를 추가했습니다. 사각형 또는 마스크 1인 셀만 0/1로 변경하며 마스크 0은 보존합니다. 음수 배치에서 원본 정렬을 유지하고 네 경계에서 자릅니다. 원점+길이·요청 면적·최소 음수의 부호 반전을 계산하지 않아 극단 값도 안전하게 처리합니다.

`TerrainChange`·`TerrainRect`는 실제 변경 수와 변경 셀의 최소 경계를 반환합니다. 빈/외부/반복 연산은 0/None입니다. 소스는 불변이고 소스/목적지 별칭은 컴파일 단계에서 거부합니다. 기존 읽기/셀 쓰기/오류 API와 0/1 불변식을 유지했습니다. 코어의 `no_std`·무할당/I/O·안전한 Rust·의존성 0개와 리소스→코어 의존 하나를 유지합니다.

## 검증

| 검사 | 결과 |
|---|---|
| 합성 TDD | 새 14개 Red·기존 코어 26개/리소스 181개 통과 → Green 코어 40개/리소스 181개 |
| `make check` | Rust 단위 221개·문서 테스트 20개(compile-fail 2개 포함)·Python 8개, 포맷·Clippy·빌드·WASM 컴파일 성공 |
| 전수 대조 | 사각형 18944건·마스크 30784건, 총 49728건의 전체 버퍼/변경 수/최소 경계를 좌표 집합 대칭 차로 대조 |
| 실제 TS | 선택/클리핑 하네스의 유효 셀 쓰기 12628번, 쓰기 전/후 스트림 서명 f4207a057fb5986b가 Rust와 일치 |
| 문서·추적 규칙 | Markdown 81개·로컬 링크 557개·앵커 95개·이슈 문서명 69개·#22 일지 확인 |
| 문서 재현·의존 경계 | 두 sh 블록 그대로 실행·서명 일치, 패키지 2개·코어 의존 0개·로컬 의존 하나 확인 |
| 경계·대여 | 음수/네 경계·소스 정렬·0 보존·i32 최소/최대·usize 최대·canary·호출 순서·불변 소스·별칭 거부 확인 |

기존 소스 46개와 SolidLayer 크기·SHA-256·git blob을 확인했습니다. 원본 게임 파일은 필요 없으며 기존 리소스 코드·입력 매니페스트·의존 방향을 변경하지 않았습니다. TS는 벌크 연산·메타데이터를 제공하지 않아 하네스가 선택/클리핑과 변경 측정을 담당한다는 비교 범위를 기록했습니다.

## 게시 상태

구현 커밋 `cf49797f1622e64f80fe8b8aab0d704e7df0c8a7`을 main에 게시했습니다. [push CI 실행](https://github.com/edwardkim/ohmylemmings/actions/runs/39128850386)의 결론과 세 작업이 모두 success입니다.

| 작업 | 실제 실행 근거 |
|---|---|
| [Python tooling](https://github.com/edwardkim/ohmylemmings/actions/runs/39128850386/job/117679649563) | Python 3.12.15, 테스트 8개 성공 |
| [Rust native](https://github.com/edwardkim/ohmylemmings/actions/runs/39128850386/job/117679649593) | Rust 1.93.1, 포맷·Clippy·빌드·단위 221개·문서 테스트 20개·전수 변형/별칭 거부 성공 |
| [WASM libraries](https://github.com/edwardkim/ohmylemmings/actions/runs/39128850386/job/117679649433) | 고정 Rust·WASM 타깃, 두 라이브러리 컴파일 성공 |

push 이벤트·대상 SHA·실제 로그·테스트 개수·전수/compile-fail·clean-input 성공을 대조했습니다. 원본·기존 target·Cargo 패키지 캐시 없는 러너에서 통과했습니다. 원격 tree API의 파일 132개 모드·blob 해시가 로컬과 일치했고 작업 트리는 깨끗합니다. 원본·참조 소스 사본·빌드 출력을 추적하지 않습니다.

G04-03과 G04 전체 완료 기준을 충족했습니다. 최종 문서 게시·CI·상위 #1 갱신·#22 종료는 GitHub 이슈 기록에서 추적합니다.

## G04 완료 판단과 다음 범위

#20 읽기·경계, #21 셀 쓰기와 이번 영역/마스크 변형의 픽셀 단위 fixture·게시·실제 CI로 G04 전체를 완료했습니다. COMP-03 통합·강철/행동·게임 경계/생존·리소스 연결·틱·실제 DOS/WASM 실행은 후속 범위입니다.

다음 구현은 새 이슈의 **G05-01 결정론적 고정 틱·입력 순서 골격**입니다.
