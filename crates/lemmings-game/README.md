# lemmings-game

네이티브와 향후 웹 호스트가 공유하는 자산 조립·CPU 화면·패널·장면 전환입니다. [#64 계약](../../mydocs/tech_insights/issue-64_shared_game.md)을 따릅니다.

`Assets::from_source(&mut source, number)`는 Fun 1..=30의 기존 표준 그래픽 지원 범위에서 필요한 논리 자산 이름을 요청합니다. `AssetSource::read`는 `Result<Option<Vec<u8>>, Box<dyn Error>>`를 반환하며 같은 형태의 클로저도 사용할 수 있습니다. `Ok(None)`은 부재이고, 필수 파일이면 이름을 가진 `MissingAsset` 오류가 됩니다. `VGALEMMI.EXE`만 선택 사항이며 그 읽기 오류나 잘못된 프로필은 숨기지 않습니다. 경로·URL·네트워크·캐시 정책과 바이트 확보는 호스트가 담당합니다.

`LevelRun`은 자산에서 같은 공유 `Session`을 만들고, `View::frame`은 세션을 변경하지 않고 320×200 RGB 바이트를 반환합니다. `panel`은 논리 좌표의 기술 버튼·미니맵·누름 반복과 표시를, `pointer`는 선택·캡션을 처리합니다. `prepare`가 새 레벨 조립에 실패하면 기존 결과·세션·장면은 보존됩니다.

이 크레이트는 파일·SDL·시스템 시각·브라우저 API를 호출하지 않습니다. 자산 파서는 resources, 시뮬레이션은 session/core에 남아 있습니다. core/resources는 no_std를 유지합니다. 데스크톱의 파일 읽기와 SDL 입력·프레임 표시가 이 API를 사용합니다.

```sh
cargo test -p lemmings-game --locked --offline
cargo build -p lemmings-game --lib --target wasm32-unknown-unknown --locked --offline
```

`tests/`의 합성 DAT/GROUND/MAIN 바이트는 자체 작성한 fixture이며 원본 파일 없이 실행됩니다. JS/WASM 바인딩·Canvas·브라우저 입력은 G09 후속 단계입니다.

## 오디오 소유 경계 / #80

`audio::AudioOutput`은 불변 음원 bytes·opaque VM/PCM continuation을 소유하고 장치 없이 typed batch→bounded mono PCM을 제공합니다. decode_adlib는 선택적 호스트 bytes를 읽고 demo_image는 독립 작성한 연습 데이터를 만듭니다. 게임 tick/렌더·장치 시간/I/O는 호출하지 않습니다. [계약](../../mydocs/tech_insights/issue-80_audio_output.md).
