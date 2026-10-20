# G10-01 AdLib 자원과 통합 계약

이슈: [#72](https://github.com/edwardkim/ohmylemmings/issues/72) · 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1)

main07c727ef0c4ec145be5e9215948f7bdcd960f251 동기화와 이슈 번호/URL 확인 후 착수했습니다. 무관한 미추적 파일과 원격 미리보기/방화벽·CDP 설정은 유지합니다.

- no_std/무할당 ADLIB DAT/image·21곡18효과음/120레벨 대응 구현, Rust9/Python6 TDD 통과.
- 합성21곡48채널18효과음·원본21곡69채널18효과음의 전체 필드/첫 bar·레벨 대응 대조 통과. 원본50/TS4파일 해시·해제 SHA·기존 목록 일치.
- 고정 TS 실제 초기화/reader 메서드 실행(채널 생성은 메타데이터 stub). 공유 bar/주소 순서·TS20곡 설정 차이를 기록. OPL/PCM/DOS 재생을 검증한 것으로 계산하지 않음.
- [통합 계약](../tech_insights/issue-72_adlib_catalog.md)과 G10-01~06 순서를 고정. 전체 make check(Rust535/문서29/Python121) 종료0·self-review 미해결0·출처/해시를 고정했습니다. 한글 PR·정확한 게시 CI/병합/마감 진행 중.

현재 자원/초기 읽기 검증을 마친 후 다음 G10-02 명령 해석·레지스터 스트림으로 넘깁니다. G10 전체는 진행 중입니다.
