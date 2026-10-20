# G11-02 난이도·레벨 ID와 전환 계약

이슈: [#87](https://github.com/edwardkim/ohmylemmings/issues/87) · [G11 계획](../plans/issue-85_compatibility_validation.md).

## ID와 선택

기존 no_std `LevelId(Difficulty, 1..=30)`를 공통 장면 로더·`LevelRun`·SDL/Web 전환의 정식 ID로 사용한다. `CatalogEntry::for_level`이 DAT/부품/ODDTABLE을 선택하며 시뮬레이션은 난이도 문자열이나 호스트 I/O에 의존하지 않는다. 난이도 이름은 fun/tricky/taxing/mayhem이며 ASCII 대소문자를 허용하고 잘못된 이름·공백·범위 밖 번호는 거절한다.

`Assets::from_level_source(source, id)`와 `LevelRun::new_for_level`이 새 경로다. 기존 `Assets::from_source(source, number)`·`LevelRun::new`는 Fun 호환 진입점이다. SDL은 `--difficulty tricky --level 1`을 받으며 생략하면 Fun이다. 원시 재생 입력의 경계/순번/명령 형식은 유지하고 캡처 config·전환 기록에 난이도를 추가한다.

Web의 `load_original_rank(bank, difficulty, number)`·`start_level_rank(difficulty, number)`는 전체 ID를 검증한다. 기존 `load_original(bank, number)`·`start_level(number)`는 항상 Fun으로 해석한다. 현재 난이도를 유지하려면 `restart_level`이나 새 API를 사용한다. `level()`은 이전처럼 번호를 반환하고 `difficulty()`와 View의 difficulty/next_difficulty가 난이도를 제공한다. 버전1 시뮬레이션 상태/정수 입력 해시는 바꾸지 않는다.

## 전환 근거와 한계

고정 Lemmix `40e9bc34451f0e9127fd53b290a3877e700d143d`의 [Styles.Base.pas](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Styles.Base.pas#L950) `TSection.DoAddLevelInfo`는 새 section의 첫 레벨을 이전 section의 마지막 레벨에 연결한다. [GameScreen.Postview.pas](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/GameScreen.Postview.pas#L142)는 성공이면 Next로 이동하고 Next가 없는 성공을 최종 완료로 표시한다. 기존 고정 Styles.Dos의 Fun→Tricky→Taxing→Mayhem 각30개 생성 순서를 함께 적용했다.

추가 두 소스의 Git blob SHA-1을 고정 commit의 GitHub tree와 대조하고 크기/SHA-256을 [증거 목록](../../assets/manifests/rank_selection.json)에 기록한다. 이것은 기준 소스 확인이며 실제 DOS의 네 난이도 마지막 레벨 완주를 새로 관측한 증거는 아니다.

| 상태/동작 | 대상·음향 |
| --- | --- |
| 진행 중 결과 Continue | 이동하지 않음 |
| 실패 후 Continue / 결과 Restart | 같은 난이도/번호, 기존 normal cursor 유지·초기화 정책 재사용 |
| 성공 1~29 Continue | 같은 난이도의 다음 번호 |
| Fun30/Tricky30/Taxing30 성공 Continue | Tricky1/Taxing1/Mayhem1, 성공 음악 cursor 1회 전진 |
| Mayhem30 성공 Continue | 대상 없음, 결과·음향 유지 |
| 명시적 선택/재시작 | 완성된 자원·세션을 조립한 뒤 교체, 기존 수명/새 세대 정책 재사용 |
| 파일/장면 조립 실패 | 기존 ID·세션·입력·시계·맵·View·음향 세대/사건 보존. 결과 전환 실패는 기존 오류 표시 정책 유지 |

## 현재 지원 범위와 검증 경계

일반 장면116개가 소스 진입 범위다. 특수 Fun22/Tricky14/Taxing15/Mayhem22는 공통 장면이 계속 거절하며 G11-03에서 연결한다. 따라서 성공 후 특수 레벨로 가는 시도도 오류를 표시하고 기존 결과를 유지한다. 레벨 건너뛰기를 자동으로 하지 않는다.

창작 바이트의 네 난이도1/30 선택·재시작, 세 번의 난이도 경계 Continue와 Mayhem30 종료, 잘못된 ID/파일 교체 보존을 Rust와 실제 WASM·제품 DOM에서 검증한다. 성공 fixture도 유효한 필요 구출1/총원1과 지형/출구를 사용하며 성공 상태를 주입하지 않는다. 원본 일반 레벨의 시작/입력 대조와 기존 Fun 완주는 별도 증거다. 전체120개 초기 재생·유효 해법 완주·성능 인증은 후속 절편에 남긴다.

이전 [#85 점검표](issue-85_compatibility_matrix.md)와 manifest는 당시 기준 commit의 역사적 점검이므로 해시·29/91 집계를 덮어쓰지 않는다. #87의 지원 범위와 실행 증거를 따로 추가한다.
