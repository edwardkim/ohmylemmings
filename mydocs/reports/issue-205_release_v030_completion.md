# #205 v0.3.0 게시·공개 Pages 최종 인수

이슈: https://github.com/edwardkim/ohmylemmings/issues/205

2026-10-24 메인테이너 지시로 **v0.3.0 정식/latest 릴리즈와 GitHub Pages 게시를 완료**했다. [#202 로컬 MP3 BGM](issue-202_local_mp3.md)을 포함하며, 2D HD·전체 원본 게임·레벨별 점진적 다운로드/진행률을 유지한다.

- [v0.3.0 다운로드](https://github.com/edwardkim/ohmylemmings/releases/tag/v0.3.0): Windows11 x64·macOS universal2·Linux x86_64·Web과 대응 소스/체크섬. 정식 공개이며 draft/시험판이 아니다.
- [공개 게임](https://edwardkim.github.io/ohmylemmings/) · [deployment.json](https://edwardkim.github.io/ohmylemmings/deployment.json): 동일 태그/소스/Web ZIP을 가리킨다.
- [버전별 실행 안내](../releases/issue-205_v0.3.0.md): Web 파일 선택, Native 파일 드롭/`--bgm FILE`, 한 곡 반복·음악/효과음 음량·오류/재시도. 사용자 음원은 업로드/번들링하지 않는다.

## 소스·CI·게시 계보

| 항목 | 고정 값 |
|---|---|
| 버전 PR | [#206](https://github.com/edwardkim/ohmylemmings/pull/206), self-review·실제 merge 완료 |
| CI head | `4ba95fe4d1a0c6fd245fa38580cfe3f31e06681e` |
| CI checkout | `5f08979329ab805fd039b8c0220b15f37a7a533f` |
| 실제 main/tag commit | `711505c35dafe2cec5e3ef4e7f4ec1d2c1a7250c` |
| 공통 source tree | `206c39dfc8b43ba82f81d606c1c043e17e791aea` |
| annotated tag object | `65b442693fe590cda10bdb8061b186c2871b368f` |
| 전체7개 CI | [40060983958](https://github.com/edwardkim/ohmylemmings/actions/runs/40060983958), 모두 SUCCESS |
| 태그 전4패키지 prepare-only | [40062239585](https://github.com/edwardkim/ohmylemmings/actions/runs/40062239585), SUCCESS·게시/Pages 단계 SKIPPED |
| tag push 정식 게시·Pages | [40062581138](https://github.com/edwardkim/ohmylemmings/actions/runs/40062581138), publish/Pages 모두 SUCCESS |

workspace/lock9개 crate를0.3.0으로 맞추고 버전별 노트와 현행 소스 연결 기록을 갱신했다. #202 이후 제품 로직/자산/외부 의존성 변경은 없다. CI의 실제 checkout·head·main merge tree가 같다. 중복 push/main CI는 이 성공 증거 확인 후 취소했고 성공으로 집계하지 않았다. 릴리즈 준비의 Python319개·release/publish/Pages 단위21개·버전/라이선스·COMP/G11/성능 자료 감사도 통과했다. #202의 Rust724개·원본21곡/C/WASM 출력·플랫폼 설치 검증은 [기능 결과](issue-202_local_mp3.md)를 따른다.

## 배포 파일 검증

4아카이브·각 체크섬·공통 소스·PLAY.md·RELEASE-MANIFEST.json·SHA256SUMS의 **12파일**을 draft에 업로드하고 전부 재다운로드해 일치를 확인한 뒤 공개했다. 게시 도구가12개 공개 URL도 인증 없이 전부 다운로드해 검증했다. 모든 플랫폼의 대응 소스는1137파일이다. 사전 검증·게시 artifact·공개 manifest/체크섬이 같으며 GitHub asset digest·크기와 일치한다.

| 플랫폼 | 공개 아카이브 SHA-256 |
|---|---|
| linux-x86_64 | `a3662e36bb48b968194412cdd8221f1bca023fc8d8f4631d6e9967cfa3a88ef4` |
| web | `1022c380fa135b30e81442a784dbbf5887ab078bf1a75e2f2b2a4f0fbe4217fd` |
| windows-x86_64 | `062b0fe55d6833f68caa7eb9e15dc0a67c9a13fc0761538a4f8d1f266b04a3a0` |
| macos-universal2 | `276a305d647277946f22ccc1070152770af7887fb13a548f4c9ae9a7468b3f47` |

공개 [RELEASE-MANIFEST.json](https://github.com/edwardkim/ohmylemmings/releases/download/v0.3.0/RELEASE-MANIFEST.json) · [SHA256SUMS](https://github.com/edwardkim/ohmylemmings/releases/download/v0.3.0/SHA256SUMS). Pages는 재빌드 없이 동일 Web ZIP의109파일을 승격했다. 배포 metadata의 archive SHA·tag·release commit·tree가 위 계보와 같다. 공개 HTTP의 HTML/JS/WASM·설정18파일도 실제 ZIP과 바이트 단위로 같고 WASM MIME은 application/wasm이다.

## 공개 웹·음악 검증

사용자 브라우저와 분리한 Chromium19394에서 실제 HTTPS 사이트를 실행했다. 기본 HD1920×1080·원본 Fun01·95글꼴·전체337캐릭터 카탈로그/현재99프레임·진행률 UI를 확인했다. 필수8페이지/19,751,567바이트를 먼저 받는다. 원본 표시 전환도 시뮬레이션 상태를 바꾸지 않는다.

664틱/665상태·23표시에서 독립 원본 게임과 상태를 비교하고 **10구출 Won**을 확인했다. 브라우저 예외0, 원본 WebAudio context running·4,608샘플 제출·4,548비영 PCM을 확인했다. 공개 확인은 고정 틱 기능 검사이며 성능 측정은 아니다.

메인테이너 제공 두 로컬 MP3(44.1kHz stereo)를 공개 페이지의 실제 파일 입력으로 선택했다. 각2048개 비영 샘플·곡 끝 반복·곡 교체/이전 정지·숨김/복귀·해제 후 원본 복귀를 확인했다. MP3 재생 중 원본 효과음 9,661개 비영 PCM도 출력됐다. 업로드 요청0·브라우저 예외0이다. 버전0.3.0 로컬 Native를 다시 빌드하고 같은 두 파일의 실제 SDL stereo 소비·모드 전환/숨김 probe도 통과했다. 앞선 실제 Native 게임 SDL 출력 검증은 #202 보고서를 따른다.

## 보존·완료 범위

v0.1.0/v0.2.0의 태그 object/target와 각12개 asset ID·크기·digest·게시/수정 시각·릴리즈 본문을 전후 대조해 불변을 확인했다. 과거 버전 노트를 수정하지 않았다. 게시된 v0.3.0 태그·asset·버전 노트도 이후 덮어쓰지 않는다.

로컬 main/Native/Web을0.3.0으로 동기화했다. Web은 실제 공개 ZIP과 짝을 이루는 WASM·대응 소스/고지를 사용한다. 사용자19222 브라우저·8080서버와 preview-172/165/196/202·Music 원본을 보존했다. 사용자 음원/파일명·경로·해시·파생 PCM과 개인 QA 자료는 ignored assets/local/에만 유지하며 공개 소스/배포에 포함하지 않았다.

#205의 게시 요구를 완료한다. **#202는 새 MP3 사람 청취 응답만 대기로 OPEN**을 유지한다. #165의 기존 원본 BGM 청취 인수와 #113 성능/구간 메모리 NOT READY, Mac 미공증/Windows11 지원 범위를 유지한다.
