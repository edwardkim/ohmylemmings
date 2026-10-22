# Windows 다운로드 첫 실행 계약

담당 [#118](https://github.com/edwardkim/ohmylemmings/issues/118), 상위 [#115](https://github.com/edwardkim/ohmylemmings/issues/115).

ZIP을 모두 푼 뒤 `OhMyLemmings.exe`를 더블클릭하면 원본 Fun01의 그래픽·지형·패널과 음향이 동작해야 한다. 명령 입력·원본 파일 별도 다운로드·자산 경로 설정을 필수로 요구하지 않는다. `start-game.cmd`는 같은 원본 게임을 실행하며 오류 로그를 보여준다. 합성 장면은 명시한 개발 옵션으로만 선택한다.

Windows x64/MSVC 정적 CRT, 고정 SDL2.dll과 SDL/OPL 라이선스, exe 옆 data의 고정 원본 자산, 대응 소스/재링크 안내와 내외부 SHA256을 포함한다. 데이터와 코드는 각각 명시한 목록으로 검증하며 파일 누락/변조/추가 파일·링크를 거부한다. DOS의 설치/도움말/다른 비디오 드라이버처럼 현재 Rust 실행에 필요 없는 파일은 런타임 목록에 넣지 않는다.

검증은 합성 offline 패키징 오류/TDD, Windows의 no-argument 자산 경로와 잘못된 자산 오류/로그, 실제 원본 Fun01 시작/그래픽/음악, 4난이도120선택과 특수4장면, 기존 원본 Fun01 성공/재시작/실패/전환을 포함한다. 압축 해제된 패키지만 사용하고 repo 원본/현재 작업 폴더를 참조하지 않는다. 실행 가능한 완성 게임 데이터와 G11의 고성능 호스트 성능 합격을 구분한다.
