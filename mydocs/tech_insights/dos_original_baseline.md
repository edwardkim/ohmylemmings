# DOS 원작 기준판과 확보 리소스

관련 작업: [G00-01 / GitHub #2](https://github.com/edwardkim/ohmylemmings/issues/2). 파일별 바이트 수와 SHA-256의 기준은 [매니페스트](../../assets/manifests/dos_original.json)입니다.

## 기준 프로필과 확보 상태

| 항목 | 값 |
|---|---|
| 기준 ID | `dos-original-vga` |
| 대상 | DOS 원작의 VGA 그래픽·AdLib 사운드 리소스 |
| 로컬 위치 | `assets/local/dos-original/` |
| 확보 | 50개 파일, 합계 831,245바이트 |
| 엔진 리소스 | 27개: 레벨 10개, GROUND 5개, VGAGR 5개, VGASPEC 4개, MAIN·ODDTABLE·ADLIB |
| 참고용 파일 | 23개: 실행 파일·시작 배치·문서·CGA/Tandy 데이터 등 |
| 세부 DOS 리비전 | 미확정; 특정 소매판 버전 번호를 추정하지 않고 매니페스트로 입력 고정 |

파일 확보와 바이트 대조를 완료했습니다. 원작 실행 파일 구동, DAT 압축 해제, 레벨별 물리 동작 비교는 후속 작업의 검증 대상입니다.

## 출처와 고정 버전

| 용도 | 고정 출처 |
|---|---|
| 원작 DAT 26개 | [Lemmix `orig.zip`](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Data/Styles/Orig/orig.zip), 커밋 `40e9bc34451f0e9127fd53b290a3877e700d143d` |
| ADLIB 및 참고용 파일 보충·상호 대조 | [DOS 데이터 디렉터리](https://github.com/VorticonCmdr/lemmings/tree/1f9258d198e85da4815d90c9cd0b55b70b4f5098/gamedata/lemmings), 커밋 `1f9258d198e85da4815d90c9cd0b55b70b4f5098` |
| 원작 파일 이름·크기 목록 | [Lemmings.ts 원작 데이터 안내](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/public/data/lemmings/README.md), 커밋 `10c7c8b9167258503d02589cf048e2d83a43dd0a` |

Lemmix ZIP과 보충 출처가 공유하는 DAT 26개의 내용이 모두 바이트 단위로 일치합니다. 확보한 엔진 리소스 27개는 Lemmings.ts의 이름·크기 목록과 모두 일치합니다. 크기 일치와 바이트 일치의 검증 범위는 별도로 기록합니다.

- `orig.zip`: 336,839바이트, SHA-256 `bf1f2dbd11cd20df9f748047f8c4a32e7033d3ac7bb4b77058c3ecf27c0a9d6d`
- `ADLIB.DAT`: 12,988바이트, SHA-256 `43eceee3e88fde48c679c635ee208d2af9ade1ed789dc291d647e7d314ffdcfe`

리소스의 파일 역할과 확보 원본은 매니페스트의 `role`, `source`, `path` 필드에 기록합니다. 영문 파일명은 로컬에서 대문자로 정규화합니다.

## 데모 비교

[DOS Games Archive의 영어 데모](https://www.dosgamesarchive.com/file/lemmings/lemming)도 확보해 비교했습니다. 배포 페이지는 이 파일을 플레이 가능한 DOS 데모로 표시합니다.

| 파일 | 원작 프로필 | 확보한 영어 데모 |
|---|---:|---:|
| `MAIN.DAT` | 56,472바이트 | 65,670바이트 |
| `ADLIB.DAT` | 12,988바이트 | 12,989바이트 |
| `LEVEL000.DAT` | 3,722바이트 | 3,417바이트 |

데모 ZIP의 SHA-256은 `2691c56bc853e01eb6de4cc765675d62b86490c5f603c78d3dbe0d3532813a41`입니다. 데모와 원작 파일은 독립된 입력이며 서로 섞지 않습니다. 데모는 초기 조사용 로컬 캐시에 보관하고, 기준 프로필의 검증은 원작 매니페스트로 수행합니다.

## 확인한 원작 레벨 체계

[Lemmix의 원작 레벨 시스템](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Styles.Dos.pas)과 [Lemmings.ts 설정](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/public/data/config.json)은 Fun·Tricky·Taxing·Mayhem 각 30개, 총 120개 진입점을 정의합니다. 저장된 레벨 데이터와 ODDTABLE의 재사용·변경 정보를 함께 해석해야 합니다.

첫 비교 대상은 Fun 1입니다. 두 레퍼런스는 `LEVEL009.DAT`의 두 번째 구간(0부터 세면 1)에 연결합니다. 이 연결의 실제 DAT 파싱과 화면 검증은 G02~G03에서 수행합니다.

## 후속 분석

세부 리비전은 확보한 실행 파일의 시작 화면과 동봉 문서를 DOS 실행 환경에서 대조해 확인합니다. 명확한 표기를 찾지 못하면 `release_revision`을 미확정으로 유지하고 파일 해시를 식별 기준으로 사용합니다. 실제 원작 실행과 비교 시나리오는 G00-03에서 구체화합니다.

G00-02에서 코드의 파일·함수와 틱·물리·파싱 역할을 고정하고, G00-03에서 전체 직업·규칙·사운드 목록과 비교 시나리오를 정의합니다. G00 전체는 이 두 작업까지 완료한 뒤 완료 처리합니다.
