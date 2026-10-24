# #202 사용자 로컬 MP3 BGM 진행 상태

이슈: https://github.com/edwardkim/ohmylemmings/issues/202

기준 main `3ee06c63bd33fab9456c3383c7422b7a97231cf4`. [계획](../plans/issue-202_local_mp3.md), [선행 검토](../reports/issue-200_custom_bgm_review.md).

이슈 등록/번호 검증·현행 구조 확인 완료. 공통 합성의 채널 역할별 gain, Web 로컬 파일 재생/UX, Native bounded 디코딩/스테레오 출력 순으로 진행한다. 원본 드라이버의 음악/효과음 채널 할당을 유지한 채 출력만 조절하며 기본 gain100/100의 기존 C/PCM 동일성을 보존한다. 사용자19222/8080·기존 미리보기와 사용자 파일은 유지한다.
