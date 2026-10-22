# Linux·Web 전체 게임 다운로드

담당 [#122](https://github.com/edwardkim/ohmylemmings/issues/122), 상위 [#115](https://github.com/edwardkim/ohmylemmings/issues/115).

사용자가 다음 작업1번을 승인했다. Linux/Web에도 Windows/Mac과 동일한 원본28개(그래픽·120레벨·음원)를 포함하고 기본 Fun01을 실행한다. 기존 원본 없는 개발 모드는 명시적으로 유지한다. 원본/캡처는 ignored assets/local과 artifact에만 둔다.

패키징 TDD로 누락/변조/심볼릭 링크·허용 목록·결정성을 확인한다. Linux는 실행 파일 옆 data를 다른 작업 디렉터리에서도 사용한다. Web은 HTTP 상대 경로로 동봉 파일을 자동 로딩하며 손상/누락을 오류 처리한다. 음향은 사용자 버튼 활성화를 유지한다. 실제 새 압축 해제의 기본 원본 화면·120개 선택·비영 PCM·Fun01 성공/재시작/다음 레벨을 확인한다. 기존 합성 회귀도 유지한다. #113의 저사양 장비60fps 작업은 재개하지 않는다.

Apple 공인 Developer ID가 없다는 사용자 답변을 기록한다. Homebrew 자체 tap의 소스 빌드 Formula는 가능한 후속 경로다. 기존 사전 빌드 앱 Cask는 격리/Gatekeeper를 우회하지 않으며 공식 Cask는 기본 보안 설정에서 통과해야 한다. tap 구현·공식 제출은 이번 범위가 아니다.

공식 근거: [Formula](https://docs.brew.sh/Formula-Cookbook), [Cask 신뢰 모델](https://docs.brew.sh/Homebrew-Security-and-Supply-Chain#casks-have-a-different-trust-model).

구현/로컬 기능 검사 완료: 패키징20·Desktop41·Clippy·Python224/감사 PASS. Linux/Web 실제 새 압축 해제 기본 Fun01/120개·동일 첫 프레임1539ca2a…·Fun01 성공→Restart/Continue와 새 시도·비영 원본 PCM PASS. Web 필수 목록/MAIN 누락 오류·refresh/하위 경로·소스/라이선스 전달 PASS. 합성 WASM/native66회·11,960상태/6,877화면·Canvas/입력/음향 명령 PASS. 저사양 성능 반복 없음. 최종 CI·다운로드 검증·self-review/실제 병합을 진행한다.

첫 PR CI의 Linux 패키지 검사는 정렬하지 않은 glob에서 첫 프레임을 골라 초기 대신2틱 프레임과 비교했다. 원본 입력/픽셀 엔진 오류가 아니며 `frame-000000`을 명시해 검사/스크린샷 선택을 고정했다. 첫 실패 run39755457756을 보존하고 수정 head의 CI를 재실행한다.
