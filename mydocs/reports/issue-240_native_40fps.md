# Native 표시 목표 40 FPS 반영

담당 [#240](https://github.com/edwardkim/ohmylemmings/issues/240). [계획](../plans/issue-240_native_40fps.md), [이전 60Hz 실측](issue-238_native_gpu_fps.md).

메인테이너의 판단에 따라 Native 기본 표시 목표를 60→40 FPS, 표시 간격을 25ms로 바꿨다. 원본/HD 및 GPU/software가 같은 시계를 사용한다. 게임 논리 tick, DOS/reference pacing, 타이머, 이동과 음향 처리는 유지한다.

## 표시 주기와 진단

- 절대 시간 격자를 사용하므로 렌더 처리 시간이 다음 대기에 누적되지 않는다. 늦은 슬롯 건너뛰기와 숨김/복귀·레벨 교체 시 시계 초기화를 유지했다.
- `--render-every N`은 평상시 40/N회/s로 표시 간격을 늘린다. `--fast`의 논리 tick 기준 간격은 그대로다. 초기화·입력·결과·캡처의 강제 그리기는 주기 표시 외에도 발생할 수 있다. 따라서 모든 호출을 엄격히 초당40회 이하로 제한하는 계약은 아니다.
- 실행 summary, performance JSON, F2 텍스트에 `presentation_base_fps:40`을 기록한다. 이 값은 기본 목표이며 실측 FPS나 render-every 적용 후 목표가 아니다.
- 현행 Native·패키지·측정 안내를 갱신했다. 과거 #107/#238의 60Hz 측정, #113의 과거 판독 계산과 NOT READY 결과는 보존한다. 기준 변경만으로 새 성능 인수를 통과 처리하지 않는다.

Web은 이미 상태가 바뀔 때만 Canvas를 그려 일반적인 자동 갱신이 약14~17회/s다. 입력·새 레벨·결과 UI는 즉시 그린다. rAF의60/120/144Hz 호출과 실제 Canvas 갱신은 별개다. 소스 검토 후 Web에는 추가40Hz 타이머를 넣지 않고 기존 시간 공급·음향 서비스·변경 기반 그리기를 유지하기로 범위를 좁혔다. Web 문서에 이 차이를 명시했다.

## 검증

표시 시계5개 테스트는 25ms 격자의60초 유지, 정확한1초40회, 101ms 지연 후125ms 다음 슬롯, render-every3의75ms 주기, 숨김/복귀·리셋을 확인한다. 이전60Hz 코드에서4개가 실패하는 red와 수정 후5개 green을 기록했다.

`make check-rust` 통과: Rust745개, format/Clippy 및 상태 재생·음악·효과음 검사. `make check-python` 통과: Python339개, 자산·고지·검증 증거 검사. 실제 SDL dummy 실행으로 software 경로와 픽셀/상태, 새 목표 진단을 검증했다. 현재 기능 앵커를40Hz 테스트로 맞추고 이전 앵커·소스 해시를 manifest의 issue-240 기록에 보존했다.

## 이 PC의 실제 HD 확인

Intel N100 PC의1920×1080/60Hz 모니터, Wayland, HD Fun01, 사운드와 동일 replay로420논리 tick(약30초)씩 GPU와 software를1회 실행했다. 현재 VNC 클라이언트가1개 연결되어 이를 유지하고 실행 전후 같은 상태임을 확인했다. 로컬 빌드와 테스트가 끝난 뒤 측정했다.

| 경로 | 표시 완료 평균FPS | 기록된 기본 목표 | 제어 tick | 입력 수 |
|---|---:|---:|---:|---:|
| opengl | 40.04 | 40 | 420 | 1 |
| software | 38.70 | 40 | 420 | 1 |

#238과 같은 임시 LD_PRELOAD 관측기로 SDL_RenderPresent 전후 단조 시간만 배열에 보관하고 종료 시 기록했다. 처음5초와 끝1초를 제외한 회당약23.8초를 분석했으며, 해당 구간의 출력은 모두1920×1080이다. 수치는 물리 scanout이 아닌 SDL 표시 완료 간격이다. GPU40.04는 표시 경계와 강제 그리기를 포함한 약40FPS로 해석한다. Software는 이번 실행에서 목표에 조금 못 미쳤다. 이 검사는 목표 설정의 동작 확인이며, 이전 VNC 해제 상태와 동일 조건의 비교나 전체 성능 인수는 아니다.

[PR#241](https://github.com/edwardkim/ohmylemmings/pull/241)의 [CI40160115426](https://github.com/edwardkim/ohmylemmings/actions/runs/40160115426) 7개 항목이 모두 SUCCESS다. 검증 head는 `39ac8a63afe4aeaf0f74a0681a057488d4f087a8`, 실제 PR checkout은 `ed38ab9`다. 병합 main `4748e4ed854cd40d25306715d8ed607e290050dc`와 같은 tree `4fbf9e156a0476e580a1adef01b0ce794b3c6d18`를 사용한다. 중복 push CI만 취소했다.

제품 병합에는 skip-ci를 쓰지 않아 [main CI40161119605](https://github.com/edwardkim/ohmylemmings/actions/runs/40161119605)가 자동 시작됐다. 문서 작성 시 진행 중이며, 완료된 인수 근거는 위 동일 tree PR CI다.

개인 red/green·로그·원시 측정은 ignored `assets/local/issue-240-fps/`에 보관하고 로컬 주 작업 폴더에도 복사했다. 개발 main과 로컬 빌드에 반영하며, 공개 v0.3.3/Pages의 새 배포는 별도 절차다.
