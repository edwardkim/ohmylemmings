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
- WASM/Canvas5코스/1,790상태·670Canvas·전212위상을 통과했다. 실제 지형 제거로 원래 보이던70,319샘플이 투명화됨을 확인했다. 별도 빈 Chromium을 사용했고 console 오류0이다. 원본120초기 장면·선택1,920틱·Web352Canvas도 Native/WASM 최종 상태와 모드별 상태/마스크/사운드가 일치했다. 원본14장면의 방향 제한과 이번 대상 중 실제 배치6종을 확인했다. 캠페인에 미배치된14종은 위 합성 코스로 검증한다. 원본 그림 fallback0·console 오류0이다.
- `make check`의 모든 구성 명령을 통과했다: 워크스페이스705Rust·304Python, WASM 컴파일, 실제 SDL 회귀, Web8개 검사 명령 종료0. 마지막 solution replay는 증거 보존 옵션을 사용했다.627소스 핀·310문서 링크·기록된 G11 무결성도 통과했다.
- self-review에서 v1 키 인코딩·경계·부분 팩 대체, PNG/프롬프트/출처, 두 방향의 실제 Canvas 화면, RGBA/기준점과25범위 예산을 검토했다. Web 검사 stride를 실제1584에 맞추고 합성 코스의 지형 인코딩/기술 입력 경계를 수정한 후 최종 증거를 다시 생성했다. core/session/resources/audio/platform 제품 코드는 바꾸지 않았다.


재현: `source assets/local/dev-tools/env.sh`(root 준비 환경), `make check`, `python3 tools/check_hd_decor.py --output assets/local/issue-181-review/new-native`, `cargo run -p lemmings-game --example hd_terrain_probe --release --locked --offline -- assets/local/web/hd-assets assets/local/dos-original`. 브라우저는 별도 빈 Chromium 프로필/CDP와 로컬 서버에서 `tools/web/hd_decor.cjs`·`hd_terrain.cjs --decor`를 사용한다. 원본 캡처는 ignored에만 보관한다.

현재 단계의 전체 UI·글꼴·지원 HD 배포와 리마스터 전체 인수는 남아 있다. 성능 #113 NOT READY와 BGM #165의 후순위를 유지한다. 사용자19222/8080·preview-172는 유지한다.

## CI와 실제 병합

PR #182 head `0744178020c71e6b84024ef2ff00a204140d35aa` / CI39955094648의7개 SUCCESS, 실제 merge `331f512f625921d792f855c8e7a28d93144c6ed4`와 검토 head 전체 tree 일치를 확인했다.

[PR #182](https://github.com/edwardkim/ohmylemmings/pull/182) · [CI39955094648](https://github.com/edwardkim/ohmylemmings/actions/runs/39955094648). Python·WASM·Rust native·Web·Windows·macOS Apple Silicon/Intel을 통과했다. PR checkout `600f96f1c703e3cb1ec1a2fc1cfbe80f3b1a5023` / tree `debcd551247cc3ed03ccae8b3232bcc13f92a57f`를 검토 head와 대조했다. attempt1의 Web은 준비11분26초 뒤15분 작업 제한으로 종료됐다. 같은 head의 attempt2에서 Web만 재실행해 성공했고 다른6개는 attempt1 성공 결과를 유지했다. 시간 제한 종료와 중복 push39955085721 취소는 PASS에 포함하지 않는다. 기존 플랫폼 패키징 검사와 전체 HD 배포/성능 인수는 구별한다. 완료 문서 실제 병합과 root Native/Web 준비·preview-172 보존은 [#181](https://github.com/edwardkim/ohmylemmings/issues/181)의 종료 근거에 기록한다.
