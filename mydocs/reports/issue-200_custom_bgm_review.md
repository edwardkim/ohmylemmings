# #200 사용자 K-pop BGM 교체 기술 검토

이슈: https://github.com/edwardkim/ohmylemmings/issues/200

2026-10-24 검토 완료. **Web/Native 모두 외부 음악으로 BGM을 교체할 수 있다. 첫 구현은 사용자가 적법하게 확보한 DRM 없는 MP3를 직접 선택하는 방식으로 권고한다.** 상용 K-pop 파일을 배포본에 포함하는 선택은 곡별 권한 확보와 별개다. 이번 작업은 소스/공식 문서 검토이며 재생 기능 구현이나 실측 검증을 완료한 것은 아니다.

## 현행 구조와 필요한 변경

- [게임 사운드 사건](../../crates/lemmings-game/src/sound.rs)은 `Music`/`Effect`/`Stop`, 음악 시작 상태와 레벨 세대를 보낸다. 음악 선택·게임 사건·시뮬레이션은 그대로 두고 출력 어댑터가 BGM 모드를 선택할 수 있다.
- [공통 AudioOutput](../../crates/lemmings-game/src/audio.rs)의 `synchronize`는 활성화/새 세대에 현재 곡을 다시 시작하며, `render`는 음악과 효과음을 합성한 **하나의 mono i16 PCM**에 단일 음량을 적용한다. 현재 BGM 단독 음소거·외부 음원 디코더·재생 목록은 없다. ADLIB.DAT를 MP3로 바꿔 넣어도 재생되지 않는다.
- [AdLib 드라이버](../../crates/lemmings-resources/src/adlib_stream.rs)는 음악/효과음의 OPL 채널 풀과 초기화/정지 처리를 공유한다. 음악 명령을 출력 계층에서 제어하고 새 effects-only 상태를 검증해야 한다. 전체 PCM을 음소거하면 효과음도 사라지고, 완성 PCM에서 원본 음악만 빼낼 수는 없다. 음악 제외 시 효과음 채널 할당/초기화가 달라질 수 있으므로 파형 동일성은 아직 주장하지 않는다.
- [WebAudio 브리지](../../apps/web/src/audio.rs)가 단일 사건 큐를 소비하고 [JS AudioHost](../../apps/web/www/audio.js)는 작은 mono 버퍼를 AudioContext에 예약한다. [SDL 어댑터](../../apps/desktop/src/audio.rs)는 현재 mono SDL AudioQueue를 쓴다. 스테레오 음악을 유지하려면 Native 믹서/장치 채널 확장이 필요하다.

제안 구조는 `원본 BGM / 내 음악 / BGM 끄기`와 독립된 효과음 음량이다. 기본 원본 모드는 기존 OPL/PCM 경로를 보존하고, 내 음악 모드는 검증된 효과음 전용 OPL 출력과 별도의 음악 재생 경로를 섞는다. 큐는 계속 한 소비자가 처리하고 외부 음악 제어용 상태를 전달한다. 시뮬레이션/core/no_std 자원 계층에 파일/네트워크/디코더 의존성을 넣지 않는다.

## 공급 방식 비교

| 방식 | 기술 판단 | 권고 |
|---|---|---|
| 사용자가 로컬 MP3 선택 | Web File 입력/Native 파일 경로·선택기로 가능. 프로젝트 서버 업로드 없이 설계 가능 | 첫 구현에 적합. 사용자가 좋아하는 곡을 바꾸며 배포 용량을 늘리지 않는다 |
| 사용 허락을 받은 음악 팩 | 정적 파일/manifest 또는 Native 파일로 가능 | 실제 게임 사용·배포 권한을 확보한 뒤 선택 기능으로 검토 |
| 최신 상용 K-pop을 릴리즈·Pages에 포함 | 디코더·전송은 가능 | 음원별 권한/배포 범위를 먼저 확인해야 하며 자동 교체 대상으로 삼지 않는다 |
| Spotify API로 게임 BGM 연결 | 현재 공식 개발자 정책 III.2·6·7은 게임, 영상 동기화, 다른 오디오와 혼합을 금지 | 이 프로젝트의 일반 구현 경로에서 제외 |

