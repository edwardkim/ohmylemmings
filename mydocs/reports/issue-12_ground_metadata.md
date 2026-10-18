# G03-03 GROUND 메타데이터·팔레트 결과

작업 이슈: [GitHub #12](https://github.com/edwardkim/ohmylemmings/issues/12). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

- [형식·API·검증 정책·레퍼런스 차이·재현 명령](../tech_insights/issue-12_ground_metadata.md)
- [진행 기록](../working/issue-12_ground_metadata.md), [작업일지](../orders/20261018.md#issue-12)

## 변경 내용

`GroundRecord`는 압축하지 않은 1056바이트 GROUND 파일을 빌려 객체 16슬롯·지형 64슬롯과 EGA/VGA 세 팔레트 뱅크를 제공합니다. 비정렬 little-endian 필드·미해석 값·빈 슬롯 번호·원문 전체를 보존합니다. 길이와 전체 96바이트의 여섯 비트 색상 범위를 검증하고 오류 위치를 제공합니다.

객체 마스크는 프레임 기준 상대 주소, 지형 마스크는 절대 주소로 구분합니다. 루프 비트와 트리거 픽셀 좌표 계산을 제공하며, VGA 원값과 TS의 4배 확대·Lemmix의 전 범위 확대를 명시적으로 분리합니다. `inspect_ground` 예제는 모든 필드와 팔레트를 JSONL로 출력합니다. 외부 패키지·라이브러리 할당·I/O를 추가하지 않았습니다.

## 검증

| 검사 | 결과 |
|---|---|
| 골격·합성 테스트 12개 | Red: 새 12개 실패, 기존 리소스 67개 통과 |
| 구현 후 리소스 테스트 | Green: 79개 통과 |
| `make check` | 전체 Rust 단위 81개·문서 예제 7개·Python 8개, 빌드·포맷·Clippy·WASM 성공 |
| 원본·소스 `verify` | 50개·46개 파일 크기·SHA-256 통과 |
| 원본 5개 독립 대조 | 객체 80슬롯·지형 320슬롯의 모든 필드·빈 슬롯·루프·트리거 일치 |
| 전체 팔레트 대조 | EGA 선택자 120개·VGA RGB 120색 원값과 두 확대 방식 모두 일치 |
| 문서의 재현 코드 그대로 실행 | 전체 슬롯·팔레트 대조와 사용 개수 54/273 확인 |
| 예제 오류 | 인자 누락, 짧은/긴 파일, EGA/VGA 손상에서 경로·오류 문맥과 실패 종료 확인 |
| 문서·Cargo·공백·미완성 코드 | Markdown 51개, 로컬 링크 312개·앵커 52개, 이슈 문서명 39개·#12 일지 통과; 기존 로컬 의존 하나·외부 패키지 0개 유지; Rust stub 없음 |

합성 검증은 모든 짧은 길이·추가 바이트, 팔레트 전체 위치의 64/255 거부, 비정렬 LE word, 마지막 슬롯·`usize::MAX`, fused iterator, 빈 슬롯의 미해석 값 보존, 트리거 top=-4·최대 좌표, 모든 64개 VGA 성분의 두 확대 공식을 포함합니다.

## 원본과 한계

| 세트 | 파일 크기 | 사용 객체 / 16 | 사용 지형 / 64 |
|---|---:|---:|---:|
| 0 | 1056 | 11 | 50 |
| 1 | 1056 | 11 | 64 |
| 2 | 1056 | 10 | 60 |
| 3 | 1056 | 11 | 62 |
| 4 | 1056 | 11 | 37 |

합계는 사용 객체 54개·지형 273개이며 나머지도 원래 슬롯으로 접근됩니다. 원본 VGA 성분 최댓값은 60입니다. 원본·기존 매니페스트는 변경하지 않았습니다.

메타데이터 파싱은 이미지의 유효성을 인증하지 않습니다. 치수·프레임·주소·마스크 결합 검증과 VGAGR 픽셀 복원은 다음 단계입니다. 알파·투명도·최종 렌더러 확대 방식과 DOS 실행 호환성도 후속 범위입니다. G03 전체는 진행 중입니다.

## 게시

구현 커밋 `799a4b00d4d4d3c547388613814f2a8fd34e4b49`을 `main`에 게시했습니다. [push CI 실행](https://github.com/edwardkim/ohmylemmings/actions/runs/39113258808)의 결론과 세 작업이 모두 success입니다.

| 작업 | 실제 실행 근거 |
|---|---|
| [Rust native](https://github.com/edwardkim/ohmylemmings/actions/runs/39113258808/job/117633680603) | Rust 1.93.1, 포맷·Clippy·빌드, 단위 81개·문서 예제 7개 성공 |
| [Python tooling](https://github.com/edwardkim/ohmylemmings/actions/runs/39113258808/job/117633680679) | Python 3.12.15, 테스트 8개 성공 |
| [WASM libraries](https://github.com/edwardkim/ohmylemmings/actions/runs/39113258808/job/117633680504) | 고정 Rust·WASM 타깃, 두 라이브러리 컴파일 성공 |

push 이벤트·대상 SHA·세 작업·실제 로그의 테스트 개수와 clean-input 성공을 확인했습니다. CI는 원본·기존 target·Cargo 패키지 캐시 없는 러너에서 통과했습니다. 원격 tree API의 파일 79개 모드·blob 해시가 로컬과 모두 일치했으며 원본·JSONL·손상 샘플·빌드 출력은 추적하지 않습니다.

G03-03 메타데이터·팔레트 범위를 완료했습니다. 상위 #1의 전체 G03는 이미지 해석·지형 재구성이 남아 진행 중이며 G00·G01·G02만 완료 상태를 유지합니다. 최종 문서 게시·CI와 #12 종료는 GitHub 이슈 기록에서 추적합니다. 다음은 별도 이슈로 진행할 G03-04 VGAGR 비트플레인·마스크 해석입니다.
