# Windows 전체 게임 패키지

담당 [#118](https://github.com/edwardkim/ohmylemmings/issues/118), 상위 [#115](https://github.com/edwardkim/ohmylemmings/issues/115).

사용자는 다운로드 ZIP의 exe가 잠깐 열리고 종료된다고 보고했다. 명령으로 합성 연습 모드는 실행됐으나 도형만 보인다고 확인한 뒤, 원본 그래픽·전체 레벨·음원을 포함해 별도 자산 준비 없이 온전한 게임을 실행하도록 지시했다. 최신 사용자 지시에 따라 Windows 패키지의 기존 원본 제외 계약을 변경한다.

- Windows 기본 자산 경로를 exe 옆 `data`로 연결하고 인수 없이 원본 Fun01을 시작한다. 원본 실행 오류는 로그/메시지로 남긴다.
- SDL2 정적 CRT 검토의 검증된 입력/빌드 방식을 정식 Windows ZIP과 CI로 연결한다. 고정 자산 목록의 그래픽/레벨/음향 파일을 동봉하고 전체 해시를 검사한다.
- 원본/파생 바이너리는 ignored assets/local과 artifact에만 둔다. Git에는 코드·출처/해시·숫자 증거를 기록한다. source archive는 기존 공개 빌드 소스만 포함하고 ZIP의 data를 별도 목록으로 검증한다.
- 압축 해제한 새 한글/공백 경로, 다른 CWD와 no-argument 원본 시작, 120레벨/특수 장면 선택·원본 그래픽/오디오 경로·재시작/전환을 실제 Windows에서 확인한다. 현재 호스트의60fps 추가 조정은 하지 않는다.

패키징5개와 기본경로 단위 검사의 red→green을 확인했다. 실제 Windows 검사는 깨끗한 Windows2022/MSVC 빌드와 압축 해제된 새 경로만 사용하도록 CI에 연결했다. 게시/PR/병합·정확한 artifact 검증 결과는 #118에 기록하며 보고서에 검사 범위와 다운로드 절차를 연결한다.
