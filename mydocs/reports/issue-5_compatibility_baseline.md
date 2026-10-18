# G00-03 지원 범위와 호환성 기준 정의 결과

작업 이슈: [#5](https://github.com/edwardkim/ohmylemmings/issues/5), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

- [작업일지](../orders/20261018.md#issue-5), [진행 기록](../working/issue-5_compatibility_baseline.md)
- [판정 계약](../tech_insights/issue-5_compatibility_baseline.md), [전체 레벨 목록](../tech_insights/issue-5_level_inventory.md)
- [기계 판독용 지원 목록과 기대 해시](../../assets/manifests/dos_compatibility.json)

## 결과

원본 VGA·AdLib 프로필의 120개 진입점·직업 8종·게임 규칙·음악 21개·효과음 18개의 범위를 정의했습니다. F00~F03의 첫 비교 사례, `dos-compat-trace-v1` 기록 형식과 COMP-01~18의 검증 책임을 G02~G11에 연결했습니다.

Fun 1은 LEVEL009.DAT 구간 1, 총 10개체·필요 구출 1·Digger 10·RR 50·5분으로 확인했습니다. ODD 변형 인덱스, TS 설정의 음악 개수, 17Hz와 60ms 차이, 원작 버그 옵션을 그대로 복사하지 않도록 채택 기준과 후속 DOS 검증 조건을 명시했습니다.

## 실행한 검증

| 명령·검사 | 결과 |
|---|---|
| `python3 tools/reference_assets.py verify` | 원작 50개 크기·SHA-256 일치 |
| `python3 tools/reference_assets.py verify --manifest assets/manifests/reference_sources.json` | 소스 46개와 선언 위치 검증 통과 |
| 임시 Python 분석: 컨테이너·XOR·해제 크기·두 소스의 연결표 대조 | 기본 레코드 80×2,048바이트, 연결 120개 일치, 변형 40개 |
| Node로 고정 TypeScript 디코더 실행 후 Python 결과 대조 | 레벨 80개와 ADLIB 1개의 크기·SHA-256 모두 일치 |
| 오디오 구조 범위 검사 | 음악 21개 헤더·악기·채널 주소와 효과음 18개 주소가 이미지 범위 안에 있음 |
| 지원 목록·기계 판독용 데이터·문서 표 대조 | 120개 레벨의 초기 조건·구간·변형·직업 수, 81개 해시, COMP-01~18 일치 |
| 문서명·이슈·로컬 링크·앵커·고정 소스 링크 검사 | 문서 30개, 로컬 링크 124개, 앵커 20개, 고정 소스 줄 링크 87개, 이슈 문서명 18개 통과 |
| `git diff --check`, `git diff --cached --check` | 통과 |
| `git ls-tree -r HEAD`와 GitHub Git tree API 대조 | 커밋 `22f6579`의 파일 36개 blob 해시 일치 |
| 로컬 HEAD와 GitHub main 참조 비교 | 일치 |
| 상위 이슈 #1 조회 | G00 및 #2·#3·#5 체크, G01~G12 미완료, 상위 이슈 열림 |

분석 환경은 Python 3.12.3과 Node v24.11.0입니다. TypeScript 실행은 고정된 BinaryReader·BitReader·BitWriter·UnpackFilePart·FileContainer 5개 클래스를 임시 파일에 모으고 import와 로그 출력만 대체했습니다. 전체 게임이나 레벨 로더를 구동한 검증은 아닙니다.

커밋 `22f6579`을 `main`에 게시하고 원격 상태를 확인했습니다. 원본 바이너리는 계속 Git 추적에서 제외됩니다.

## 검증 한계와 후속 작업

G00는 비교 입력과 지원·판정 기준을 정하는 단계입니다. 실제 DOS 화면·입력 재생, Lemmix 계측 실행, 곡/효과음의 이벤트 대응과 Rust 호환성 판정은 아직 수행하지 않았습니다. 구조만 확인한 데이터를 실제 재생·완주 성공으로 계산하지 않습니다.

G05는 구조화 추적, G06/G08은 DOS 실행 대조, G10은 음악·효과음 매핑과 속도, G11은 120개 레벨의 실행 판정을 담당합니다. 세부 리비전 이름은 파일 해시로 입력을 고정해 식별 문제를 해결합니다. 다음 개발 작업은 G01-01이며 별도 이슈에서 진행합니다.
