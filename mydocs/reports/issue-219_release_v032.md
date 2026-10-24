# v0.3.2 릴리즈·Pages 완료

담당 [#219](https://github.com/edwardkim/ohmylemmings/issues/219). [다운로드](https://github.com/edwardkim/ohmylemmings/releases/tag/v0.3.2) · [공개 게임](https://edwardkim.github.io/ohmylemmings/) · [배포 정보](https://edwardkim.github.io/ohmylemmings/deployment.json).

## 변경과 인수

#113 FPS/피드백 표시, #215 HD 질감·수직벽 개선, #216 굴착 타격감·절차적 효과음·원형 조준·추락 충격·수량0 선택 차단을 포함한다. #216은 메인테이너가 “모두 통과 판정입니다.”로 플레이와 청취를 인수했다. 원본 게임 규칙·기존6 PCM은 유지하며 새 합성 trace만 확장했다. #113 성능 NOT READY는 별도다.

## 게시 계보

| 항목 | 값 |
|---|---|
| 제품 PR | [#220](https://github.com/edwardkim/ohmylemmings/pull/220), self-review·인수·병합 완료 |
| PR head | `08ca135a38e694e4ea319bd20c616b24d1dd926b` |
| 실제 CI checkout | `be6fcae2f10000a12e4bc91731892e6b7df1afa9` |
| main/tag commit | `3f7bbdf54056cefd1dd8b5a681619dd237b33656` |
| 공통 source tree | `3cd8ef042a5a78da468adc1baa9ac7cf13014af2` |
| annotated tag object | `326a5ba83a329d9b4d0f10e63f5a784e8907295f` |
| 플랫폼 CI7개 | [40090002104](https://github.com/edwardkim/ohmylemmings/actions/runs/40090002104), 모두 SUCCESS |
| 태그 전 사전 검증 | [40091260950](https://github.com/edwardkim/ohmylemmings/actions/runs/40091260950), SUCCESS |
| 정식 게시·Pages | [40091521149](https://github.com/edwardkim/ohmylemmings/actions/runs/40091521149), publish/Pages SUCCESS |

병합 커밋의 skip-ci로 동일 소스 CI 중복 실행을 생략했다. annotated tag 게시 후 정식 Release workflow를 해당 태그에서 명시적으로 dispatch했다. 과거 태그를 재지정하지 않았다. PR과 실제 CI checkout, 병합/tag의 tree가 같다.

4개 패키지·공통 대응 소스·PLAY.md·체크섬·manifest의12파일을 공개했다. 각 패키지의 대응 소스1276개를 검증했으며 사전 검증/게시/공개 RELEASE-MANIFEST가 동일하다.

| 플랫폼 | SHA-256 |
|---|---|
| linux-x86_64 | `ac09d7df30a31063cdfb25a116690c290f00d54ab25bd4ab2d0027042e79b37c` |
| web | `842d06edc3c1f3eb24d904acb96cc15bc55147cd13c3358e9e2097859a41af96` |
| windows-x86_64 | `ce5ab791dcfc013cda6cdf0c9d2409560a6a75dbdbd43009a6a4140ca795f095` |
| macos-universal2 | `809ac6b95db7eb269544a06246841185ec41d22eba5ceb804fbd0cd7f684eee5` |

## 공개 검증

- 실제 공개111파일의 내용과 SHA-256이 릴리즈에서 추출한 사이트와 일치하고 WASM MIME도 통과했다. Pages는 같은 Web ZIP을 재빌드 없이 승격했다.
- 실제 공개 Fun02: Digger0/Floater10, 키/툴바 선택 차단, Native241상태 일치, Splatting 표현. 굴착4회/PCM peak4156, 효과음 음소거·일시정지·모드 상태 일치, console 오류0.
- 실제 공개 Fun01: 툴바8개·배정·일시정지·미니맵·재시작·전체화면,664틱/665상태·10구출 Won, console 오류0.
- 기존4개 릴리즈의 본문·태그·asset ID/크기/digest/시각을 전후 대조해 불변을 확인했다. 기존 버전 노트도 수정하지 않았다.
- CI Web 전체 게임 패키지에서120레벨 로드·진행률·HD 기본·원본 전환·오류 경로·WebAudio PCM을 통과했다. Windows11/macOS Apple Silicon·Intel/Linux 패키지 검사도 성공했다.

원본 유래 화면·PCM/로그는 ignored `assets/local/issue-219-release/`에만 보관한다. 사용자 CDP19222·HTTP8080·기존 미리보기와 로컬 음악은 보존했다. 공개 검증은 별도19396 QA 브라우저로 수행하고 종료했다. 새 성능 인수나 모든 레벨의 수동 플레이 결과는 아니다.

#216 완료/closed. #219의 릴리즈·Pages 게시 요구도 완료했다.
