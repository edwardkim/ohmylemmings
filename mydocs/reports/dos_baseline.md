# G00-01 기준판 식별·원본 확보 결과

- 추적 이슈: [#2](https://github.com/edwardkim/ohmylemmings/issues/2), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1)
- [작업 지시](../orders/dos_baseline.md), [진행 기록](../working/dos_baseline.md), [기준판 분석](../tech_insights/dos_original_baseline.md)

## 결과

원작 프로필 `dos-original-vga`의 파일 50개를 확보했습니다. 엔진 리소스 27개와 원작 실행·참고용 파일 23개이며, 총 831,245바이트입니다. 출처와 파일별 SHA-256은 [매니페스트](../../assets/manifests/dos_original.json)로 고정합니다.

Lemmix 원작 묶음과 보충 출처의 공통 DAT 26개가 바이트 단위로 일치했고, 엔진 리소스 27개는 고정된 Lemmings.ts 크기 목록과 일치했습니다. 별도로 확보한 영어 데모의 데이터 차이도 기록했습니다.

## 검증

| 명령·검사 | 결과 |
|---|---|
| `python3 -m unittest discover -s tools/tests -v` | 6개 통과: ZIP·단일 파일 확보, 손상 입력, 로컬 변경·누락, 잘못된 파일명 |
| `python3 tools/reference_assets.py verify` | 로컬 원본 50개 크기·SHA-256 일치 |
| `python3 tools/reference_assets.py fetch --destination assets/local/reproduced-dos-original` | 새 폴더에 50개 파일 재확보 및 크기·SHA-256 일치 |

- 로컬 문서 링크 53개와 매니페스트의 파일 수·역할·총 바이트 수를 검사했습니다.
- 원본 파일의 Git 추적 제외는 `git check-ignore`로 확인했습니다.
- `git diff --cached --check`를 통과했습니다.
- 커밋 `5288701`을 `main`에 게시하고 원격 파일 25개의 blob 해시가 로컬과 일치함을 확인했습니다.
- 이슈 #2를 완료 처리하고 상위 #1에 연결했습니다. G00 전체는 진행 중입니다.

## 현재 한계와 후속 작업

원본 매체의 세부 리비전과 실제 실행 동작은 검증 전입니다. 파일 식별 기준은 확보한 바이트 프로필로 고정했습니다. 다음 작업 G00-02는 레퍼런스 코드의 역할·함수·버전을 정리하는 것입니다.
