# #202 사용자 로컬 MP3 BGM 구현·검증

이슈: https://github.com/edwardkim/ohmylemmings/issues/202

상태: 구현·로컬 검증 진행. 기준 main `3ee06c63bd33fab9456c3383c7422b7a97231cf4`. [계획](../plans/issue-202_local_mp3.md) · [선행 검토](issue-200_custom_bgm_review.md).

Web은 별도 MP3 선택과 원본/내 음악/끄기, 음악·효과음·전체 음량, 한 곡 반복, 준비/재생/차단/오류/재시도·파일 해제를 제공한다. HTMLMediaElement와 object URL을 사용하며 업로드·영구 파일 복사·전곡 WASM/AudioBuffer 디코딩을 하지 않는다. 새로고침 후 재선택한다.512MB 이하 DRM 없는 MP3가 대상이다.

Native는 MP3를 창에 드롭하거나 `--bgm FILE`로 선택한다. 별도 파일 선택 대화상자는 없다. F6 원본/MP3/끄기, F7 재시도, F5 파일 해제, `[`/`]` 음악·`,`/`.` 효과음 음량을 지원하고 창 제목에 상태를 표시한다. MP3 worker의8개×1024프레임 큐와 현재 블록,64KiB 입력 버퍼·제한된 메타데이터/커버아트 처리로 파일 전체 디코딩을 피한다. mono/stereo MP3를 장치 표본율로 변환하고 원본 효과음과 스테레오 출력에 혼합한다. 파일 읽기/디코딩은 게임/장치 처리 스레드 밖에서 수행하며 교체 시 이전 수신 큐를 해제한다.

공통 OPL 드라이버의 음악/효과음 사건·채널 배정·시계는 유지한다. 합성 채널의 출력 gain만 분리하고 release tail의 역할을 보존한다. Unity100/100은 기존 integer PCM과 같으며0gain에서도 envelope/oscillator/VM은 계속 진행한다. 원본/내 음악/끄기 선택은 출력만 바꾸고 시뮬레이션에는 들어가지 않는다. 결과/숨김에서 음악을 정지하며 새 세대·재시작/복귀에서 처음부터 재생한다. 게임 Pause에서는 계속 재생한다. MP3 실패는 게임/원본 효과음을 중단하지 않는다.

Symphonia0.5.5의 MP3 feature만 Native에 고정했다. 총30개 registry 의존성의61개 고지와 대응 소스 URL을 license_inventory/THIRD_PARTY_NOTICES에 기록했다. Symphonia는 MPL-2.0이며 수정하지 않았다. LGPL OPL 변경 기록/patch/소스 전달을 갱신했다. 프로젝트 자체 MIT와 음원 권리는 별개이며 상용 음원을 포함하지 않는다.

## 로컬 검증

- TDD red: AudioOutput의 set_mix 부재로 새 테스트 실패, 구현 후 음악 무음·전체18효과음 지속/시계·unity 동일성/잘못된 음량 원자성 검사 통과.
- 고정 C buffered 대조24사례/3,502,528프레임 통과. 기존 PCM golden을 변경하지 않았다.
- Native worker: 합성 MP3 stereo/loop·표본율/상한 블록·없는/잘못된 파일 검사 통과. 실제 SDL dummy 큐의 stereo·소비·숨김 시 제거·모드 전환 probe 통과(peak6251).
- 실제 Chromium의 파일 입력으로 합성 MP3 선택·사용자 소리 켜기·좌우2048샘플 비영/서로 다른 출력·loop·음량·숨김/복귀·끄기/효과음 gain 유지·손상 MP3 오류/재시도 표시·해제/원본 복귀를 확인했다. 게임 상태 동일, 업로드 요청0, 브라우저 예외0.
- Node 수명주기 검사: autoplay 차단/재시도·취소/늦은 play 완료·세대·독립 gain·오래된 파일 사건 무시·URL 해제 통과.
- 전체 회귀/패키지/CI와 최종 self-review는 이어서 진행한다. 실제 사람의 새 MP3 청취 판정은 아직 없다.

![사용자 MP3 반복 재생과 독립 음량 UI, 원본 없는 합성 테스트 장면](issue-202_local_mp3_web.png)

스크린샷은 프로젝트가 직접 합성한440/660Hz MP3와 연습 장면이다. 원본 게임/K-pop 녹음이 아니다. fixture의 생성 명령·해시·크기는 [기록](../../assets/manifests/local_music_fixture.json)에 고정했다. 실제 원본 및 HD 게임 재생·배포 인수는 최종 검증에서 별도로 기록한다.
