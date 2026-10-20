# G10-01 AdLib 자원 검증 결과

이슈: [#72](https://github.com/edwardkim/ohmylemmings/issues/72) · 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1)

ADLIB.DAT의 전체21곡/18효과음 메타데이터를 no_std 자원 API로 연결했습니다. [계약](../tech_insights/issue-72_adlib_catalog.md) · [출처/해시](../../assets/manifests/adlib_catalog.json). 실제 음악 명령 실행·합성/재생은 다음 절편입니다.

## 구현·대조

AdlibFile은 정확히 한 DAT 구간과22125바이트 선언 크기를 확인하고 기존 decoder로 호출자 버퍼에 해제합니다. AdlibImage는21곡/18효과음을 모두 선검증하며 헤더 값·악기/프로그램/첫 bar·효과음 주소와120개 표시 레벨의21곡 순환을 제공합니다. I/O·할당·플랫폼·게임 시계 의존성이 없습니다.

| 자료 | 곡 | 채널 | 효과음 | 레벨 대응 | 결과 |
| --- | ---: | ---: | ---: | ---: | --- |
| 공개 가능한 합성 fixture | 21 | 48 | 18 | 120 | 실제 Rust inspection과 전체 필드 일치 |
| 고정 원본 | 21 | 69 | 18 | 120 | Rust·기존 목록·고정 TS 초기화 함수 전체 필드 일치 |

원본50파일과 TS4파일의 크기/SHA를 확인하고 해제 SHA `ad4f33eb383c60b5a67945987f27580fc0540e38558d4c158efe1a672db5f490`도 기존 기준과 일치했습니다. 종료 후 원본50파일 해시도 동일합니다. TS의 실제 initMusic/initSound/채널 initMusic와 reader 함수 본문을 실행하며 채널 생성은 메타데이터 stub입니다. command VM/OPL/PCM이나 DOS 소리를 실행한 증거로 계산하지 않습니다.

TS 설정은20곡이지만 고정 G00/Lemmix 계약을 따라21곡을 공급했습니다. readWordBE가 LE인 점, 여러 곡이 bar21727을 공유하는 점, music-09의 채널 주소 순서를 확인했습니다. 파서는 전체 음악 영역의 초기 읽기만 검증하고 곡별 주소 정렬/분할을 요구하지 않습니다.

Rust9개와 Python6개를 TDD로 추가했습니다. 절단/추가·빈/과도 채널·0 factor·인덱스/주소 범위·잘못된 DAT 수/압축·21번째 누락·공유 bar/채널 순서·전체 필드 비교/숫자형/원본 보관 경계를 검사합니다. make check-rust에 원본 없는 실제 inspection 대조를 연결했습니다. Cargo 의존성/lockfile·공통 게임 규칙·입력·렌더·기존 브라우저 파일은 변경하지 않았습니다.

전체 make check 종료0: Rust535/문서29/Python121·포맷/Clippy·오프라인 빌드/상태 재생·합성 AdLib 전체 필드 대조·WASM·전체 SDL/기술/판정/패널/포인터·기존/신규 브라우저 회귀를 통과했습니다. 로그 SHA-256 `5b8b601a65c70071b7380c9f6fb093143cf1e07ec4f0ba637c8c1e104abf5090`. 자체 검토에서 초기 읽기 범위·공유 데이터/순서·레퍼런스 실행 범위·변경 격리·후속 통합 계약을 확인했고 미해결 항목은 없습니다. 정확한 push/PR/main 네 작업 CI·실제 전체 로그·같은 병합 트리와 이슈 마감은 #72에서 기록합니다.

## 다음 단계

18개 논리 이벤트와 숫자ID를 구분하고 게임 경계→레지스터 스트림→PCM→SDL/Web 출력을 분리했습니다. 음악 시계를 게임17Hz/화면 callback에서 독립시키고, Pause/숨김/결과·음소거/볼륨·초기화·장치 오류와 실제 DOS 근거를 후속 검증으로 명시했습니다. LAN HTTP를 유지하는 웹 PCM 출력 경로도 후속 실측 항목입니다.

다음 G10-02는 ADLIB 명령 해석과 레지스터 스트림·반복/정지·타이밍 근거입니다. G10 전체·OPL/PCM/장치 재생·음원/이벤트 DOS 매핑은 아직 완료하지 않았습니다. 원본 파생물은 ignored assets/local에 보관하며 공개 저장소에는 합성 fixture와 출처/크기/해시·계약만 추가합니다.
