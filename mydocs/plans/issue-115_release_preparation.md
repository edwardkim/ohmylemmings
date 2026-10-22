# 배포 대상·검증 계약

담당 [#115](https://github.com/edwardkim/ohmylemmings/issues/115), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 현재 상태 — 배포 준비 완료 / 2026-10-22

사용자가 #115의 완료 항목 정리와 잔여 작업 분리 권고안을 수용했다. Linux/Web 개발 패키징(#117), Windows 전체 게임(#118), macOS universal2(#120), Linux/Web 원본 동봉(#122), 버전·태그 정책/공개 v0.1.0(#124), GitHub Pages 플레이(#126)를 완료했다. [최종 정리·검증](../reports/issue-115_release_preparation.md#배포-준비-완료와-후속-이관--2026-10-22)을 따른다.

| 후속 | 남은 범위 |
|---|---|
| [#129](https://github.com/edwardkim/ohmylemmings/issues/129) | 자체 코드 MIT 선택·고지/metadata 정비 적용 완료. [보고서](../reports/issue-129_project_license.md) |
| [#130](https://github.com/edwardkim/ohmylemmings/issues/130) | Windows 10/11 각각의 사용자 PC 화면·입력·실제 청취 기록 |
| [#113](https://github.com/edwardkim/ohmylemmings/issues/113) | 더 좋은 호스트의 성능·구간 메모리 재검증 |

Mac의 ad-hoc/미공증은 현재 공개 플레이 배포의 안내 조건이다. Developer ID가 없는 상태에서 새 인증서 작업을 시작하지 않는다. #115 종료는 배포 준비의 완료이며 G11 성능 합격·G12 전체 완료나 라이선스 선택을 뜻하지 않는다.

## 최초 계약과 변경 이력

아래는 첫 개발 패키지 계약이다. 원본 제외/Windows 미검증/공개 보류 상태는 뒤의 #118/#120/#122/#124 변경과 위 최신 상태로 갱신되었다.

2026-10-22 사용자 승인으로 G12 준비를 시작한다. #113의 60fps 완료 마감 검증은 더 좋은 호스트로 이월하며 현재 장비에서 반복하지 않는다. G11 성능 합격과 G12 완성을 선언하지 않는다.

| 대상 | 준비할 산출물 | 완료 증거 |
|---|---|---|
| Linux x86_64 | release 실행 파일·실행 안내·라이선스/전체 대응 소스·체크섬의 압축 묶음 | Ubuntu24.04 깨끗한 소스 빌드, SDL2 런타임 의존성 확인, 원본 없는 실행/재시작/음향 smoke |
| Web | 런타임 정적 파일과 WASM·라이선스/대응 소스·HTTP 실행 안내·체크섬 | 깨끗한 WASM 빌드, 실제 브라우저 연습 장면/입력/결과/재시작·오디오, 원본 제외 archive 검사 |
| Windows x86_64 | MSVC 실행 파일과 필요한 SDL2 런타임·라이선스/대응 소스·체크섬 ZIP | 실제 Windows runner 빌드·패키지/의존 DLL·합성 실행 검증. 현재 미검증이며 Linux 교차 빌드로 대신하지 않음 |

버전은 기존 `0.1.0`을 기준으로 개발용 패키지임을 구분한다. 준비 중 임의의 정식 릴리스 태그나 지원 인증을 게시하지 않는다. 패키징은 명시한 런타임 파일만 허용하고 `assets/local/`, 원본 DAT/EXE, 캡처/PCM·개인 파일·개발 캐시를 제외한다. 대응 소스 묶음의 공개 출처·파일별 해시와 실제 실행/WASM의 해시를 연결한다.

기능 회귀와 패키지 검증은 계속한다. 이 호스트에서의 60fps 조정·긴 성능 반복은 하지 않는다. 고성능 환경의 성능 검증과 최종 공개 릴리스 판단은 잔여 항목으로 명시한다.

첫 구현은 [Linux·Web 개발용 패키징](../reports/issue-115_release_preparation.md)이다. Windows 실제 runner/DLL 검증은 후속 절편으로 유지한다. 저장소 전체 라이선스가 아직 정해지지 않았으므로 임의로 허가를 부여하거나 정식 배포를 게시하지 않는다.

### #118 Windows 다운로드 계약 변경

사용자가 시험 ZIP의 합성 장면만 실행되는 문제를 확인하고 원본을 포함한 온전한 게임 패키지를 지시했다. [#118](https://github.com/edwardkim/ohmylemmings/issues/118)에서 Windows 다운로드에 원본 그래픽·120개 레벨·음원을 포함하고 exe 더블클릭으로 시작하도록 변경한다. 앞선 원본 제외 방침은 Linux/Web 개발 패키지의 기존 계약이며 Windows에 대한 이 최신 지시로 대체한다. 성능 이월은 유지한다.

### #120 macOS 추가

Windows 실행을 확인한 사용자가 Mac 배포 준비를 요청했다. [#120](issue-120_macos_full_game.md)에서 Apple Silicon/Intel universal2 앱·원본28개·SDL을 동봉하고 실제 두 아키텍처에서 검증한다. 서명 자격이 없는 현재 시험판 ad-hoc와 정식 Developer ID/공증 단계를 구분하며 성능 이월은 유지한다.

### #122 Linux·Web 계약 변경

사용자가 다음 작업1번을 승인해 [#122](issue-122_linux_web_full_game.md)에서 Linux/Web 다운로드에도 원본28개·120레벨·음원을 동봉하고 기본 Fun01 실행으로 변경한다. 앞선 원본 제외는 명시적인 개발 모드에만 유지한다. Windows/Mac 완료와 실제 CI 증거는 각 #118/#120을 따른다. Apple Developer ID는 준비되지 않았으며 자체 Homebrew tap의 소스 Formula가 후속 후보다. 기존 앱 Cask는 Gatekeeper 요구를 없애지 않는다.

### #124 사용자 플레이 릴리즈 승인

사용자는 토이프로젝트를 Releases에서 내려받아 플레이하게 하도록 공개 배포를 승인했고 버전·태그 정책 수립을 추가 지시했다. [#124 정책](issue-124_release_policy.md)에 따라 첫 v0.1.0과4플랫폼 파일을 게시한다. 앞선 공개 릴리즈 보류 문구는 이 최신 지시로 대체한다. 프로젝트/원본에 임의의 라이선스를 부여하지 않으며 #113 성능 이월과 Mac 공증 상태를 구분한다.
