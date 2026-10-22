# #140 HD 표시/좌표·입력 진행

이슈: https://github.com/edwardkim/ohmylemmings/issues/140

메인테이너 다음 진행 승인, main5a543b3·OPEN/번호/URL 확인 뒤 별도 issue-140-hd-presentation worktree에서 착수했다. 관련 #137/#138 계획과 공통 LevelRun·Session·원본 renderer/패널을 확인했다.

먼저 경계/좌표·발 기준점과 읽기 전용 snapshot의 합성 테스트를 작성하고 실패를 확인한다. 구현 뒤 같은 입력의 상태·terrain·RGB·사운드 및 결과/재시작을 대조한다. 정확한 검사와 CI/병합은 이 기록과 결과 보고서에 남긴다.

새 API 부재로 실패를 확인한 뒤 구현했고 분수 배율의 실제 패널 경계 실패를 출력 좌표 직접 비교로 수정했다. 현재16개 모두 PASS이며 snapshot 읽기3회×180틱의 상태/지형/원본 RGB·사운드가 대조 실행과 일치한다. 실제 폭발의 충돌/표시 mask 차이·제거된 입자 기록, freeze/재시작·대기 입력·오류/음수 극값을 포함한다.

Python253·라이선스/기존 증거 감사 PASS. 소스 영향은 새 HD 모듈/통합 테스트와 lib export, Session의 읽기 전용 getter2개다. 현재438개 source pin과 G11의 연결 pin을 갱신하고 이전 pin·NOT READY를 보존했다. 워크스페이스·WASM과 정확한 CI/병합은 [결과 보고서](../reports/issue-140_hd_presentation.md)와 GitHub #140에 기록한다.

로컬 Rust637/doc29·workspace Clippy/format·WASM library build, Markdown10개/상대 파일 링크246개·공백/self-review PASS. PR을 게시해 실제 지원 플랫폼 CI를 확인한 뒤 병합/완료 처리한다.
