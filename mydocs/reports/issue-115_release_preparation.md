# 개발용 Linux·Web 패키지 준비

담당 [#115](https://github.com/edwardkim/ohmylemmings/issues/115), [배포 계약](../plans/issue-115_release_preparation.md), 성능 후속 [#113](issue-113_web_dispatch.md).

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
