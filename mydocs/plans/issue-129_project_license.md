# #129 자체 코드 라이선스 적용안

이슈: https://github.com/edwardkim/ohmylemmings/issues/129

## 소유자 결정 — MIT / 2026-10-22

소유자가 대화에서 “mit 로 진행하세요”라고 명시하여 자체 작성 코드에 **MIT**를 적용한다. 토이프로젝트의 코드 수정/재사용을 간단히 허용하는 정책이며 상업적 이용도 허용한다. 검토 당시 대안 GPL-3.0-or-later와 제3자/원본 예외를 제시했고, 소유자가 MIT를 선택했다.

| 범위 | 적용/보존할 내용 |
|---|---|
| 자체 작성 Rust·JS·Python/도구·문서 | 루트 LICENSE, README의 적용 범위, 워크스페이스와 해당8개 crate의 metadata에 MIT 반영 |
| crates/lemmings-opl | 기존 LGPL-2.1-or-later와 원저작권·수정/재링크 고지 보존. 루트 선택으로 덮어쓰지 않음 |
| tools/audio/reference | 고정 Nuked-OPL3 C oracle의 LGPL-2.1-or-later 보존 |
| Lemmix 참조/추출 검증 경로 | 독자 Licence.txt 원문·출처 보존. 변경한 Rust 구현과 원본 Pascal을 구분하고 원작/Graphics32 부분의 권리까지 취득했다고 표시하지 않음 |
| Lemmings.ts 참조 경로 | Thomas Zeugner의 MIT 및 파일 안 OPL 별도 고지를 원문 보존. 제품 합성기는 해당 OPL 대신 Nuked-OPL3 Rust 코어를 사용 |
| SDL2 및 Cargo.lock 의존성 | SDL2 Zlib와 잠긴 Rust 의존성18개의 라이선스/고지 확인·전달. 자체 코드 라이선스와 구분 |
| DOS 원본 그래픽·레벨·음악/효과음·상표 | 프로젝트 라이선스 적용 대상에서 제외. 기존 GAME-DATA 출처/해시 보존. 새 재배포 권한을 부여하는 문구를 만들지 않음 |

루트 LICENSE는 `Copyright (c) 2026 Edward Kim and contributors`와 표준 MIT 원문을 사용하고 예외/출처는 별도 고지에서 명시한다. 다른 권리자의 지분까지 재허가했다고 선언하지 않는다.

## 구현·검증 순서

1. 고정 upstream 원문/해시, 로컬 OPL/C oracle, Cargo.lock 의존성 목록과 현재 패키지의 고지 경로를 기록한다.
2. 소유자의 명시적 라이선스 결정을 받는다. 그 전에는 루트 LICENSE/워크스페이스 license를 게시하지 않는다.
3. 결정에 맞춰 metadata·고지·모든 플랫폼 패키지/소스·웹 라이선스 화면을 맞추고, 누락/변조 검사를 TDD로 보강한다.
4. 원본/제3자 예외·기존 LGPL 대응 소스/재링크, 전체 source/archive 해시와 실제 패키지 고지를 대조한다. 과거 v0.1.0은 수정하지 않고 다음 버전에 적용한다.
5. self-review·PR/검증·문서/일지·이슈 상태를 동기화한다. 새 릴리즈/Pages 갱신은 별도 배포 지시 범위를 따른다.

근거: [MIT](https://choosealicense.com/licenses/mit/), [GPLv3](https://choosealicense.com/licenses/gpl-3.0/), [Cargo license metadata](https://doc.rust-lang.org/cargo/reference/manifest.html#the-license-and-license-file-fields), [고정 Lemmix 고지](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/Licence.txt), [고정 Lemmings.ts 고지](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/LICENSE). 이는 적용안과 확인된 원문 범위이며 원본 게임/참조 저장소 전체의 권리 정리를 인증하는 문서가 아니다.
