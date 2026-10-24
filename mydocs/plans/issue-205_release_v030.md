# #205 v0.3.0 정식 릴리즈·GitHub Pages

이슈: https://github.com/edwardkim/ohmylemmings/issues/205

상태: 정식/latest 릴리즈·GitHub Pages 공개 검증 완료. [최종 결과](../reports/issue-205_release_v030_completion.md). 아래는 수행한 계획이다.

메인테이너가0.3.0으로 정식 릴리즈와 ghpage 게시를 지시했다. 기준 main `dca8b2dc2b2c166e4e0cf8600d165f83402d587d`, 기능 [#202](../reports/issue-202_local_mp3.md), 정책 [#124](issue-124_release_policy.md)를 따른다.

1. workspace/lock의9개 crate를0.3.0으로 맞추고 불변 버전 노트·실행 안내를 작성한다. #202 기능 코드와 원본/HD 자산은 유지한다.
2. self-review·같은 source tree의 CI7개 및 Windows/macOS/Linux/Web 전체 게임·새 설치/MP3 probe·원본120레벨 로딩·체크섬·대응 소스 검증을 완료한다.
3. main 병합 후 prepare-only Release workflow로4개 패키지와 소스/manifest를 검증한다. 성공 후 annotated v0.3.0 태그를 만들고 push한다.
4. draft 업로드/재다운로드 검증·정식/latest 공개·12개 익명 다운로드 검증, 검증된 동일 Web ZIP의 Pages 승격과 공개 HTTPS의 게임/로컬 MP3/진행률을 확인한다.
5. v0.1.0/v0.2.0 태그·asset·노트를 보존했는지 대조하고 완료 문서/로컬 환경을 동기화한다. #205는 완료 시 종료한다.

MP3는 사용자 로컬 파일만 선택하며 음원/파생 PCM을 공개하지 않는다. #202의 사람 청취 응답과 #113 NOT READY는 별도이며 배포 지시를 청취/성능 합격으로 확대하지 않는다. 사용자19222/8080·기존 미리보기와 Music 원본을 유지한다.
