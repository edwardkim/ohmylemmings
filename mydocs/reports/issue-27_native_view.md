# G06-01 네이티브 화면·입력 연결 결과

작업 이슈: [GitHub #27](https://github.com/edwardkim/ohmylemmings/issues/27). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

- [실행 안내](../../apps/desktop/README.md), [계약·재현·제한](../tech_insights/issue-27_native_view.md), [진행 기록](../working/issue-27_native_view.md), [작업일지](../orders/20261019.md#issue-27).
- [원본 X11 실행·캡처 증거](../../assets/manifests/native_view.json).

## 변경 내용

apps/desktop에 SDL2 네이티브 어댑터를 추가했습니다. 원본 Fun01의 지형·객체 이미지·팔레트·MAIN 스프라이트를 조합하고 17Hz 생성·낙하·걷기를 표시합니다. 카메라·Pause/Resume·RR·종료·입력 기록/재생을 연결했습니다. 320×200 화면을 정수 배율로 표시하고 SDL readback과 논리 프레임을 함께 저장합니다.

기존 코어/리소스 no_std 경계와 앱의 unsafe 금지를 유지했습니다. SDL2와 7개 고정 레지스트리 의존은 앱에만 연결됩니다. CI는 SDL2 SDK/잠금 의존 준비 후 오프라인 검사하고 WASM은 두 라이브러리를 빌드합니다.

## 검증 결과

| 검사 | 결과 |
|---|---|
| TDD | 세션 새 5개 Red, 화면 새 4개 Red/기존 7개 통과 → 데스크톱 15개 Green |
| 경계/오류 | 카메라 양끝·발 좌표/투명/잘림·제거 비표시·손상/누락 자원·입력 순서/반복/제한·CLI·렌더 순수성 |
| F00 실시간 | SDL x11/software 2.30.0, 340호출·341스냅샷·20.180초·870렌더 |
| 다른 렌더 주기 | F00 57렌더에서도 341개 상태가 동일한 기록 Lemmix 실행 서명과 일치 |
| 원본 기준 재생 | F02-98/99·F03 glitch on 포함 네 시나리오/다섯 실행의 모든 상태 서명 일치 |
| 실제 X11 키 | Pause/Resume·RR 3변경·카메라·Escape, 기록된 5개 게임 입력의 정수 경계로 G05에서 81개 상태 재현 |
| 전체 원본 실행 | 6실행·1786스냅샷, SDL 캡처 63장 전체 픽셀의 3배 nearest readback 대조 성공 |
| F00 상태 SHA-256 | 3e56870904094448fdeab59715ce9c7ad96ec25deb9ff34017cd24058d74d4fe |

원본 50개 크기/해시를 검증했습니다. 실행 바이너리·앱 소스·Cargo.lock·참조 manifest·매 실행의 상태/입력·63개 캡처(RGB/PPM/PNG) 해시를 증거 manifest에 보존했습니다. 로컬 화면은 `assets/local/issue-27-native-proof/F00-realtime/frame-000170.png`입니다. 원본 파생 이미지는 추적하지 않습니다.

make check의 Rust 단위·예제 297개·문서 27개·Python 18개·기존 참조/독립 모델·실제 SDL 합성 4실행/1124상태/40캡처·포맷·Clippy·네이티브/WASM을 통과했습니다. 문서의 release 빌드·원본 검증·격리 X11 재현 sh 블록을 그대로 실행해 성공했습니다. 재현 실행의 X11 키 전달 경계는 달랐지만 저장된 입력의 상태 재생과 63개 캡처 검증은 같습니다. 문서 98개·로컬 링크 697개·앵커 114개·이슈 문서명 84개·#27 일지와 앱/라이브러리 의존 경계를 확인했습니다. 구현 게시·실제 CI·원격 tree를 확인했습니다.

## 완료 범위와 다음 목표

G06-01의 네이티브 화면·입력 연결과 실행 증거를 확보했습니다. 게시/CI 확인으로 이슈의 완료 기준을 충족했습니다. **G06 전체는 진행 중**이며 다음은 별도 이슈의 **G06-02 DOS 실행·동일 장면 대조**입니다. 이번 결과는 전체 DOS 게임·직업·객체 트리거/구출·승패·음향·Windows 호환성을 인증하지 않습니다.

## 게시 근거

구현 `766cca58a9657e354fa2f30b66754dea6f3014a7`를 main에 게시했습니다. [push CI](https://github.com/edwardkim/ohmylemmings/actions/runs/39152276025)의 SHA·이벤트·실제 로그·세 작업 결론을 확인했습니다.

| 작업 | 실제 결과 |
|---|---|
| [Rust native](https://github.com/edwardkim/ohmylemmings/actions/runs/39152276025/job/117748183487) | Rust 1.93.1·SDL2 SDK/잠금 의존 준비, 단위/예제 297개·문서 27개·기존 참조/독립 회귀·실제 SDL 4재생/1124상태/40픽셀 캡처·두 상태 서명 success |
| [Python tooling](https://github.com/edwardkim/ohmylemmings/actions/runs/39152276025/job/117748183468) | Python 3.12.15, unittest 18개·기존 독립 모델의 두 서명 success |
| [WASM libraries](https://github.com/edwardkim/ohmylemmings/actions/runs/39152276025/job/117748183250) | 잠금 의존 준비 후 기존 두 no_std 라이브러리 오프라인 컴파일 success |

원본·기존 target/Cargo package cache 없는 입력에서 시작해 SDK/fetch 후 검사를 통과했습니다. CI는 SDL dummy/software, 원본 X11·키 입력은 로컬 증거입니다. 원격 tree의 파일 177개 모드/blob이 로컬과 일치했고 작업 트리는 깨끗합니다. 원본·SDK·실행 파일·원본 파생 이미지/전체 추적은 추적하지 않습니다.

최종 문서 게시·CI·상위 #1의 G06 진행 갱신·#27 종료는 GitHub 이슈 기록에서 추적합니다.
