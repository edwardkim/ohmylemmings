# Linux 고품질 그래픽·GPU 확장 조사

담당 [#233](https://github.com/edwardkim/ohmylemmings/issues/233). 조사 기준 main `8d44533c8caeb23c20e49c67795144109b45472f`, 2026-10-25. 관련 [HD 설계](../plans/issue-137_2d_remaster.md), [배포 최적화](issue-226_package_size.md), [현재 릴리즈](issue-230_release_v033.md).

## 결론과 권고

**Linux에서 더 높은 품질의 2D 자산과 GPU 효과를 사용하는 버전으로 확장할 수 있다.** 운영체제 변경이나 게임 규칙 재작성은 필요하지 않다. 현재 HD도 GPU 가속을 우선하는 SDL2 렌더러를 사용하므로, 단순히 GPU 스위치를 켜는 작업으로 목표가 완성되지는 않는다. 화질을 결정하는 제작 자산, GPU에서 수행할 작업, 메모리·파일 전달 정책을 함께 설계해야 한다.

권고는 **1080p 기본 자산 유지 + 선택형 고품질 자산 + 별도 GPU 렌더러의 단계적 검증**이다. 먼저 한 테마의 지형·지물·캐릭터를 일관된 품질로 제작하고 SDL2 경로에서 효과를 확인한다. 동적 조명·노멀맵·GPU 지형 합성까지 요구한다면 Linux 우선 wgpu/Vulkan 어댑터를 별도로 구현하는 것이 유력하다. 엔진 전체를 교체하는 작업보다 현재 공통 세션/표시 자료를 재사용하는 편이 범위가 작다. 이는 소스 구조와 공식 API 지원에 근거한 설계 판단이며 새 렌더러의 성능 검증 결과는 아니다.

이번 이슈는 조사만 수행한다. 제품 코드/자산/의존성·공개 릴리즈 변경 및 새 구현 이슈 착수는 포함하지 않는다.

## 현재 구현에서 확인한 것

| 구간 | 현재 상태 | 확장 시 의미 |
|---|---|---|
| `render.rs` | 320×200 원본 프레임을 CPU에서 생성 | IDE에서 연 파일은 원본 표시 경로다. 전체 HD 경로는 별도다. |
| `hd/layout.rs` | 1920×1080, 게임 영역1920×960, 논리 배율6 | 출력 창을4K로 늘리는 것과4K 세부 자산을 쓰는 것은 별개다. |
| `apps/desktop/src/main.rs:975` | HD는 기본 CanvasBuilder, 원본은 명시적 software | rust-sdl2는 기본적으로 하드웨어 가속을 우선한다. 실제 선택은 드라이버/환경에 따라 다르다. |
| `apps/desktop/src/hd.rs:70` | RGBA32·선형 필터·revision 캐시, 변경 시 texture를 새로 생성해 전체 업로드 | 변경하지 않은 자산은 재사용한다. 잦은 지형/객체 변경의 재생성과 업로드는 개선 후보다. |
| `hd/scene.rs:441` | 지형 mask/bricks를 대조하고 변경 주변32×32 논리 셀 청크만 재합성 | 전체 HD 세계를 매번 그리는 구조는 아니다. 다만 mask 대조 및 변경 청크의 고해상도 질감 생성은 CPU 작업이다. |
| `hd/surface.rs` | CPU의 질감 혼합·경계 명암/윤곽, 선택 재질 캐시 최대1024×1024 | 더 큰 원본 PNG만 넣어도 캐시/표시 규격이 자동으로 세부 질감을 보존하지 않는다. |
| `hd/assets.rs`·`hd/catalog.rs` | sRGB로 인코딩된 straight-alpha RGBA8, 4096 최대 치수, 활성128MiB/32페이지, 전체512MiB/128페이지 | 현재 고정 포맷에4K 자산·노멀맵·압축 텍스처를 추가하면 한도/파서/버전/보안 검증을 함께 바꿔야 한다. |

SDL2 기본 가속 우선 선택은 [현재 사용 중인 rust-sdl2 0.38.0 문서](https://docs.rs/sdl2/0.38.0/sdl2/render/struct.CanvasBuilder.html), renderer 선택은 [SDL_CreateRenderer](https://wiki.libsdl.org/SDL2/SDL_CreateRenderer)와 [SDL_HINT_RENDER_DRIVER](https://wiki.libsdl.org/SDL2/SDL_HINT_RENDER_DRIVER)를 대조했다. SDL2 renderer에 `vulkan` 이름을 지정하는 것만으로 Vulkan 구현이 생기지는 않는다.

현재 API는 Fill/Blit/Clip 중심이며 노멀맵·실시간 조명·HDR·사용자 셰이더용 제품 경로는 없다. 그러한 기능을 단순 설정으로 활성화할 수는 없다. 현재 색상 데이터가 sRGB라고 선언된 것과, 선형 공간 합성/출력 전송이 올바르게 구성된 것은 각각 검증해야 한다.

## 이 Linux 호스트의 짧은 기능 확인

PCI 장치는 Intel Alder Lake-N UHD Graphics, 커널 드라이버는 i915다. 별도의 SDL offscreen64×64 컨텍스트에서 다음을 확인했다.

- SDL2 라이브러리 버전: pkg-config2.32.72.
- renderer: `opengl`, flags10(ACCELERATED + TARGETTEXTURE).
- GL vendor/renderer: `Intel` / `Mesa Intel(R) Graphics (ADL-N)`.
- GL version: `4.6 (Compatibility Profile) Mesa26.2.2-arch1.1`.
- 보고된 최대 texture: 16384×16384.

이는 하드웨어 OpenGL 컨텍스트 생성 확인이다. 창/Wayland 표시, Vulkan 장치 생성, 실제 게임 프레임 시간·전용 VRAM·4K 처리량은 측정하지 않았다. Intel Vulkan ICD 파일이 설치돼 있지만 Vulkan 가동/성능을 입증하지 않는다. max texture 수치는 안전한 자산 예산을 뜻하지 않는다. 진단 스크립트/출력은 ignored `assets/local/issue-233-graphics/`에 두었고 드라이버나 시스템 설정은 변경하지 않았다. 이전 지시에 따라 이 저사양 호스트에서60fps 맞추기 반복 측정은 재개하지 않았다.

## 무엇을 더 좋게 만들 수 있는가

1. **그림 자체의 완성도**: 지형/기둥/시설/캐릭터의 선 굵기·붓질·팔레트·광원 방향을 통일하고, 암석의 중간 크기 형태와 미세 질감을 따로 제작한다. 기존의 수직벽 부조화를 단순 사진 텍스처로 덮으면 양식 차이가 커질 수 있다. 타일 반복·가장자리 이음새와 작은 캐릭터의 식별성을 먼저 인수한다.
2. **입체감과 효과**: 지형/지물에 base color와 normal/roughness/emissive 같은 선택적 재질 자료를 만들어 부드러운 방향광·횃불·물/불·분진을 표현할 수 있다. 노멀맵이 없는 자산에서 GPU만으로 정확한 표면 정보가 생기지는 않는다. 굴착 단면의 노멀/가림은 읽기 전용 논리 mask로 생성할 수 있다.
3. **더 큰 출력**: 1440p/4K 및 고밀도 디스플레이의 실제 drawable 크기에 맞춰 렌더 타깃과 글꼴/레이아웃을 구성한다. 기존 코드의6배 상수·1920/960 clip·좌표/입력 변환을 함께 정리해야 한다. UI 포인터 좌표에 배율을 중복 적용하지 않는다.
4. **원본 규칙 보존**: 고품질 그림은 현재 충돌 mask·steel·one-way·출입구/함정 기준점을 따른다. 조명·분진·표시 보간은 게임 상태를 바꾸지 않는다. 원본 지형 형상의 변화는 별도 게임 설계 변경으로 취급한다.

재질 원천 후보로 [Poly Haven](https://polyhaven.com/license)은 자사 자산을CC0로 제공한다고 명시한다. 암석/흙/금속의 고해상도 재질을 프로젝트 화풍에 맞게 가공하는 출발점이 될 수 있다. 특정 자산을 선택/다운로드하거나 현재 게임에 적용하지는 않았다. 신규 재질의 라이선스 기록은 기존 provenance 정책을 따르며, 이것만으로 원본 캐릭터/레벨 데이터 전체를 대체했다고 판단하지 않는다.

## 구현 경로 비교

| 경로 | 가능한 개선 | 작업 범위·판단 |
|---|---|---|
| SDL2 유지, 자산/캐시 개선 | 더 좋은2D 자산, 고밀도 표시, texture 재사용/부분 갱신, 가속 경로 진단 | 가장 빠른1단계. 기존 게임을 비교 기준으로 쓰기 좋다. 사용자 셰이더 효과에는 별도 확장이 필요하다. |
| SDL2 창/입력 유지 + wgpu 어댑터 | Vulkan 기반 GPU 합성, 노멀/광원/색 공간·필터·입자 제어 | 장기 고품질 모드의 우선 후보. SDL Renderer와 wgpu가 같은 창을 동시에 소유하지 않도록 호스트 분기·surface 생명주기를 설계해야 한다. |
| SDL3 + GPU API | Vulkan/Metal/D3D12 렌더·compute | 가능하지만 SDL2 이벤트/오디오/패키징 이행까지 범위가 커진다. 그래픽 조사만으로 전체 SDL 이행을 권하지 않는다. |
| Vulkan 직접 구현 | 세밀한 GPU 제어 | 장치·메모리·동기화·swapchain 유지 비용이 크다. 이2D 프로젝트에서는 wgpu 대비 우선할 근거가 현재 없다. |

[wgpu backend 문서](https://docs.rs/wgpu/30.0.1/wgpu/struct.Backends.html)는 Linux Vulkan 및 GL을 제공하며 GL은2순위 지원 경로로 설명한다. 따라서 Linux Vulkan 우선, 실패 시 검증된 기존 SDL2 경로로 돌아가는 방안을 권한다. 이 fallback은 제품이 직접 구현해야 하며 라이브러리가 기존 SDL 화면을 자동 복원하는 것은 아니다. 장래 다른 플랫폼 확장도 가능하지만 이번 Linux 조사를 WebGPU 전환 결정으로 확대하지 않는다.

확인한 [wgpu30.0.1의 공식 Cargo manifest](https://github.com/gfx-rs/wgpu/blob/v30.0.1/Cargo.toml)는 Rust1.93을 요구하므로 현재1.93.1과 버전 조건은 맞는다. 실제 의존성 해석/플랫폼 컴파일은 수행하지 않았으며 채택 시 전이 의존·Rust/플랫폼·라이선스·빌드 크기를 별도 검증해야 한다. 최신 번호를 이유로 의존성을 지금 추가하지 않았다.

[SDL3 GPU](https://wiki.libsdl.org/SDL3/CategoryGPU)는 별도 shader/pipeline을 사용하는 API이고 [SDL_CreateGPUDevice](https://wiki.libsdl.org/SDL3/SDL_CreateGPUDevice)는 Vulkan 등 backend를 제공한다. 현재 SDL2 Canvas API의 옵션 변경과는 다른 이행 작업이다.

## 고품질 자산과 용량의 균형

기본은 현재 HD를 두고 High/Ultra를 선택 다운로드하는 구성이 적합하다. Linux라는 이유만으로 모든 테마·고해상도 원본을 한꺼번에 상주시킬 필요는 없다. 테마별 선택 로딩과 메모리 상한을 이어가고, 고품질 팩의 버전·호환 manifest·해시·실패 복구를 별도로 둔다. 새 팩에 필요한 대응 소스/고지는 제공하되 모든 플랫폼 실행 ZIP에 중복 내장하지 않는다.

단순 계산으로1920×1080 RGBA8 단일 버퍼는7.91MiB,3840×2160은31.64MiB다. 같은 구성에서 가로·세로2배는 픽셀/바이트4배이며, base color에 같은 크기의 normal map을 추가하면 그 채널 저장량도 더해진다. mipmap·중간 렌더 타깃·CPU 복사·전환 중 두 테마가 겹치는 peak는 별도다. 이 수치는 총VRAM 예측이나 성능 벤치마크가 아니다. 지금의128MiB 입력 한도를 무작정 올리는 방식은 권하지 않는다.

[KTX2](https://registry.khronos.org/KTX/specs/2.0/ktxspec.v2.html)는 mip 수준과 GPU 텍스처 배포·supercompression을 지원하는 후보 포맷이다. GPU 압축은 다운로드 gzip과 다른 문제이며, 지원되는 BC 계열 등을 어댑터 기능으로 확인해야 한다. [wgpu 압축 feature 문서](https://docs.rs/wgpu/30.0.1/wgpu/struct.Features.html#associatedconstant.TEXTURE_COMPRESSION_BC)는 대역폭 절감에 품질 손실이 수반됨을 명시한다. 따라서 큰 정적 지형 재질부터 비교하고 글꼴/작은 캐릭터/경계 alpha에는 lossless 대안을 유지한다. atlas mip 생성에는 프레임별 padding/색 번짐 처리가 필요하다. CPU에서 계속 바뀌는 지형 mask/단면을 압축 포맷으로 매번 재인코딩하는 것은 피하고 정적 재질과 동적 mask를 분리한다.

## 후속 절차 제안과 인수 기준

1. **대표1테마 인수**: Fun02의 기둥·암석/흙, 입출구와 캐릭터를 대상으로1080p/4K 정지 장면을 제작한다. 부자연스러운 직선·타일 반복·대상 식별을 비교한다. 실제 그림 품질이 좋아지는지 먼저 판정한다.
2. **현재 SDL 경로 정리**: renderer/실제GPU·출력 크기·texture 업로드 바이트와 횟수를 기록한다. 같은 크기 texture는 revision마다 생성하는 대신 재사용/부분 갱신을 검토한다. 자주 갱신되는 texture는 [SDL_UpdateTexture 지침](https://wiki.libsdl.org/SDL2/SDL_UpdateTexture)에 따라 streaming/lock도 비교한다. VSync는 표시 타이밍 선택이며 단독 속도 향상을 약속하지 않는다.
3. **한 레벨 GPU 시제품**: 기존 LevelRun/snapshot과 논리mask는 유지하고 정적 재질·mask·배치 정보를 GPU에 전달한다. CPU에서 최종 HD RGBA를 만들어 업로드하는 단계를 점차 shader로 옮긴다. 현재2D draw list를 받아 그리는1차 버전과 GPU 지형 합성 버전을 구분한다. 첫 버전만으로 CPU 지형 합성 비용이 사라진다고 판단하지 않는다.
4. **고성능 Linux 실측**: X11/Wayland, 통합/외장GPU에서1080p·1440p·4K의 CPU/GPU 프레임 시간p50/p95/p99·업로드량·상주/peak메모리·시작/테마 전환을 비교한다. 목표 예시는60Hz용16.67ms 예산이며 실제 지원등급은 결과로 결정한다. 입력 지연·읽기 쉬움·가속 장치 부재/손상 팩 fallback도 확인한다.
5. **정식화 판단**: 기존 저장 입력과 전120레벨 상태/충돌/결과 동일성, 굴착/계단/폭발·steel/one-way/시설 가림을 검증한다. 조명 변경은 이전 이미지의 완전 일치 대신 상태 동일성과 새 승인 이미지로 검증한다. 게임 코어/자원 parser에wgpu·SDL·파일I/O 타입을 넣지 않는다. 고품질 팩은 선택 배포하며 #113의 미인수 결과를 이번 기능 확인으로 통과 처리하지 않는다.

현 시점 권고는 **대표 테마의 고품질 자산 시안과 SDL 업로드 개선을 먼저 구체화하고, 동적 재질/조명 요구가 확정되면 wgpu/Vulkan 시제품으로 넘어가는 것**이다. Linux 하드웨어 활용 가능성은 확인했지만 최고의 화질·4K60fps의 최종 인수는 후속 제작과 실측이 필요하다.
