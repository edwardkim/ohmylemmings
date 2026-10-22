# Windows 패키징 가능성 검토

담당 [#115](https://github.com/edwardkim/ohmylemmings/issues/115), [검토 진행 기록](../working/issue-115_windows_assessment.md), [기계 증거](../../assets/manifests/windows_packaging_review.json).

**가능하다. 기존 게임 Rust 코드를 바꾸지 않고 실제 Windows MSVC 빌드와 시험 ZIP의 압축 해제 실행에 성공했다.** 권장 초기 형태는 Windows10/11 x64 대상의 portable ZIP, 정적 MSVC CRT와 동봉 SDL2.dll이다. 정식 패키징 명령 통합과 실제 사용자 PC의 화면/입력/청취 검증은 남아 있다. 현재 main의 Linux/Web 패키징은 변경하지 않았다.

## 실제 확인

Windows Server2022 GitHub runner, Rust1.93.1, SDL2 2.32.10 VC x64 입력이다. `LIB`로 import library를 전달하며 제품 Rust/Cargo.lock은 main `41ec527`과 동일하다. SDL 입력의 공식 archive SHA256과 DLL/라이선스를 확인하고, checkout의 LF를 유지했다.

|후보|실제 Windows run|EXE 크기|CRT 동적 import|압축 해제 실행|
|---|---|---:|---|---|
|기본 MSVC|[39726923791](https://github.com/edwardkim/ohmylemmings/actions/runs/39726923791)|903,168bytes|VCRUNTIME140.dll·UCRT API-set 있음|PASS|
|`+crt-static`|[39727233075](https://github.com/edwardkim/ohmylemmings/actions/runs/39727233075)|1,000,448bytes|VCRUNTIME140/MSVCP140/UCRT API-set 없음|PASS|

두 후보 모두 별도 **한글·공백 경로**에 ZIP을 풀고 그 exe로 help, Won→Restart→Won→Continue→Lost와 실제 SDL dummy audio queue를 검사했다. 제출8,192frames 중 비영3,625frames, 소비1,024bytes를 확인했다. 이는 물리 장치의 청취나 실제 데스크톱의 키/마우스/DPI 검증은 아니다.

실제 PE32+/AMD64 header와 `dumpbin /DEPENDENTS`를 확인했다. 정적 CRT exe의 직접 import는 `SDL2.dll`, `KERNEL32.dll`, `ntdll.dll`, `api-ms-win-core-synch-l1-2-0.dll`이다. SDL2.dll의 직접 import도 Windows 시스템 DLL이다. 이 결과는 실행 중 선택적으로 로드하는 모든 모듈이나 모든 Windows 설치 상태를 인증하지 않는다.

원격 artifact container digest와 시험 ZIP hash, 내부8개 파일·704개 대응 소스의 전체 hash를 검사했고 모든 소스 바이트가 해당 실행의 Git blob과 일치했다. 정적 후보의 실행 source는 `1ad5be8426d5b20139f4a0116776c0f90dc620f2`, ZIP SHA256은 `eb15a02399593b3501140e602ab2280d005a70abd8b20d69a0d0e8a02dc43f79`다. 두 run의 정확한 checkout·성공 step·원시 로그도 보존했다.

## 배포 구성과 필요한 변경

권장 ZIP은 `lemmings-desktop.exe`, `SDL2.dll`, 실행 안내/런처, SDL2와 LGPL 라이선스, 전체 대응 소스/재링크 안내, 내부/외부 SHA256을 포함한다. 원본 DOS 자산은 포함하지 않는다. [Rust 공식 문서](https://doc.rust-lang.org/reference/linkage.html#static-and-dynamic-c-runtimes)가 설명하는 `RUSTFLAGS=-C target-feature=+crt-static` 선택과 실제 import 확인을 함께 사용한다. [MSVC target의 Windows 최소 기준](https://doc.rust-lang.org/rustc/platform-support/windows-msvc.html)은 Windows10+이며 이번 실제 runner는 Server2022다.

현재 bare exe는 인수 없이 실행하면 저장소용 상대 자산 경로 `assets/local/dos-original`을 찾는다. 압축만 푼 사용자가 더블클릭해 연습 장면을 시작하려면 `--synthetic-outcome win` 같은 인수를 주는 런처가 필요하다. 원본 플레이의 자산 폴더 지정도 안내하거나 선택 UI를 제공해야 한다.

정식 구현은 기존 `release_package.py`의 Windows target/PE·필수 DLL/SDL 라이선스 허용 목록, ZIP native 실행 분기, 고정 SDL acquisition manifest와 offline 오류 검사, Windows CI job을 추가하면 된다. 기존 코어·자원·시뮬레이션 구조를 바꿀 필요는 이번 시험에서 발견하지 못했다. `archive_bytes`의 ZIP와 대응 소스 생성은 이미 재사용했다.

정식 패키지 완료 전에는 Windows10/11 실제 PC에서 창/커서/키·패널/미니맵·배율·실제 오디오와 자산 폴더/재시작을 확인해야 한다. 시험 ZIP은 **검토용**이며 정식 릴리스/지원 인증이 아니다. 프로젝트 전체 라이선스와 #113 고성능 호스트 성능 판정도 기존 잔여 항목으로 유지한다. 이 검토에서 현재 호스트의60fps 조정은 재개하지 않았다.
