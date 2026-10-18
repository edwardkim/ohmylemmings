# Rusty Lemmings

고전 DOS 게임 레밍즈(Lemmings)의 데이터와 게임 동작을 Rust로 재현하는 프로젝트입니다. 렌더링·입출력과 분리된 코어 엔진을 만들고, SDL2 기반 데스크톱 환경과 WebAssembly 기반 웹 환경을 지원하는 것이 목표입니다.

## 현재 상태

현재는 기획과 문서 구조를 정리한 단계입니다. Rust 소스와 `Cargo.toml`은 아직 없으며, 빌드·실행·테스트 명령은 구현 단계에서 추가합니다.

## 구현 목표

- 원본 DAT/VGASPEC 리소스 파싱
- 고정 틱 기반의 결정론적 게임 루프
- 레밍의 행동 상태 머신과 중력·충돌 처리
- 파기와 폭발에 따른 픽셀 단위 지형 변경
- 데스크톱과 웹에서 동작하는 플랫폼 어댑터

전체 설계와 단계별 작업은 [프로젝트 기획서](mydocs/plans/rusty_lemmings.md)를 참고하세요.

구현 순서와 목표별 완료 기준은 [분할정복 기반 전체 구현 계획](mydocs/plans/implementation_plan.md)에 정리합니다. 첫 착수 목표는 원작 기준판과 호환성 검증 기준 확정(G00)입니다.

진행 상황은 [GitHub 이슈 #1](https://github.com/edwardkim/ohmylemmings/issues/1)에서 추적합니다.

## Hyper-Waterfall 개발 방법론

이 프로젝트는 관리자가 정립한 Hyper-Waterfall을 채택합니다. 구현 전에 관련 문서로 컨텍스트를 맞추고, 작은 모듈부터 TDD로 구현하며, 피드백과 기술 지식·문제 해결 이력을 문서에 반영합니다.

```text
mydocs/
├── plans/             # 계획 및 아키텍처
├── orders/            # 작업 지시 및 요구사항
├── working/           # 진행 중 작업 상태와 컨텍스트
├── reports/           # 작업 결과와 검증 보고서
├── feedbacks/         # 피드백과 반영 결과
├── tech_insights/     # 기술 분석 및 도메인 지식
└── troubleshootings/  # 문제 해결 이력
```

## 참여 방법

작업을 시작하기 전에 [기여 지침](AGENTS.md)과 [문서 안내](mydocs/README.md)를 읽어주세요. PR에는 변경 목적, 관련 작업, 검증 결과를 기록하고 계획된 기능과 구현된 동작을 구분합니다.
