# #169 테마별 HD 자산 로딩

이슈: https://github.com/edwardkim/ohmylemmings/issues/169 · 상위 #137

기준 main8e2497f. 메인테이너 지시로2D 리마스터 우선, BGM #165는 이후 처리한다. 기존27atlas/131699864바이트 팩을 픽셀·좌표 보존한 atlas별 v1 파일로 분리하고 bounded binary catalog가 공통/테마/향후 특수장면 범위를 선택한다. 단일 v1 팩 호환성을 유지한다. 선택 집합은 기존128MiB/32atlas 한도를 유지하며 디스크 전체와 현재 상주 집합을 구분한다.

1. TDD: 카탈로그/선택 팩 경계·손상/중복·잘못된 scope·한도와 원자적 교체.
2. Python 분할·출처/크기/CRC 기록, 공통 parser와 Native/Web 선택 로딩.
3. 테마 전환/실패·실제 출력·원본 규칙 회귀, source audit/docs/CI/self-review/실제 병합.

현재 조사: 혼합 atlas는 전체 페이지를 유지한다. 재배치/잘라내기는 Canvas 샘플 위상을 바꿀 수 있으므로 하지 않는다. 이는 새 그림 제작이 아니라 이후 특수 장면·효과/UI 확장을 위한 로딩 기반이다. 성능 인수/장시간60fps·BGM 수정은 범위 밖이다.

구현/로컬 검증: 현재18~19atlas/94.2~100.5MB 선택,440키/원본 atlas 좌표·RGBA 보존, Native/Web5코스 및 제품 전환/실패 복구 PASS. [결과·한계](../reports/issue-169_hd_loading.md). 소스 감사·CI7개·실제 병합 완료.

완료: PR #170 head `8fb1d0916333200acdf92ad2ce6f21bdd6940e5c` / CI39922131331의7개 SUCCESS, 실제 merge `b7a5e73dfb15c2b7a2171e3740520cfc0d7f4a04`와 검토 head 전체 tree 일치를 확인했다.
