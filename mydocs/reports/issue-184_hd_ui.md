# #184 HD 글꼴·전체 게임 UI

이슈: https://github.com/edwardkim/ohmylemmings/issues/184 · 상위 #137

HD의 원본5×7 확대 글자를 Noto Sans SemiBold의 AA 글꼴로 교체하고 상태·기술·미니맵 패널, Pause·카운트다운·성공/실패 안내를 완성했다. 원본14개 조작 영역, 원본 모드·입력·논리/지형/사운드는 유지한다. 기술 이름/단축키·남은 개수·선택/소진 상태와 실제 구출 수/목표·남은 시간·재시도/다음 안내를 같은1920×1080좌표로 표시한다. 마지막 레벨에는 없는 다음 동작을 안내하지 않고 Replay/불러오기 실패를 구별한다.

## 자산과 구현

[글꼴 출처/해시·생성 규격](../../assets/manifests/hd_ui_font.json)과 [OFL 고지](../../third_party/NotoSans-OFL.txt)를 고정했다. [공식 원본](https://github.com/notofonts/noto-fonts/tree/ffebf8c1ee449e544955a7e813c54f9b73848eac)은 변경하지 않았고 다운로드 TTF는 ignored에 보존한다. FreeType2.14.3의48px grayscale coverage로 ASCII32..126의95자를 생성했다. 실행 중 시스템 글꼴/FreeType·추가 Rust/Python 패키지는 필요 없다. 지원 밖 문자는 물음표로 대체한다. 현재 게임 UI는 영문이며 한국어 번역/글꼴을 추가한 것은 아니다.

공통 Rust가 동일한 glyph/advance/RGBA를 Native/WASM에 제공한다. 글꼴3색 페이지와32px nine-slice UI skin의 추가 RGBA는4,730,880바이트이며 reset 때 만들고 표시 중 재사용한다. AA 모서리·그라데이션은 코드로 생성한다. 기존13아이콘·899개 게임 자산/50카탈로그 페이지·선택 범위·v1팩/12정수 draw wire는 유지했다. 글자 픽셀마다 draw하는 경로를 문자별 blit으로 바꿨다. 이 결과를60fps 성능 인증으로 확대하지 않는다.

대상은 현재 제품의 게임 UI다. 기존 Web HTML의 난이도/장면·파일·화면/전체화면·오디오 설정은 유지하며 실제 동작을 회귀 검증한다. Native/Web에 없던 DOS MAIN.DAT 메뉴/로고 화면을 새 게임 흐름으로 추가하지 않는다. 지원 HD 배포와 전체 리마스터 인수는 HD-06 후속이다.

## 검증

- Python 글꼴 atlas·핀·여백과 Rust 비례 advance/UTF-8 생략 테스트에서 미구현 실패 후 통과했다.95글자·94비공백 글리프·부분 alpha를 확인했다.
- 렌더러의8기술·읽기 전용 상태/마스크·글꼴 캐시 재사용, 성공/실패/마지막 레벨/Replay/불러오기 실패 안내를 검사했다.
- 실제 SDL7코스·826상태에서 원본/HD 입력·상태·지형/객체/사운드가 일치했다. 실제 WASM/Canvas도7코스826상태·38표시와8기술/Pause/미니맵을 통과했다. 실제 Fun01 UI는665상태·664틱·36표시·10구출, 전체화면/재시작·8기술/대상 할당과6레벨의 독립 선택 팩 대비 RGBA 전체 일치, 로딩 실패의 원본 복구/재시도를 통과했다.
- 워크스페이스709Rust 테스트·WASM 컴파일·game 전체 타깃 clippy·642소스 핀/313문서 링크·고지39개를 통과했다. 기존51개 카탈로그 구성원 전부가 바이트 단위로 같고25선택 범위 최대26페이지/124,698,404바이트도 유지했다.
- 실제 제품 비교 도구는 오래된 기본 팩 대신 전체 카탈로그를 독립 조합하도록 수정했다. 비동기 IIFE가 끝나기 전에 다음 장면으로 넘어가던 도구도 수정하고 최종6레벨 RGBA 일치/console 오류0을 확인했다. Python 전체·기존 Web 회귀·정확한 head CI와 실제 병합은 완료 기록에 고정한다.

재현: `python3 tools/hd_ui_font.py`와 `python3 tools/license_policy.py`, `cargo test --workspace --locked --offline`, `python3 tools/check_hd_ui.py --output assets/local/issue-184-review/new-native`. 글꼴 재생성은 `python3 tools/hd_ui_font.py --fetch --generate`(host C compiler/pkg-config/FreeType 개발 파일 필요)이며 일반 빌드·오프라인 검사는 이 도구를 실행하지 않는다. 별도 빈 Chromium 프로필에서 `tools/web/hd_ui.cjs`와 `hd_product.cjs`를 사용한다.

BGM #165는 리마스터 이후, #113 성능 이월/NOT READY는 유지한다. 사용자19222/8080·preview-172와 파일, 공개 v0.1.0/Pages는 유지한다.

[실제 Web 패널](../../assets/remaster/previews/issue-184_play.png) · [Pause](../../assets/remaster/previews/issue-184_paused.png) · [성공](../../assets/remaster/previews/issue-184_won.png) · [실패](../../assets/remaster/previews/issue-184_lost.png). 자체 합성 장면의1920×1080 Canvas이며 원본 레벨 바이트를 사용하지 않았다.
