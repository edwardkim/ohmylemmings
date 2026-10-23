# #181 회전 깃발·일방통행 표시 HD

이슈: https://github.com/edwardkim/ohmylemmings/issues/181 · 상위 #137

원본 GROUND0O..4O의 남은20객체/212디스크 프레임을 HD로 연결했다. 장식은 횃불이 아닌 녹색/청색 회전 깃발10종/140프레임이다. 방향 표시는10종/72프레임이며 화산 테마의8프레임·21px 높이와 다른 테마의7프레임·32px 높이를 유지한다. 방향7/8트리거·마스크·배치 플래그·논리 틱·사운드·원본 모드는 변경하지 않았다.

## 자산과 경계

[자산/프롬프트 안내](../../assets/remaster/README.md), [원본 메타데이터·출처·해시](../../assets/manifests/hd_decor.json), [확장 manifest](../../assets/manifests/remaster_decor.json). built-in image_gen의 선택 PNG2장은 편집하지 않았다. 깃발28포즈를5테마가 공유한다. 방향 표시는5색상·좌우2방향의72포즈를 사용하고 하이라이트로 맥동한다. 이는 새 장식 디자인이며 원본 화살표 스크롤 픽셀의 정확한 재현은 아니다. 런타임은 참조 사각형 RGBA를 그대로 복사하고 cutoff127을 표시할 때 적용한다.

기존48페이지 바이트가 일치하고 새2페이지의2,815,732참조 RGBA 샘플·표시 크기·기준점이 일치했다. 원본54객체의540디스크 키를 모두 갖춰 총899자산이다. 전체 카탈로그50페이지/51파일·204,704,712바이트, catalog SHA-256 `97704c7cfb4bdb9025cb6a067e37f7d3b575147e3e3b3e184cd9eceec2303a8e`. 베이스 팩131,699,864바이트는 유지된다. 25선택 범위는128MiB/32페이지 한도를 검사한다.

## 검증

- Python/Rust의 신규212키·경계 테스트에서 미구현 실패를 먼저 확인하고 통과시켰다. 비어 있는 객체 번호와 범위 밖 프레임은 거부한다.
- 렌더러는212루프 위상에서 상태·마스크·사운드를 보존한다. 기존 부분 팩은 원본 그림으로 대체할 수 있다.
- 실제 SDL5합성 코스/1,790상태에서 원본/HD 입력·상태·최종 지형/객체·사운드 기록이 일치했다. Digger의 실제 지형 변경과10스텝 Pause를 확인했다.
- WASM/Canvas5코스/1,790상태·670Canvas·전212위상을 통과했다. 실제 지형 제거로 원래 보이던70,319샘플이 투명화됨을 확인했다. 별도 빈 Chromium을 사용했고 console 오류0이다. 원본120장면과 선택1,920틱, 전체 검사·출처 감사·self-review·정확한 head의 CI·실제 병합은 최종 결과를 아래에 추가한다.

재현: `source assets/local/dev-tools/env.sh`(root 준비 환경), `make check`, `python3 tools/check_hd_decor.py --output assets/local/issue-181-review/new-native`, `cargo run -p lemmings-game --example hd_terrain_probe --release --locked --offline -- assets/local/web/hd-assets assets/local/dos-original`. 브라우저는 별도 빈 Chromium 프로필/CDP와 로컬 서버에서 `tools/web/hd_decor.cjs`·`hd_terrain.cjs --decor`를 사용한다. 원본 캡처는 ignored에만 보관한다.

현재 단계의 전체 UI·글꼴·지원 HD 배포와 리마스터 전체 인수는 남아 있다. 성능 #113 NOT READY와 BGM #165의 후순위를 유지한다. 사용자19222/8080·preview-172는 유지한다.