곡/음원 사용 권한은 코드의 MIT 라이선스로 부여되지 않는다. WIPO는 게임 등의 영상물 음악 결합에 동기화 라이선스를 설명하고 저작자/퍼블리셔와 녹음 권리자의 권한을 구별한다. 공개 패키지·웹 전송을 하려면 대상 곡에 대한 작곡/작사·녹음 사용 및 복제·배포/전송 등 필요한 범위를 권리자와 확인해야 한다. 무료 토이프로젝트라는 이유만으로 포함 가능하다고 판단하지 않는다. 로컬 파일 기능도 모든 파일 이용에 대한 포괄적 허락은 아니다. 스트리밍 서비스의 앱 전용/DRM 다운로드는 일반 MP3 파일 입력 대상이 아니다. [WIPO 음악 권리 안내](https://www.wipo.int/en/web/music/) · [Spotify 개발자 정책](https://developer.spotify.com/policy).

## Web 구현 권고

1. 게임 데이터 파일 입력과 별도의 `내 음악 선택`으로 파일을 받고 `URL.createObjectURL(file)`을 HTMLAudioElement에 연결한다. 업로드·서비스 캐시 저장을 하지 않고 교체/해제 시 object URL을 정리한다. 기본 구현에서는 새로고침 후 재선택한다. 파일 이름은 텍스트로 표시한다. [File API 공식 문서](https://developer.mozilla.org/en-US/docs/Web/API/File_API/Using_files_from_web_applications).
2. HTMLAudioElement를 기존 AudioContext의 MediaElementAudioSourceNode와 음악용 GainNode에 연결한다. 효과음용 gain·master gain을 구분한다. 길이가 긴 곡은 브라우저의 미디어 재생/버퍼링 경로를 이용하고, 전체 파일을 WASM heap에 복사하거나 전곡을 AudioBuffer로 디코딩하지 않는다. `decodeAudioData`는 완전한 파일을 요구해 부분 다운로드만으로 시작하는 경로에 맞지 않는다. [미디어 소스 노드](https://developer.mozilla.org/en-US/docs/Web/API/AudioContext/createMediaElementSource) · [decodeAudioData](https://developer.mozilla.org/en-US/docs/Web/API/BaseAudioContext/decodeAudioData).
3. 사용자의 소리 켜기/재생 버튼에서 context.resume와 media.play를 요청하고 거절을 UI에 표시한다. 기존 효과음이 활성화됐다고 새 미디어 요소의 자동 재생까지 보장하지 않는다. MP3를 첫 인수 형식으로 하고 실제 Windows11 Chromium/지원 브라우저에서 확인한다. AAC/OGG/FLAC 등은 브라우저/OS 차이를 검증한 뒤 추가한다. [자동 재생 정책 안내](https://developer.mozilla.org/en-US/docs/Web/Media/Guides/Autoplay).
4. 첫 버전은 한 곡 반복과 원본 복귀를 제공하고 재생 목록·레벨별 곡 대응은 후속으로 분리한다. 파일/디코드 실패 시 게임과 효과음을 유지하고 원본 BGM 복귀 또는 재선택을 제공한다. 기존 원본 데이터/음향 오류 처리와도 구분한다.

GitHub Pages에서도 파일 선택과 브라우저 재생이 가능하므로 사용자 음원 저장용 서버가 필요하지 않다는 설계 판단이다. 다만 실제 구현/배포 브라우저 인수는 아직 수행하지 않았다.

## Native 구현 권고

호스트 파일 입력 → 디코더 작업자 → 상한이 있는 스테레오 PCM ring → SDL 출력 믹서를 권고한다. 원본 효과음 mono를 두 채널에 보내고 음악을 장치 표본율로 리샘플링하며 독립 gain/헤드룸·포화 처리를 적용한다. 디스크 읽기·디코딩·할당/대기를 실시간 장치 콜백에 넣지 않는다. 두 SDL 장치를 따로 열어 음악과 효과음의 시계가 어긋나게 하기보다 하나의 출력 흐름을 사용한다.

디코더 후보는 Pure Rust Symphonia다. 공식 문서는 MP3/FLAC/Vorbis/AAC 등의 지원과 MP3 기능의 명시적 활성화를 설명한다. **아직 도입/버전 확정하지 않았다.** 구현 시 Rust1.93.1/MSRV·선택 codec·라이선스/고지·의존성 고정·Windows11/macOS13+/Linux 크로스 빌드·새 설치 시험을 확인해야 한다. SDL2_mixer를 추가하는 대안은 별도 바이너리/codec 패키징 비용이 있어 이번 검토의 우선안으로 삼지 않았다. [Symphonia 공식 README](https://github.com/pdeljanov/Symphonia/blob/main/symphonia/README.md).

## 다운로드·메모리·진행률 UX

- 로컬 파일 방식은 음악 다운로드가 없어 초기 게임 리소스를 늘리지 않는다. `음악 준비 중 / 재생 중 / 재생을 시작하려면 클릭 / 지원하지 않는 파일 / 파일 다시 선택`을 표시하고 현재 곡·반복·BGM/효과음 음량을 제공한다.
- 별도 사용 허락을 받은 호스팅 음악을 추가한다면 현재 곡만 준비하고 HD 필수 다운로드에 우선순위를 양보한다. 음악 준비 실패로 게임을 막지 않고 다음 곡 전부를 미리 받지 않는다. 현재 HD gzip loader를 MP3 디코더로 재사용하지 않는다.
- HTMLMediaElement의 `progress`/`buffered`는 미디어 버퍼 구간을 제공한다. 재생 위치에서 이어지는 버퍼 길이(초)와 로딩/대기를 표시할 수 있지만 이를 정확한 수신 바이트 백분율로 표시하면 안 된다. 정확한 바이트 진행률이 필요하면 별도의 계측 다운로드 설계가 필요하다. Range 응답·CORS·codec/container·회선 경쟁은 실제 호스트에서 검증해야 하며 GitHub Pages에서 무조건 스트리밍된다고 확정하지 않는다. 기존 게임 필수 리소스의 실제 바이트 진행률은 유지한다. [HTMLMediaElement 공식 문서](https://developer.mozilla.org/en-US/docs/Web/API/HTMLMediaElement).
- 산술 예시: 3분 곡은192kbps에서 약4.32MB(메타데이터 제외), 20곡은 약86.4MB다. 같은 곡의44.1kHz·스테레오·Float32 전곡 PCM은 `180×44100×2×4=63,504,000`바이트(약60.6MiB)다. 이는 실측 성능이 아니며 긴 플레이리스트 전곡 선디코딩을 피할 근거다. 음악이 추가되어도 HD 업스케일과 시뮬레이션 성능 목표를 변경하지 않는다.

## 재생 수명주기와 구현 시 인수

첫 구현은 원본 동작에 맞춰 음악 시작 사건에서 재생, 결과/숨김에서 정지, 재시작/다음 레벨/복귀에서 선택한 곡을 처음부터 시작하는 안이다. 게임 Pause 중 음악은 계속한다. 사용자 음악은 한 곡 반복으로 시작하며 실제 제품 동작은 구현 계획에서 확정한다. 모드 변경은 세대/취소 토큰으로 오래된 비동기 완료를 무시하고 이전 음악/OPL 잔향·큐를 정리한다. UI 설정은 gameplay 입력/결정론 틱에 넣지 않는다.

구현 시 TDD/인수 항목:

- 원본 모드 기존 PCM/사운드 사건/게임 상태 회귀, 내 음악 모드의 원본 BGM 무음 및 효과음 지속·Stop/새 세대·복귀·즉시 모드 변경.
- 원본 전체18효과음/특수 곡 전환·효과음 중첩·클리핑, 합성 음악 파일로 stereo/rate/loop/end/고장 디코더·상한 버퍼·긴 파일/메타데이터 검증. 상용 곡을 테스트 fixture로 커밋하지 않는다.
- 실제 사용자 클릭·소리 거절/재시도·숨김/결과/Restart·일시정지·로딩 표시·스크린리더/키보드·파일 미업로드와 object URL 정리.
- Windows11/macOS/Linux 새 패키지·공개 HTTPS Web, 저사양 환경의 UI 응답/큐 한도, Native/Web 청취와 실제 효과음 동시 재생.

검토에서는 소스/공식 API·정책을 확인하고 문서 링크/변경 범위를 검증했다. 새 음악을 재생하거나 디코더 성능·전 기기 호환성을 시험하지 않았다. 검토 이슈는 문서 병합으로 완료하며, 제품 구현은 메인테이너가 방향을 선택한 뒤 별도 이슈로 진행한다. #165는 완료 상태다.
