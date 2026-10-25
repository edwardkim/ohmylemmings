# Linux 그래픽 확장 조사 진행

담당 [#233](https://github.com/edwardkim/ohmylemmings/issues/233).

현재 원본/HD 경로·자산 로더·CPU 지형 합성과 SDL upload를 조사했다. 짧은 offscreen SDL probe에서 Intel Mesa OpenGL4.6 가속/16384 최대texture를 확인했다. 공식 SDL/wgpu/KTX 문서를 대조하고 선택형 고품질 자산과 단계적 GPU 이행을 권고했다. 제품 구현/배포는 수행하지 않았다. [결과 보고서](../reports/issue-233_linux_graphics_review.md)를 따른다.
