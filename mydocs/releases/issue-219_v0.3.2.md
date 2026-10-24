# Oh My Lemmings 0.3.2 — 지형 질감과 플레이 피드백 개선

담당 이슈: https://github.com/edwardkim/ohmylemmings/issues/219

- HD 지형의 질감·경계 명암·반복 무늬와 수직벽 윤곽을 개선했습니다 (#215).
- Digger 굴착에 몸의 충격 동작·분진·절차적으로 합성한 슉슉 효과음을 추가했습니다. 원본 굴착 속도와 범위는 유지합니다 (#216).
- 마우스 대상 표시를 원형 조준점으로 바꾸고, 치명적 추락은 납작한 충격 자세와 분진으로 구별합니다.
- 잔여 수량0인 기술은 마우스와 키보드 모두 선택할 수 없습니다. 2레벨 Digger도 동일합니다.
- 플레이 중 F3으로 FPS 표시, F2로 피드백 요약 복사를 지원합니다 (#113). 성능 목표/메모리의 잔여 검증은 계속합니다.

메인테이너가 지형 수준과 위 플레이 피드백·효과음을 직접 확인하고 통과 판정했습니다. 전체120레벨·HD·점진적 Web 다운로드와 진행률, 로컬 MP3·개별 음량·Native 파일 선택·아이콘을 유지합니다. 사용자 MP3를 서버에 업로드하거나 배포에 포함하지 않습니다.

## 다운로드와 실행

Windows 11 x64 ZIP을 전부 풀고 OhMyLemmings.exe를 실행하세요. Windows 10은 지원하지 않습니다. macOS13+ universal2 앱은 전체를 Applications로 복사합니다. ad-hoc 서명/미공증이며 차단되면 시스템 설정의 개인정보 보호 및 보안에서 실행을 허용할 수 있습니다. Linux x86_64는 SDL2 runtime이 필요합니다.

Web은 [공개 게임](https://edwardkim.github.io/ohmylemmings/)에서 바로 플레이하거나 Web ZIP 전체를 HTTP로 제공합니다. MP3 선택·음량·키 안내는 [0.3.1 실행 안내](https://github.com/edwardkim/ohmylemmings/blob/v0.3.1/mydocs/releases/issue-208_v0.3.1.md)를 참고하세요.

4개 플랫폼 패키지에 게임 데이터·HD·대응 소스·라이선스 고지를 포함합니다. SHA256SUMS와 RELEASE-MANIFEST.json에서 파일 무결성·CI/소스 계보를 확인할 수 있습니다. 자체 코드는 MIT이며 기존 외부 자산/라이브러리 고지를 유지합니다. 과거 릴리즈를 덮어쓰지 않습니다.
