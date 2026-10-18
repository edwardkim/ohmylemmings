# G03-05 VGASPEC 특수 배경 결과

작업 이슈: [GitHub #14](https://github.com/edwardkim/ohmylemmings/issues/14). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

- [형식·API·RLE·경계 정책·독립 비교·재현 명령](../tech_insights/issue-14_vgaspec_backgrounds.md)
- [진행 기록](../working/issue-14_vgaspec_backgrounds.md), [작업일지](../orders/20261018.md#issue-14)

## 변경 내용

`VgaSpecFile`은 전체 입력의 정확히 한 DAT 구간을 검증·해제합니다. `VgaSpecRecord`는 40바이트 헤더와 네 RLE 스트립 전체를 검증하고 VGA/EGA·미해석 헤더 바이트를 보존합니다. 각 스트립은 14400바이트이며 네 종료 표식·패킷 경계·후미를 엄격히 검사합니다.

호출자 버퍼에 스트립 또는 전체 57600바이트 플레인을 해제하고, `VgaSpecImage`로 960×160 색상 인덱스·불투명 여부를 제공합니다. 색상 0은 투명합니다. 기존 `Rgb6`와 `IndexedPixel`을 재사용했으며 외부 패키지·라이브러리 할당·I/O는 추가하지 않았습니다. `inspect_vgaspec` 예제와 원본 비교용 고정 해시 매니페스트를 도입했습니다.

## 검증

| 검사 | 결과 |
|---|---|
| 골격·합성 테스트 14개 | Red: 새 14개 실패, 기존 리소스 93개 통과 |
| 구현 후 리소스 단위 테스트 | Green: 107개 통과 |
| `make check` | 전체 Rust 단위 109개·문서 예제 9개·Python 8개, 포맷·Clippy·빌드·WASM 성공 |
| 원본·소스 | 50개·46개 파일 크기·SHA-256 통과, 앞 단계 추가 소스 2개 고정 해시 대조 |
| 원본 4개 전체 대조 | Rust·독립 Python·고정 TS에서 내부 레코드·230400플레인 바이트·614400픽셀 및 팔레트 일치 |
| 기존 DAT 결과 | 4개 내부 레코드 전체 바이트가 G02-02 결과와 일치 |
| 호스트 오류 | 인자 누락/초과·파일 누락·추가 DAT 구간·손상 후미·짧은 헤더·잘못된 팔레트·조기/누락 종료·내부 후미에서 실패와 경로 문맥 확인 |
| 문서 재현 명령 그대로 실행 | 팔레트·전체 해시, 독립 RLE·614400픽셀 재구성 통과 |
| 문서·의존·공백·미완성 코드 | Markdown 57개, 로컬 링크 364개·앵커 62개·이슈 문서명 45개·#14 일지 통과; 기존 로컬 의존 하나·외부 패키지 0개 유지; Rust stub 없음 |

합성 검증은 모든 불완전 접두부, VGA/EGA 각 위치의 64/255 거부와 미해석 값 보존, 복사 명령 0~127·반복 명령 129~255의 전체 길이, 데이터 안의 종료 값 128, 스트립·행·바이트·플레인 순서와 전체 픽셀, 네 스트립의 조기 종료·경계 초과·필수 종료·추가 후미, 큰 인덱스·좌표, 잘못된 출력 길이에서 메모리 보존, DAT 손상·해제·내부 오류 문맥, 원본 레코드와 독립된 이미지 수명을 포함합니다.

## 원본 대조

| 파일 | 내부 레코드 바이트 | 플레인 바이트 | 픽셀 | 투명 픽셀 |
|---|---:|---:|---:|---:|
| VGASPEC0.DAT | 34366 | 57600 | 153600 | 84021 |
| VGASPEC1.DAT | 24368 | 57600 | 153600 | 111194 |
| VGASPEC2.DAT | 27599 | 57600 | 153600 | 108147 |
| VGASPEC3.DAT | 40127 | 57600 | 153600 | 51052 |
| 합계 | 126460 | 230400 | 614400 | 354414 |

독립 Python은 원본 DAT의 비트를 새로 해제하고 RLE 반복자·플레인 이진 문자열로 픽셀을 구성했습니다. TS는 원본 `VgaSpecReader`·`PaletteImage`·DAT 코드를 실행하고 Frame/ColorPalette 캡처 어댑터를 사용했습니다. 원본 Frame의 RGB·알파 렌더링은 검증하지 않았습니다. 모든 바이트·팔레트가 일치한 뒤 [검증 매니페스트](../../assets/manifests/vgaspec_backgrounds.json)에 내부·플레인·픽셀 해시와 팔레트를 고정했습니다. 기존 원본·입력 매니페스트는 유지했습니다.

이 API의 좌표는 배경 내부 (0,0)부터입니다. TS의 x=304 배치 호출은 별도로 확인했으며 전체 레벨 배치·RGB/알파 합성·충돌·실제 DOS 실행 호환성은 후속 범위입니다. G03 전체는 지형 재구성이 남아 진행 중입니다.

## 게시

구현 커밋 `722f9d80d5e0ca56f6e0c42184a0e7dee7df76fd`을 `main`에 게시했습니다. [push CI 실행](https://github.com/edwardkim/ohmylemmings/actions/runs/39116139981)의 결론과 세 작업이 모두 success입니다.

| 작업 | 실제 실행 근거 |
|---|---|
| [Rust native](https://github.com/edwardkim/ohmylemmings/actions/runs/39116139981/job/117642203195) | Rust 1.93.1, 포맷·Clippy·빌드, 단위 109개·문서 예제 9개 성공 |
| [Python tooling](https://github.com/edwardkim/ohmylemmings/actions/runs/39116139981/job/117642203086) | Python 3.12.15, 테스트 8개 성공 |
| [WASM libraries](https://github.com/edwardkim/ohmylemmings/actions/runs/39116139981/job/117642203000) | 고정 Rust·WASM 타깃, 두 라이브러리 컴파일 성공 |

push 이벤트·대상 SHA·세 작업의 실제 로그·테스트 개수·clean-input 성공을 대조했습니다. CI는 원본·기존 target·Cargo 패키지 캐시 없는 러너에서 통과했습니다. 원격 tree API의 파일 91개 모드·blob 해시가 로컬과 모두 일치했으며 원본·추가 소스 사본·JSONL·빌드 출력은 추적하지 않습니다.

G03-05 특수 배경 범위를 완료했습니다. 최종 문서 게시·CI와 #14 종료는 GitHub 이슈 기록에서 추적합니다. 상위 #1의 G03 전체는 지형 재구성이 남아 진행 중이며 G00·G01·G02만 전체 목표 완료로 유지합니다. 다음은 별도 이슈의 G03-06 레벨 지형 재구성입니다.
