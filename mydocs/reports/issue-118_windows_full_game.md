# Windows 전체 게임 다운로드 패키지

담당 [#118](https://github.com/edwardkim/ohmylemmings/issues/118), 상위 [#115](https://github.com/edwardkim/ohmylemmings/issues/115).

기존 시험 ZIP은 원본 자산을 포함하지 않았고 기본 실행은 개발용 경로를 찾다가 종료했다. 합성 옵션은 도형 장면을 실행했다. 사용자 지시에 따라 Windows ZIP에 필요한 원본 그래픽·120개 레벨·음향 데이터28개를 포함하고, exe 옆 `data`를 기본으로 읽어 원본 Fun01을 시작하도록 변경했다.

## 사용자 실행

`ohmylemmings-0.1.0-windows-x86_64.zip`을 내려받아 **모두 압축 해제**하고 `OhMyLemmings.exe`를 더블클릭한다. 별도 자산 확보·명령 입력·런타임 설치는 요구하지 않는다. exe, SDL2.dll, data 폴더를 함께 유지한다. 원본 기술 패널과 포인터 입력을 사용할 수 있으며 성공 결과의 Enter는 다음 레벨, R은 재시작이다. 난이도별 임의 선택은 동봉 README의 고급 옵션을 따른다.

시작 오류는 Windows 메시지와 `%LOCALAPPDATA%\OhMyLemmings\last-error.log`로 남는다. `start-game.cmd`는 같은 원본 게임을 실행하고 `last-run.log`를 기록하며 실패 시 안내 창을 유지한다. SDL2.dll 자체가 없을 때도 CMD에서 누락을 안내한다.

## 산출물과 검증

- Windows x64/MSVC, 고정 SDL2 2.32.10 DLL, 정적 MSVC CRT를 사용한다. SDL/OPL 라이선스·대응 source.tar.gz/재링크 안내, 원본28개 출처/해시 GAME-DATA.json과 내부 SHA256SUMS, ZIP 외부 sha256을 포함한다. ZIP 전체는39개 파일이며 DOS 설치/CGA/Tandy처럼 현재 실행에 쓰지 않는 파일은 제외한다.
- `tools/windows_package.py`는 입력별 크기/해시·PE x64 종류·허용 목록·일반 파일과 source inventory를 검사한다. 누락·손상·링크·변조·덮어쓰기를 거부한다. 합성 offline fixture5개와 exe 옆 기본경로 단위1개의 red→green을 확인했다.
- CI `Windows full game`은 Windows2022의 깨끗한 source/locked Cargo/고정 SDL·원본 입력을 사용한다. 단위검사와 release 빌드 후 ZIP을 한글/공백 경로에 풀어 다른 CWD에서 실행한다. 실제 무인수 표시 창, 원본 Fun01 전체 readback/픽셀, 네 난이도120개 선택(특수4 포함), Fun01 저장 해법 성공·재시작/다음 레벨과 독립 새 시도 동일성, 원본 ADLIB의 비영 SDL PCM, CMD 정상/누락 data·DLL 오류/로그를 검사한다. dumpbin으로 외부 VCRUNTIME/MSVCP/UCRT 의존이 없음을 검사한다.
- `development-windows-x86_64` artifact는 ZIP/sha256과 numeric JSON/로그·원본 Fun01 화면을14일 보존한다. Git에는 원본 바이너리를 넣지 않고 source archive도 공개 소스만 담는다. 정확한 검증 source SHA·run·artifact·이미지·병합 확인은 [#118](https://github.com/edwardkim/ohmylemmings/issues/118)에 기록한다.

이는 호스팅 Windows의 기능/패키징 검증이다. SDL dummy 오디오는 실제 ADLIB PCM 생성/큐 출력을 검사하며 사람의 스피커 청취와 구분한다. 기존120개 해법/240개 성공·실패·DOS 비교 증거와 #113 성능 이월을 새 실행/성능 합격으로 바꾸지 않는다. Linux/Web 개발 ZIP의 원본 제외 계약은 기존대로다.
