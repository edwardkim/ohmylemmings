# 원본 검증 리소스

DOS 원작 VGA 데이터 프로필의 출처와 파일 식별값을 [매니페스트](manifests/dos_original.json)에 고정합니다. 원본 파일은 `assets/local/dos-original/`에 확보하며 이 폴더는 Git 추적에서 제외됩니다.

## 확보와 검증

Python 3의 표준 라이브러리만 사용합니다. 저장소 루트에서 실행합니다.

```sh
python3 tools/reference_assets.py fetch
python3 tools/reference_assets.py verify
python3 -m unittest discover -s tools/tests -v
```

`fetch`는 고정 커밋의 소스를 내려받아 크기와 SHA-256을 확인한 뒤 파일을 씁니다. `verify`는 로컬 파일 50개를 매니페스트와 비교합니다. 엔진용 리소스 27개와 원작 실행·참고용 파일 23개로 구분됩니다.

별도 폴더에서 재현하려면 `fetch --destination assets/local/reproduced-dos-original`을 사용합니다.

## 기준판 식별

기준 ID는 `dos-original-vga`입니다. 원본 매체의 세부 리비전은 아직 확인되지 않았으며 확보한 파일의 바이트 식별값으로 입력을 고정합니다. 확보 출처, 상호 대조 결과, 데모와의 차이는 [기준판 분석](../mydocs/tech_insights/dos_original_baseline.md)에 기록합니다.
