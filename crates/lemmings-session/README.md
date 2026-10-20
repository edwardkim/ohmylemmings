# lemmings-session

네이티브와 향후 웹 호스트가 공유하는 소유 게임 세션입니다. [#62 계약](../../mydocs/tech_insights/issue-62_shared_session.md)을 따릅니다.

`Session`은 core의 규칙으로 입력·개체·지형·수량·객체·완료 결과를 조립합니다. 호스트가 디코딩한 데이터를 구성하고 `queue`/`tick`으로 진행하며, `due(Duration)`는 명시적으로 전달한 경과 시간의 호출 수만 계산합니다. 파일·시스템 시각·렌더·SDL·브라우저 API는 호출하지 않습니다. core/resources는 no_std를 유지하고 이 크레이트는 std의 소유 컬렉션을 사용합니다.

```sh
cargo test -p lemmings-session --locked --offline
cargo build -p lemmings-session --lib --target wasm32-unknown-unknown --locked --offline
```

WASM 라이브러리 컴파일은 JS 바인딩이나 실제 브라우저 실행 검증과 별도입니다. 자산 공급·공유 장면은 [lemmings-game / #64](../lemmings-game/README.md)에서 연결합니다. WASM/Canvas 어댑터는 G09 후속 절편입니다.
