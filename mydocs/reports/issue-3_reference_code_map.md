# G00-02 레퍼런스 코드 고정 결과

- 작업 이슈: [#3](https://github.com/edwardkim/ohmylemmings/issues/3), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1)
- [작업일지](../orders/20261018.md#issue-3), [진행 기록](../working/issue-3_reference_code_map.md)
- [코드 지도](../tech_insights/issue-3_reference_code_map.md), [소스 매니페스트](../../assets/manifests/reference_sources.json)

## 산출물

- Lemmix·Lemmings.ts의 커밋 두 개와 참고 역할 고정
- 소스 46개, 원본 446,049바이트의 Git blob SHA-1·SHA-256·인코딩 기록
- 소스 선언 290곳의 줄 번호·접두 문자열 고정
- 13개 영역별 진입점과 직업 8종의 할당·행동 함수 연결
- 틱 시간, 체크섬 오류 처리, 생성·입구, 오디오 경로 차이 기록

## 검증

| 검증 | 결과 |
|---|---|
| `python3 -m unittest discover -s tools/tests -v` | 8개 통과 |
| 소스 위치 검증 TDD | 잘못된 줄 번호 0·1·99에서 오류를 놓치는 Red 확인 후 검증 추가, Green 통과 |
| `python3 tools/reference_assets.py verify --manifest assets/manifests/reference_sources.json` | 46개 파일의 크기·SHA-256과 선언 290곳 일치 |
| 같은 매니페스트로 새 폴더 `fetch` | 파일 46개 재확보 및 선언 검증 통과 |
| `python3 tools/reference_assets.py verify` | 기존 원작 파일 50개 검증 통과 |

로컬 문서 링크 78개, 소스 줄 링크 75개와 원본 Git blob 46개의 해시를 추가로 확인했습니다. `git diff --cached --check`를 통과했고, 게시 커밋 `eef1d59`의 원격 파일 32개가 로컬 blob 해시와 일치함을 확인했습니다. 이후 이슈 #3을 완료 처리하고 상위 #1에 반영했습니다.

참조 프로그램의 빌드·실행과 Rust 포팅의 동작 비교는 이번 정적 분석의 검증 범위에 포함하지 않았습니다.

## 사용자 피드백과 해결 기록

이슈 등록 전 작업 문서 초안을 생성했던 순서를 바로잡았습니다. [이슈 선등록 피드백](../feedbacks/issue-3_issue_first_workflow.md)을 `AGENTS.md`와 작업 흐름에 반영했고, 이슈 #3 조회 확인 후 소스 확보·분석을 수행했습니다.

Contents API의 바이트 불일치는 [전송 문제 해결 기록](../troubleshootings/issue-3_github_source_bytes.md)에 남겼습니다.

## 다음 작업

G00-03에서 전체 지원 범위와 원작 실행·입력·시간·직업별 호환성 비교 시나리오를 정의합니다. G00 전체는 진행 중입니다.
