# WebGPU 웹 렌더러

이슈: [#246](https://github.com/edwardkim/ohmylemmings/issues/246)

## 범위와 설계

- Rust/WASM의 시뮬레이션·HD draw-list 계약은 유지한다. 브라우저 어댑터에서 직접 WebGPU로 화면을 그린다.
- 원본 RGB 프레임과 HD의 사각형·텍스처·게임 영역 clip을 지원한다. 변경된 revision만 업로드하고 레벨 교체 때 GPU 자원을 해제한다.
- 보안 컨텍스트/API/adapter/device/파이프라인 준비를 확인한다. 초기화 제한 시간을 두고 실패하면 Canvas2D로 시작한다.
- 기존 입력 canvas를 유지하고 같은 위치의 표시 전용 GPU canvas를 사용하여 fallback 때 키보드·포인터·전체화면 연결을 보존한다. GPU→Canvas2D 프레임 복사는 하지 않는다.
- device loss·비동기 GPU 오류·동기 렌더 오류에는 현재 게임을 Canvas2D로 다시 표시한다. 게임 재시작이나 리소스 재다운로드는 하지 않는다.
- 실제 renderer와 fallback 사유를 안내/F2에 포함한다. 진단용 `?renderer=canvas2d`를 제공한다.

## 검증 순서

1. 실패하는 offline JS 계약 테스트를 먼저 추가한다: 선택/fallback/장치 손실/캐시/clip/혼합과 순서.
2. 실제 브라우저에서 합성 픽셀 fixture의 Canvas2D/WebGPU 출력을 비교하고 원본/HD 전환·입력·레벨 교체·loss를 확인한다.
3. WASM/웹 회귀와 배포 파일 선별을 확인하고 결과·제한을 보고서에 기록한다.

후속 메인테이너 지시로 구현 검증 완료 뒤 **0.3.4 릴리즈와 Pages 게시**까지 범위를 확장했다. 같은 source tree의 CI7개·4플랫폼 패키지 검증, 태그 전 prepare-only, annotated v0.3.4 태그, 공개 Release/Pages/대응 소스와 플레이를 검증한다. 새 그래픽 자산·시뮬레이션 속도 변경은 포함하지 않는다.

## 기술 근거

- [WebGPU configure](https://developer.mozilla.org/en-US/docs/Web/API/GPUCanvasContext/configure)
- [device.lost](https://developer.mozilla.org/en-US/docs/Web/API/GPUDevice/lost)
- [writeTexture](https://developer.mozilla.org/en-US/docs/Web/API/GPUQueue/writeTexture)
- [WGSL 명세](https://www.w3.org/TR/WGSL/)
