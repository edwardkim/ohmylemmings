# G10-02 ADLIB 명령과 레지스터 스트림

이슈: [#74](https://github.com/edwardkim/ohmylemmings/issues/74) · 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1)

main b6d0d3c72a295347f20373095a2979e63193d3a5와 이슈 번호/URL 확인 후 issue-74-adlib-stream에서 착수했습니다. 무관한 미추적 디렉터리와 HTTP8080/방화벽·CDP19222 loopback은 유지합니다.

- no_std/무할당9채널 명령·음악 분주기/효과음4슬롯·순서 있는 레지스터 스트림, 반복/종료/stop/reset·재시작, 최대4096명령과 오류 시 상태/출력 보존 구현.
- Rust15/Python7 TDD와 원본 없는 합성10시나리오877호출·1621쓰기 전체 상태/순서 대조 통과.
- 원본 x86-16 드라이버 직접 실행과21곡의 채널별2회 반복·18효과음 종료·혼합/재시작2시나리오, 합계227615호출/215681쓰기 일치. 원본50파일 전후 해시 동일.
- 초기 쉼표·주파수 표 교차/AL 산술·B0 명령과 DX=0 ABI, TS 차이 및 원본 호출 단위/미확정 PIT/PCM 시간을 [계약](../tech_insights/issue-74_adlib_stream.md)에 고정.
- 전체 make check(Rust550/문서29/Python128) 종료0·self-review 미해결0·출처/해시 고정. 한글 PR·정확한 push/PR/main CI와 전체 로그·같은 병합 트리·이슈/상위 마감 진행 중.

이번 결과는 드라이버 호출 단위 명령/상태/레지스터 검증입니다. G10 전체는 진행 중이며 다음은 G10-03 OPL/PCM·샘플/청크 결정론과 원본 시간 근거입니다.
