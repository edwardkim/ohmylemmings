# G06-02 DOS 실행·동일 장면 대조 결과

이슈: [#28](https://github.com/edwardkim/ohmylemmings/issues/28), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

[계약·명령·결과](../tech_insights/issue-28_dos_comparison.md) · [진행](../working/issue-28_dos_comparison.md) · [일지](../orders/20261019.md#issue-28) · [증거 manifest](../../assets/manifests/dos_observation.json).

## 결과

원본 DOS 프로그램을 고정 DOSBox에서 세 번 실행하여 Fun01의 생성·낙하·걷기를 실제 영상으로 확보했습니다. 매 실행의 원본 50개 해시는 변하지 않았습니다. 원본 키 디스크는 없으며 별도 생성 RUSSELL.DAT의 카운터 seed가 필요했습니다. [#29](../tech_insights/issue-29_dos_runtime.md)에 초기 실패 화면·변경 바이트/해시·전제를 기록했습니다.

| 검증 | 결과 |
|---|---|
| 반복 실행 | 새 프로세스/복사본 3개, 게임 영상 1811/1806/1811 = 5428프레임 + 메뉴 219프레임 |
| 상부 320×100 | 0/35/54/60/100/170/340의 일곱 네이티브 장면이 세 번 모두 픽셀 일치 |
| 전체 320×160 | 초기 장면만 일치, 이후 객체 장식 19~26픽셀 차이; UNVERIFIED |
| 관측 속도 | 대응 장면 1호출당 영상 5프레임; 고정 환경 약 14.017 장면/초와 네이티브 17Hz 차이 |
| 네이티브 재검증 | F00 기존 상태 서명과 341상태 일치, 실제 SDL 캡처 10개 전체 확대 픽셀 일치 |
| 합성 TDD | 캡처 도구 미존재 Red → 7 Green; 범위 분리 1 Red → 총 8 Green |
| 전체 회귀 | Rust 단위/예제 297·문서 27, Python 26, 기존 참조/모델·SDL 4실행/1124상태/40캡처·WASM·포맷/Clippy 성공 |

전체 영역과 상부 범위를 분리해 불일치를 숨기지 않습니다. RGB565 정밀도를 맞추기 전 정확한 2배 복제를 확인하고 좌표·허용 오차·커서 마스크 없이 비교했습니다. 영상 인덱스로 DOS 내부 틱/개체 상태를 단정하지 않습니다.

로컬 비교 그림은 `assets/local/issue-28-dos-comparison-verified/run-1/pair-000170.png`입니다. 왼쪽 DOS/오른쪽 네이티브이며 출구 장식 차이를 포함합니다. 초기 실패/메뉴/레벨 안내/전체 영상·입력·원본 파생 그림은 ignored assets/local/에만 있습니다.

## 완료 범위와 다음 작업

G06-02의 실제 DOS 실행·반복 관측·동일 장면 대조를 완료했습니다. 전체 화면 호환성 통과는 아니며 **G06 전체는 진행 중**입니다. 다음은 [#30 객체 위상·관측 속도 보정](https://github.com/edwardkim/ohmylemmings/issues/30)입니다. DOS 내부 상태·키 디스크/무설정 기동·직업·트리거/구출/승패·음향·Windows는 미검증입니다.

## 게시 검증

구현 `4aa4199fe3d13690ec3f8dec61b7f61045b3d76f`를 main에 게시했습니다. [실제 push CI](https://github.com/edwardkim/ohmylemmings/actions/runs/39155065616)의 이벤트·정확한 SHA·세 작업/초기 clean-input·SDK/잠금 의존 준비·실제 테스트/모델/SDL 서명을 확인했습니다.

- Rust native: 단위/예제 297개·문서 27개·기존 참조/모델·SDL 4실행/1124상태/40캡처 success.
- Python tooling: Python 3.12.15, unittest 26개·독립 모델 두 서명 success. DOSBox/PyAV/원본 없이 합성 검사합니다.
- WASM libraries: 기존 두 no_std 라이브러리의 잠금 의존 준비·오프라인 컴파일 success.

원격 187개 파일 모드/blob이 로컬과 일치하고 작업 트리는 깨끗했습니다. 원본·도구/실행 파일·원본 파생 영상/화면/전체 추적은 추적하지 않습니다. 문서 103개·로컬 링크 736개·앵커 118개·이슈 문서명 89개도 확인했습니다. 최종 문서/CI·상위 #1 갱신·#28/#29 종료·#30 미완료 상태는 GitHub 기록에서 추적합니다.
