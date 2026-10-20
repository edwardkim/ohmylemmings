# G10-05 SDL/Web 오디오 출력

이슈: [#80](https://github.com/edwardkim/ohmylemmings/issues/80) · 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1)

SDL/Web 출력·사용자 활성화/음소거/볼륨·숨김/재개·재시작·원본/연습 음원과 라이선스/소스 배포를 연결했습니다. [계약](../tech_insights/issue-80_audio_output.md)·[보고](../reports/issue-80_audio_output.md)를 기준으로 최종 make check(Rust596/문서29/Python155) 종료0·자체 검토 미해결0·해시 고정. 한글 PR 및 정확한 게시/병합 CI·전체 로그·동일 트리와 #80/상위 #1 갱신을 진행합니다.

원본50/PCM/검증 영상·바이너리·개별 로그는 assets/local/issue-80-*에 보관합니다. 실제 PipeWire와 trusted CDP19222/LAN UI, 6조합249492프레임과원본Web8648상태4978Canvas를 확인했습니다. 초기 전체 실행은 소스 배포 검사를 추가하며 중단했고 최종 구성으로 재실행합니다. 재링크는 고정 입력 복사본을 사용하며 의도적 무음은 제품 검증과 구별합니다. 무관한 미추적 glyph, HTTP8080/방화벽·CDP loopback을 유지합니다. 다음은 G10-06이며 G10 전체는 진행 중입니다.
