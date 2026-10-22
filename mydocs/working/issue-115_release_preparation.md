# G12 배포 준비

담당 [#115](https://github.com/edwardkim/ohmylemmings/issues/115), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1), 성능 후속 [#113](issue-113_web_dispatch.md).

2026-10-22 사용자는 현재 저사양 컴퓨터에서의 60fps 조정을 중단하고 더 좋은 환경에서 성능을 재검증하며 다음 단계로 진행하도록 승인했다. 이슈 OPEN·번호/URL을 확인한 뒤 착수했다. G11 성능 합격이나 최종 릴리스 품질 인증으로 바꾸지 않는다. G12 준비의 선행 조건 예외를 사용자 지시로 기록한다.

- 현재 버전은 workspace `0.1.0`, Linux SDL2와 Web 빌드/합성 실행 검사가 있다. Windows 배포 빌드·실행 증거와 릴리스 패키징 도구는 없다.
- 배포 파일은 원본 DOS 자산을 제외한다. 사용자 소유 자산을 가져오는 경로와 원본 없는 연습 장면을 제공한다.
- SDL2 런타임 요구, 지원 OS/아키텍처, 웹 HTTP 배포, LGPL 대응 소스·재링크, 체크섬과 설치/재시작/오디오 검증을 배포 계약에 명시한다.
- 먼저 기존 기능 변경을 검증·게시하고 [배포 계약](../plans/issue-115_release_preparation.md)에 따라 깨끗한 소스에서 재현 가능한 산출물을 준비한다. Windows 미검증과 고성능 호스트 성능 검증은 별도 잔여 항목이다.

- Linux TAR.GZ/Web ZIP 패키징을 TDD로 구현했다. 7개 오프라인 검사와 실제 압축 해제·SDL 결과/재시작/오디오·Web WASM/DOM/오디오/HTTP 소스 전달 pilot를 통과했다. 런타임 허용 목록·현재 Web 소스 일치·링크/누락/형식·전체 source inventory/해시·변조·덮어쓰기 거부와 입력별 결정성을 검사한다.
- 기존 Ubuntu24.04 Native/Web CI에 패키지/설치 검사와 14일 artifact 보관을 추가한다. upload-artifact v7.0.2 tag/commit/action.yml의 Node24 및 input을 공식 API로 확인하고 SHA를 고정했다. CI의 깨끗한 빌드를 로컬 pilot와 구분한다.
- #113 소스/측정과 NOT READY는 유지한다. #115 source index 변경은 빌드/패키징 도구에 한정하며 과거120레벨/PCM/성능을 새로 실행한 것으로 세지 않는다. Windows, 전체 프로젝트 라이선스 결정, 더 좋은 호스트 성능과 정식 릴리스가 남아 있어 #115 OPEN을 유지한다.

- 최신 사용자 지시로 [#118](issue-118_windows_full_game.md)을 분리했다. Windows는 원본 그래픽·전체120개·음원을 포함하고 더블클릭으로 시작하는 게임 ZIP으로 변경한다. 이전 원본 제외/Windows 미구현 기술은 해당 변경 이전 기준이다.
