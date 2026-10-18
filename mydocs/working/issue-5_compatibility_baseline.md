# G00-03 진행 기록

작업 이슈: [#5](https://github.com/edwardkim/ohmylemmings/issues/5), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 현재 상태

이슈 등록과 번호·URL·열린 상태를 확인한 뒤 분석을 시작했습니다. 지원 목록·비교 계약 작성과 로컬 검증을 마쳤으며 게시 확인을 진행합니다.

- [작업일지](../orders/20261018.md#issue-5)
- [호환성 기준](../tech_insights/issue-5_compatibility_baseline.md)
- [전체 레벨 목록](../tech_insights/issue-5_level_inventory.md)
- [기계 판독용 목록](../../assets/manifests/dos_compatibility.json)
- [결과 보고서](../reports/issue-5_compatibility_baseline.md)

## 확인한 내용

- 원작 50개와 고정 소스 46개의 크기·SHA-256·선언 위치 검증을 통과했습니다.
- 120개 진입점의 DAT 구간·변형 여부가 두 레퍼런스에서 일치했습니다. 기본 레코드 80개와 ODD 변형 40개입니다.
- 임시 Python 분석과 Node의 고정 TypeScript 디코더 실행으로 해제 구간 81개 전체의 크기·SHA-256이 일치했습니다.
- ADLIB 이미지에서 음악 헤더 21개·효과음 주소 18개를 확인했습니다. TS 설정의 20곡 제한과 ODD 인덱스 경로 차이를 기준에 기록했습니다.
- 직업 8종, 원작 메커니즘·버그, 입력·상태 비교 형식과 COMP-01~18을 후속 목표에 연결했습니다.
- 문서 30개의 로컬 링크 124개·앵커 20개·고정 소스 줄 링크 87개와 이슈 문서명 18개를 검사했습니다. 레벨 표와 기계 판독용 목록도 일치합니다.
- 실제 DOS 실행과 Rust 동작 비교는 아직 수행하지 않았습니다. 검증 책임과 통과 증거를 G05~G11에 지정했습니다.

## 다음 행동

1. 게시 후 원격 파일과 커밋을 확인하고 이슈 #5와 상위 #1의 G00 상태를 갱신합니다.
2. 다음 G01-01은 별도 이슈 등록·확인 후 Cargo 개발 골격을 도입합니다.
