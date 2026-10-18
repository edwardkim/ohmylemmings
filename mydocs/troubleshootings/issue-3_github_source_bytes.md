# GitHub 소스 응답의 원본 바이트 불일치

관련 작업: [G00-02 / 이슈 #3](https://github.com/edwardkim/ohmylemmings/issues/3).

## 증상과 확인

고정 커밋의 Delphi 파일을 Contents API의 Base64 응답으로 확보했을 때 메타데이터의 크기·Git blob 해시와 실제 디코딩한 바이트가 달랐습니다.

대상: `ericlangedijk/Lemmix`, 커밋 `40e9bc34451f0e9127fd53b290a3877e700d143d`, `src/Dos.Consts.pas`.

| 경로 | 바이트 수 | Git blob SHA-1 |
|---|---:|---|
| 저장소 메타데이터 | 16,284 | `86c27941207e076569ac3d3463f71f5890421df6` |
| Contents API 응답의 디코딩 결과 | 16,285 | `ee12ef76f2c419a7a1a212b2690416e33c4f1bb3` |
| 원본 다운로드 및 Git blob API | 16,284 | `86c27941207e076569ac3d3463f71f5890421df6` |

전송 경로에서 바이트 표현이 달라지는 현상을 확인했습니다. 정확한 변환 원인은 확정하지 않았습니다. 원본은 CP1252로 해석할 수 있으며, 해시 계산 전에 텍스트로 변환하면 바이트 동일성을 잃을 수 있습니다.

## 해결과 검증

- 고정 커밋의 `raw.githubusercontent.com` 원본 바이트를 사용합니다.
- Git tree 메타데이터의 크기·blob SHA-1과 대조한 뒤 SHA-256을 매니페스트에 기록합니다.
- 선언의 줄 번호를 확인할 때만 매니페스트의 `text_encoding`을 적용합니다.
- 소스 46개 모두의 원본 blob 해시를 대조했고, 별도 폴더 재확보도 통과했습니다.

재현 명령:

```sh
python3 tools/reference_assets.py fetch --manifest assets/manifests/reference_sources.json
python3 tools/reference_assets.py verify --manifest assets/manifests/reference_sources.json
```
