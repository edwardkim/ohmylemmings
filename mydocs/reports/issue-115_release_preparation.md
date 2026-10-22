# 개발용 Linux·Web 패키지 준비

담당 [#115](https://github.com/edwardkim/ohmylemmings/issues/115), [배포 계약](../plans/issue-115_release_preparation.md), 성능 후속 [#113](issue-113_web_dispatch.md).

## 배포 준비 완료와 후속 이관 / 2026-10-22

이후 소유자가 MIT를 명시적으로 선택하여 [#129의 자체 코드 정책·고지 적용](issue-129_project_license.md)을 완료했다. 남은 사용자 PC 검증은 #130, 성능/메모리는 #113에서 계속한다. 아래는 #115 종료 시점의 이관 기록이다.

사용자가 열린 이슈 검토의 권고안을 수용하여 완료된 배포 준비 #115를 정리하고 미결 항목을 분리한다.

| 완료 항목 | 근거 |
|---|---|
| Linux/Web 개발 패키지·대응 소스·체크섬·설치 CI | PR #117 및 이 보고서의 초기 기록 |
| Windows 전체 원본 게임·런처·정적 CRT/SDL·패키징/실행 CI | [#118](https://github.com/edwardkim/ohmylemmings/issues/118), PR #119, 사용자 실행 확인 |
| macOS ARM/Intel universal2·원본/SDL·실제 양쪽 실행 | [#120](https://github.com/edwardkim/ohmylemmings/issues/120), PR #121 |
| Linux/Web 전체 원본 게임·기본 Fun01·압축 해제 검사 | [#122](https://github.com/edwardkim/ohmylemmings/issues/122), PR #123 |
| 버전·불변 태그/asset·4플랫폼 v0.1.0 공개 다운로드 | [#124](https://github.com/edwardkim/ohmylemmings/issues/124), PR #125, [릴리즈](https://github.com/edwardkim/ohmylemmings/releases/tag/v0.1.0) |
| Pages 원본 게임·동일 payload·실제 공개 기능 검사/플레이 배지 | [#126](https://github.com/edwardkim/ohmylemmings/issues/126), PR #127/#128, [플레이](https://edwardkim.github.io/ohmylemmings/) |

[CI39766459833](https://github.com/edwardkim/ohmylemmings/actions/runs/39766459833) main3594bb0의7개 작업이 모두 SUCCESS임을 재확인했다. [Pages39766461582](https://github.com/edwardkim/ohmylemmings/actions/runs/39766461582)도 SUCCESS이다. 이 기록 확인은 새 전체 해법 실행·성능 측정이나 실제 사람의 Windows10/11 청취 검사로 계산하지 않는다.

잔여 자체 코드 라이선스 결정/고지는 [#129](https://github.com/edwardkim/ohmylemmings/issues/129), Windows10/11별 사용자 PC의 화면·입력·물리 청취 기록은 [#130](https://github.com/edwardkim/ohmylemmings/issues/130)으로 이관했다. Windows packager/런처·CI의 완료와 아직 없는 OS별 수동 검증 기록을 분리했다. 성능/메모리는 기존 #113에서 계속하며 #1의 G11/G12 전체는 미완료로 유지한다. Mac ad-hoc/미공증은 안내한 현재 배포 조건이고 자격 확보 후의 정식 서명은 별도 단계다.

이번 변경은 이슈/문서 정리이다. 자체 코드 라이선스를 선택하거나 기존 공개 태그/asset을 바꾸지 않는다. self-review에서 각 완료 항목의 이슈/PR·실제 CI·후속 링크를 대조하고, 과거 계약을 이력으로 구분했다. 추가한 로컬 문서 링크23개·git diff --check와 make check-python(Python243개/기존 증거 index 포함) PASS. 문서 전용 변경으로 전체 플랫폼 빌드 행렬은 반복하지 않는다. 정확한 PR/병합/완료 처리 결과는 GitHub #115 최종 기록에 남긴다.

## 초기 개발 패키지 구현 기록

이하 내용은 PR #117 당시의 범위와 한계이다. 이후 계약 변경과 현재 결과는 위 완료/이관 기록을 따른다.

현재 호스트의 60fps 반복 조정을 중단하고 배포 준비로 이동한다는 사용자 지시에 따라 개발용 `0.1.0` Linux x86_64 TAR.GZ와 Web ZIP을 만든다. Windows와 더 좋은 호스트의 성능 판정은 미검증이며 정식 공개 릴리스나 G11/G12 완료를 선언하지 않는다.

## 사용

고정 개발 의존성을 준비한 뒤 `make package-linux`, `make package-web`을 실행한다. 출력은 무시되는 `assets/local/dist/` 아래다. 동일 경로의 기존 증거는 덮어쓰지 않는다. 다시 만들 때는 기존 결과를 보존하고 `python3 tools/release_package.py linux|web --runtime 경로 --output 새폴더`를 사용한다.

```sh
python3 tools/release_package.py verify --archive assets/local/dist/ohmylemmings-0.1.0-dev-web.zip
python3 tools/check_release_package.py --archive assets/local/dist/ohmylemmings-0.1.0-dev-web.zip
```

패키지의 README에 실행·사용자 소유 자산 입력과 개발 상태를 명시한다. Linux는 SDL2 동적 런타임이 필요하며 Ubuntu24.04 CI를 깨끗한 빌드 기준으로 쓴다. Web은 압축을 풀어 HTTP로 제공하며 `file://` 실행을 안내하지 않는다.

## 구성·검증

플랫폼별 명시적 런타임 허용 목록만 읽으며 원본 DAT/EXE, 성능 기록, 개인 파일이나 staging 디렉터리를 복사하지 않는다. Web JS/CSS/HTML이 현재 소스와 다르면 오류를 반환한다. 누락·빈 파일·심볼릭 링크와 다른 ELF 아키텍처/WASM 헤더도 거부한다.

전체 대응 소스 묶음·OPL 고지/라이선스·수정/재링크 안내·내부 파일별 SHA256과 외부 archive SHA256을 함께 전달한다. 외부 해시뿐 아니라 내부 허용 목록·파일 해시·소스의 전체 inventory/바이트/해시와 Web WASM/라이선스 연결을 검사한다. 시간·소유자·정렬·권한을 고정하며 같은 입력의 archive 바이트가 같은지 두 플랫폼에서 검사한다. 다른 compiler/압축 라이브러리 환경 사이의 bit-for-bit 빌드 인증은 아니다.

압축을 새 임시 폴더에 푼 뒤 해당 파일로 검사한다. Linux는 SDL dummy/software에서 Won→Restart→Won→Continue→Lost와 실제 SDL dummy 오디오 큐/비영 PCM을 검사한다. Web은 실제 WASM/DOM의 무변화 rAF·틱·입력/release·완료/재시작, 실제 AudioContext의 제출·음소거/재활성화, HTTP 라이선스/소스 전달을 검사한다. 자동 오디오 활성화 플래그를 쓴 기능 검사이며 실제 사람의 청취나 성능 합격을 대신하지 않는다.

CI의 기존 Ubuntu24.04 Native/Web job이 깨끗한 checkout/build 뒤 패키지·설치 검사를 수행한다. 통과한 개발 패키지와 SHA256을 14일 보관하는 artifact로 남긴다. upload-artifact v7.0.2의 공식 tag→commit 및 Node24/input 메타데이터를 확인하고 전체 commit SHA를 고정했다.

## 확인과 잔여

오프라인 패키징 테스트7개: 원본/staging 제외, 소스/해시 전달, 두 형식의 결정성, 실행 권한, 누락/링크/stale/다른 형식, 덮어쓰기 거부, 외부/내부 변조 및 추가 파일 거부다. 로컬 pilot의 Linux·Web 실제 압축 해제 실행도 통과했다. 정확한 최종 CI·artifact/병합 증거는 #115 게시 기록에 연결한다.

Windows 실제 빌드·DLL/설치 검증, 고성능 호스트의 성능 판정, 최종 릴리스 판단은 남아 있다. 저장소에 프로젝트 전체 라이선스가 아직 없으므로 이를 임의로 부여하지 않는다. 현재 작업은 개발용 artifact 준비이며 정식 공개 배포 전 라이선스 결정도 필요하다.
