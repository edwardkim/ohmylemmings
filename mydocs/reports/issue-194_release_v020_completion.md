# #194 v0.2.0 게시·공개 Pages 최종 인수

이슈: https://github.com/edwardkim/ohmylemmings/issues/194 · https://github.com/edwardkim/ohmylemmings/issues/196

2026-10-24 메인테이너의 릴리즈 지시에 따라 **v0.2.0 정식 릴리즈와 GitHub Pages 전환을 완료**했다. 추가 요구한 레벨별 점진적 다운로드와 실제 진행률/재시도/전체화면 UX도 포함했다.

- [v0.2.0 다운로드](https://github.com/edwardkim/ohmylemmings/releases/tag/v0.2.0): Windows11 x64·macOS universal2·Linux x86_64·Web, 전체 원본 데이터/HD와 대응 소스/체크섬 포함. 정식/latest이며 draft나 시험판이 아니다. 게시 시각은2026-10-24 11:13:11 KST다.
- [공개 게임](https://edwardkim.github.io/ohmylemmings/): [deployment.json](https://edwardkim.github.io/ohmylemmings/deployment.json)이 v0.2.0/동일 소스/아카이브를 가리킨다.

## 소스·CI·게시 계보

| 항목 | 고정 값 |
|---|---|
| 코드 PR | [#197](https://github.com/edwardkim/ohmylemmings/pull/197), self-review·실제 merge 완료 |
| CI head | `70e22ff5aa90797bc3ba967f135461c904c11539` |
| CI checkout | `ce91efd28e08115e46fe60b98f83aa7cd51416f7` |
| 실제 main/tag commit | `063fb535e142eb51f65a590759abf9d47c86ca38` |
| 공통 source tree | `24afcd911d5649d77e3d1556cbc56f9681710712` |
| annotated tag object | `2e5ea4398fc76b4dbf7c2c6b0317174dc51d10b9` |
| 전체7개 CI | [40051273994](https://github.com/edwardkim/ohmylemmings/actions/runs/40051273994), 전부 SUCCESS |
| 태그 전4패키지 prepare-only | [40052497737](https://github.com/edwardkim/ohmylemmings/actions/runs/40052497737), SUCCESS·게시/Pages 단계 SKIPPED |
| tag push 정식 게시·Pages | [40052908152](https://github.com/edwardkim/ohmylemmings/actions/runs/40052908152), publish/Pages 모두 SUCCESS |

#195의 준비 CI는 해당 준비 스냅샷에만 적용했다. #196 추가 구현 후에는 별도 동일 tree CI7개를 통과했다. source pin 누락은 당시 producer/성능값을 보존하며 갱신했고, 느린 연속 전송의 무응답 타이머도 검증한 최종 head70e22ff를 사용했다. main의 중복 CI 취소는 이 성공 PR CI와 병합 tree 일치를 확인한 뒤 수행했으며 취소된 실행을 성공으로 집계하지 않았다. Web 상태 조회가 대기로 보인 기간에도 실제 완료 결과는 SUCCESS였다.

## 배포 파일 검증

4개 아카이브·각 체크섬·공통 대응 소스·PLAY.md·RELEASE-MANIFEST.json·SHA256SUMS의 총12파일을 draft에 업로드하고 전부 재다운로드하여 바이트 일치를 확인한 뒤 공개했다. 게시 도구가 공개 URL12개도 인증 없이 전부 받아 일치를 검증했다. 각 플랫폼의 source1096파일이 현재 태그와 일치한다. 사전 검증 manifest와 공개 manifest도 동일하다.

| 플랫폼 | 공개 아카이브 SHA-256 |
|---|---|
| linux-x86_64 | `dc3f1566814f1a69a6bec009600d0ebffec0081ff249e65b11321ff257502a70` |
| web | `aee6c24c2020666a94dab0688be78b80f6cd1cd118735699322f81a0a934e368` |
| windows-x86_64 | `2789f76d76003e11984c50dbcde4c8371c1ee9e88d786eee25026c936b47a8e2` |
| macos-universal2 | `6eb3d5c232f23b2650d8242f007967048ded67da2b29251237ec0c3c8cd92ab8` |

공개 [RELEASE-MANIFEST.json](https://github.com/edwardkim/ohmylemmings/releases/download/v0.2.0/RELEASE-MANIFEST.json), [SHA256SUMS](https://github.com/edwardkim/ohmylemmings/releases/download/v0.2.0/SHA256SUMS)의 크기/해시가 GitHub asset digest와 전부 일치한다. 별도 검증에서 manifest·체크섬·PLAY를 익명으로 받고 교차 확인했다. Pages는 재빌드 없이 Web ZIP의108파일을 승격했으며 archive SHA `aee6c24c2020666a94dab0688be78b80f6cd1cd118735699322f81a0a934e368`가 같다.

## 공개 웹 인수

사용자 브라우저와 분리한 Chromium19394에서 실제 HTTPS 사이트를 실행했다. v0.2.0/원본 Fun01·기본 HD·1920×1080·95글꼴·전체337캐릭터 키 카탈로그를 확인했다. 현재 레벨의8페이지/19,751,567바이트를 먼저 준비하며 현재 필요한99캐릭터 프레임과 시설이 로드됐다. 진행률 UI가 있고 원본 전환도 동작한다.

664틱/665상태·23표시로 Fun01 10명을 구출해 Won을 확인했다. 브라우저 예외는0, WebAudio context running·3,072샘플 제출·3,012비영 PCM을 확인했다. 원본/HD 공통 상태·픽셀/실제 바이트 진행률·캐시0재다운로드·다음 레벨4파일·실패/같은 상태 재시도·전체화면·느린 연속 전송/무응답의 로컬 인수는 [#196 보고서](issue-196_web_progressive.md)를 따른다.

## 보존과 남은 항목

v0.1.0 태그 object/target와12개 asset ID·크기·digest·게시/수정 시각을 비교해 불변을 확인했다. 과거 버전별 노트도 보존했다. 게시된 v0.2.0 태그·asset·버전별 노트 역시 이후 덮어쓰지 않는다. 사용자19222/8080/preview-172/165와 파일을 보존하고 새 preview-196을 추가했다. 개인 원시 자료는 ignored assets/local/issue-196-review에 유지한다.

#194/#196의 릴리즈·웹 요구를 완료한다. BGM #165의 제품/디지털 검증은 완료했으나 메인테이너의 새 청취 응답이 없어 **#165 OPEN을 유지**한다. #113의 성능/구간 메모리 NOT READY, Mac ad-hoc/미공증과 Windows11 지원 범위는 바꾸지 않는다. 이번 공개 확인을 사람의 청취·새120레벨 HD 완주·60fps 합격으로 확대하지 않는다.
