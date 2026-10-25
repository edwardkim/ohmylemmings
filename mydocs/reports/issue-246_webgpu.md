# WebGPU 구현·v0.3.4 배포 검증

담당 [#246](https://github.com/edwardkim/ohmylemmings/issues/246), [계획](../plans/issue-246_webgpu.md), [릴리즈 노트](../releases/issue-246_v0.3.4.md).

## 구현과 로컬 인수

WebGPU adapter/device/비동기 파이프라인을 준비하여 원본 RGB·HD draw-list를 직접 표시한다. premultiplied RGBA·선형 필터·draw 순서·960px 게임 영역 clip을 지원하며 revision이 같은 텍스처는 재업로드하지 않는다. 원본은 nearest 필터로 표시한다. 입력 canvas를 유지하고 같은 위치의 GPU 표시 canvas를 사용하므로 장치 손실에도 포커스/입력/게임 상태를 유지한다. 미지원·비보안 origin·실패·3초 시간 초과·GPU 오류/loss에는 Canvas2D로 복구한다. 실제 renderer와 사유는 화면/F2에 기록한다.

- Node 계약 테스트: API 없음/비보안/명시 Canvas2D/초기화 실패·시간 초과와 늦은 자원 해제/초기화 중 loss/실행 중 loss/동기 draw 실패·캐시 revision/해제/명령 순서·clip·premultiplied alpha 통과. 구현 모듈이 없을 때 실패를 확인한 뒤 구현했다.
- Python 전체340개·배포/Pages/Web 빌드 관련34개 테스트 통과. 최초 전체 검사에서 소스 pin 불일치를 발견하여 이전 pin과 역사 판정을 보존하면서 현재 source-index를 갱신했다.
- WASM 빌드와 `make check-wasm` 통과. 실제 WASM/Native/Canvas2D 회귀33사례×2회,11,960상태·6,877프레임/readback 통과. HD13장면·7,593상태·821프레임·337캐릭터 프레임과 입력/음향/지형 일치를 포함한다.
- Chromium152/Intel gen-12lp, `isFallbackAdapter=false`에서 실제 WebGPU 원본 RGB 픽셀 전수 일치. 합성 blend/crop/negative destination/clip/revision, 굴착·폭발·승리/실패 HD 화면, 게임/음향 상태 유지와 device.destroy에 따른 복구 통과.
- 실제 제품 DOM에서 Fun01→02→22 HD 전환, 원본/HD 선택, 입력 canvas/표시 위치 일치, Pause 상태 device loss·복귀 입력·원본 전환·재시작 통과. 초기 GPU 선택과 fallback 안내도 확인했다.

HD는 Canvas2D와 완전 동일한 픽셀을 주장하지 않는다. 합성 확대 이미지의 필터 가중치 차이는 최대12/255였고, HD 장면 RGBA 평균 차이는0.0144~0.1749/255·최대15/255였다. 합성 비필터/clip 영역은2/255 이내로 검사한다. 필터 영역은16/255 이내로 제한했다. 전체120레벨 재완주나 FPS 성능 측정은 이번 검증에 포함하지 않는다.

Linux headless 실제 GPU 검사에는 `--enable-unsafe-webgpu --enable-features=Vulkan --use-angle=vulkan --disable-vulkan-surface`를 임시 프로필에만 사용했다. 일반 사용자 브라우저 설정은 변경하지 않았다. GPU canvas를 Canvas2D로 직접 읽으면 검증용 캡처가 투명하게 반환되는 문제가 있어, 실제 GPU texture→mapAsync readback으로 검증했다. 처음 두 옵션만 사용한 시도에서는 Chrome SharedImage 오류도 발생했다. 실패한 시도는 통과 증거에서 제외하고 보존했다.

![WebGPU 굴착 장면](issue-246_webgpu.png)

개인 원시 증거: `assets/local/issue-246/`. 최종 GPU: `gpu-original-final/summary.json`. Canvas 회귀: `web-regression/summary.json`. 기존 사용자 브라우저/서버와 원본 MP3는 유지했다.

## v0.3.4 게시 완료

[정식 릴리즈](https://github.com/edwardkim/ohmylemmings/releases/tag/v0.3.4)와 [Pages 웹 플레이](https://edwardkim.github.io/ohmylemmings/)를 게시했다. 네 실행 패키지·공통 대응 소스·PLAY.md·개별/전체 체크섬·RELEASE-MANIFEST의12파일을 제공한다. Cargo workspace/lock9개와 한·영 안내/노트는0.3.4이다.

| 항목 | 값 |
|---|---|
| 구현 PR | [#247](https://github.com/edwardkim/ohmylemmings/pull/247) |
| PR head | `d10c307ea1d48a7455d5f38aae5ff7d74ef97a0c` |
| 실제 CI checkout | `93b75effaa420a72b435f3846dc913d2cae8d09b` |
| main/tag commit | `7969fc9ed8f07c9dd266a81d6889d5536a559957` |
| 공통 source tree | `d371bb618c6095f11cd051d11cff599f5cb6653e` |
| annotated tag object | `d7369c66d682255d6f4d57f2694280f5df49136c` |
| 플랫폼 CI7개 | [40168898131](https://github.com/edwardkim/ohmylemmings/actions/runs/40168898131), 전부 SUCCESS |
| 태그 전 prepare-only | [40170111497](https://github.com/edwardkim/ohmylemmings/actions/runs/40170111497), SUCCESS |
| 정식 게시·Pages | [40170406244](https://github.com/edwardkim/ohmylemmings/actions/runs/40170406244), publish/Pages SUCCESS |

중복 feature push CI만 취소했다. 병합 [main CI40170110521](https://github.com/edwardkim/ohmylemmings/actions/runs/40170110521)도7개 검사 전부 SUCCESS였으며, 배포에는 먼저 통과한 PR CI의 동일 tree 패키지를 사용했다. 사전 검증과 정식 RELEASE-MANIFEST의 CI/소스/파일 해시가 일치한다.

## 공개 인수

- 초안 업로드 파일 재다운로드와 공개 후12파일의 인증 없는 다운로드 바이트 검증을 Release workflow에서 통과했다.
- Pages0.3.4의 commit/tree/Web ZIP SHA가 RELEASE-MANIFEST와 일치한다. 공개 실행 파일109개 전체 크기·SHA-256, WASM MIME과 대응 소스 URL/크기/해시/공개 HEAD 접근을 별도로 검증했다. 배포 메타데이터까지111파일이다.
- 공개 HTTPS 사이트를 실제 Wayland Chromium 창에서 WebGPU로 열어 원본/HD 선택·8기술 키·Pause/Resume·1920×1080 전체화면과 입력/표시 위치 일치를 확인했다. Fun01에서60틱 Digger 배정 후660틱/10구출 Won, 다음 Fun02로 점진적 로딩·이동을 확인했다. 이는 저장된 해법의 빠른 명시적 틱 검증이며 실시간 FPS 측정이나 전체120레벨 재완주가 아니다.
- 공개 Fun02 Pause 상태에서 GPU device를 파괴하여 상태 보존·Canvas2D 전환·이후 입력 복귀를 확인했다. 브라우저 예외0이었다.
- 공개 `?renderer=canvas2d` 시작에서도8기술 키·Pause/Resume·1920×1080 전체화면·Fun01 10구출 Won·다음 Fun02 이동을 통과했다. 이 경로도 브라우저 예외0이었다.
- 기존6개 릴리즈의 본문·게시 시각·asset ID/이름/크기/digest/시각과 기존 태그가 보존됨을 확인했다. 다운로드 횟수 같은 변동성 통계는 비교하지 않았다.
- Linux headless의 GPU framebuffer readback은 맞지만 전체 페이지 캡처는 검게 반환되는 환경 제한이 있었다. 같은 코드·GPU·자산을 일반 Wayland 창에서 실행한 실제 표시/캡처는 정상이다. 이 구분을 확인한 후 게시했고, 제품 렌더러를 검증용 복사 경로로 변경하지 않았다.

개인 공개 검증 자료: `assets/local/issue-246/public-verification.json`, `public-webgpu/summary.json`, `public-webgpu/public-product.png`, `public-canvas2d/summary.json`. 공개 CI·Release workflow와 release-publication artifact에도 계보가 남는다. 임시 QA 브라우저는 종료했으며 사용자19222/8080 환경은 유지했다.

## 게시 파일

| 파일 | 바이트 | SHA-256 |
|---|---:|---|

| `ohmylemmings-0.3.4-linux-x86_64.tar.gz` | 98,441,503 | `a2f175b096d2b62201060c2332af2f847ca849b870c964614d5573c8f343c7d6` |
| `ohmylemmings-0.3.4-macos-universal2.zip` | 100,820,836 | `981e93e1147c618d5322b01fb33b2001ec8f7b2f23da46a536fb49d274b51e7f` |
| `ohmylemmings-0.3.4-source.tar.gz` | 86,015,002 | `8909286f8ae5135b1d98a88611debaca81363395de0a660f83e0b57f03dbea56` |
| `ohmylemmings-0.3.4-web.zip` | 97,570,060 | `05efca14c73cfe87e77ceeadf6f7d5dfa3f2f1a1f5875cab60ae474e57beff9e` |
| `ohmylemmings-0.3.4-windows-x86_64.zip` | 98,881,393 | `dc9ef2ab089fc5ab9d424b7ec76b9538ef407d6b926aaac42bf1a4bd18588052` |

Web ZIP은0.3.3의97,565,086바이트에서97,570,060바이트로4,974바이트 증가했다. 새 Rust GPU 의존성이나 추가 그래픽 다운로드는 없다. 초기 레벨의 점진적 다운로드 정책은 유지한다.
