# G07-01 Digger 검증 결과

이슈: [#31](https://github.com/edwardkim/ohmylemmings/issues/31), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

[계약/재현](../tech_insights/issue-31_digger.md) · [진행](../working/issue-31_digger.md) · [일지](../orders/20261019.md#issue-31) · [증거](../../assets/manifests/digger.json).

## 변경과 검증

Walking 개체의 Digger 할당·수량·초기 세 줄/8틱 굴착·공백/강철 전환·삭제 지형을 보는 후속 개체·네이티브 클릭/표시/정수 재생을 구현했습니다. 코어/리소스 no_std, 의존 방향·앱 unsafe 금지와 기존 불변 지형 API를 유지합니다.

| 검증 | 결과 |
|---|---|
| TDD/경계 | 새 API Red→초기 8 Green, 최종 Digger 12개·화면 밖 클릭 거부 Red→Green. 모든 9비트 지지 패턴×16프레임×강철 2종의 16384경우 검사 |
| 실제 Pascal 기준 | 원문 18개 함수, 합성 6사례·12재생·4092상태·모든 삭제 인덱스 일치 |
| 원본 Fun01 기준 | 3렌더/시간 방식의 각 341상태·전체 삭제 목록이 실제 기준 함수와 일치 |
| 실제 네이티브 | X11 4실행·1204상태·160개 픽셀 정확 SDL readback. 실제 마우스 경계61 입력의 181상태도 참조 재생과 일치 |
| DOS 관측 | 첫 Walking 뒤 수량10→9와 Digging/frame0/발 y+1. EXE/50개·메모리 변경 없음, 별도 #29 seed 조건 |
| DOS 전체 장면 | 원본 두 실행의100/170/340 전체320×160, 세 장면×2회 차이0픽셀. 영상4435프레임·읽기 전용1170샘플 |
| 전체 검사 | Rust 단위/통합/예제320·문서27·Python37·기존 모델/참조/SDL·WASM·포맷/Clippy 성공 |

원본 F01의 전체 상태/삭제 목록 SHA-256은 `b1e8267b51a279558e7b6b3e3892a43cc3592ec37f82161ac60d12b14ea45b14`입니다. 수정 빌드의 원본 F00 341상태/SDL10캡처·기존10전체 장면과 초기 G05 F00·두 독립 모델·기록된 기준·SDL 네 실행의 서명은 같습니다. 합성 Digger CI는 추가12재생/4092상태/534픽셀 캡처입니다. 레벨1 최초 Walking은 native snapshot60이며 할당 입력60의 결과는 snapshot61에 나타납니다. 실제 마우스의61은 별도로 기록한 실제 경계입니다.

비교 그림은 assets/local/issue-31-dos-comparison-final/run-1/pair-000170.png에 보존합니다. 실패한 준비 실험 #31-dos-1~4와 초기 교정 실행5는 최종 두 실행6/7의 통과 횟수에 포함하지 않습니다.

## 범위와 다음 단계

G07-01의 Digger 한 직업을 검증했습니다. 다른 직업에서의 Digger 전환·secondary/중복 후보·전체 객체/트리거 맵·출구/구출/승패/완주·오디오·전체 DOS 내부 상태/입력 추적·Windows는 후속 범위입니다. F01의 전체 완주/COMP-18을 완료 처리하지 않습니다. G07 전체는 진행 중이며 다음 후보는 G07-02 Basher입니다.

## 게시 검증

구현/문서 게시 후 실제 CI의 대상 SHA·세 작업/실제 로그·clean-input·기존 및 Digger 회귀 서명을 확인합니다. 상위 #1과 #31 종료 근거는 GitHub 기록으로 추적합니다.
