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

## 레퍼런스 소스

[소스 매니페스트](manifests/reference_sources.json)는 Lemmix·Lemmings.ts의 고정 파일과 선언 위치를 기록합니다. 같은 도구에서 `--manifest assets/manifests/reference_sources.json`을 지정해 `fetch` 또는 `verify`를 실행합니다. 소스는 `assets/local/reference-sources/`에 저장합니다.

파일·함수의 역할과 후속 구현에서 읽을 순서는 [레퍼런스 코드 지도](../mydocs/tech_insights/issue-3_reference_code_map.md)를 참고합니다.

## 호환성 비교 목록

[호환성 매니페스트](manifests/dos_compatibility.json)는 120개 레벨의 DAT/ODD 연결·초기 조건, 해제 구간 81개의 크기·SHA-256, 음악 21개·효과음 18개의 주소를 기록합니다. `reference_assets.py`의 확보 매니페스트와는 용도가 다르므로 `fetch --manifest` 입력으로 사용하지 않습니다.

[비교 기준](../mydocs/tech_insights/issue-5_compatibility_baseline.md)에 따라 G02~G11에서 기대값과 실제 동작을 대조합니다. 현재 목록은 바이트·정적 구조 검증 결과이며 실행 호환성 통과 표가 아닙니다.

Rust 해제 예제와 기존 81개 구간의 해시 대조는 [G02-02 재현 명령](../mydocs/tech_insights/issue-9_dat_decompression.md#개발용-해제와-고정-해시-재현)을 따릅니다. 해제 결과도 `assets/local/` 안에 저장하며 기존 결과는 덮어쓰지 않습니다.

## 기준판 식별

기준 ID는 `dos-original-vga`입니다. 원본 매체의 세부 리비전은 아직 확인되지 않았으며 확보한 파일의 바이트 식별값으로 입력을 고정합니다. 확보 출처, 상호 대조 결과, 데모와의 차이는 [기준판 분석](../mydocs/tech_insights/issue-2_dos_original_baseline.md)에 기록합니다.
