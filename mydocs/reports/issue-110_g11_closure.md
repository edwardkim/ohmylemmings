# G11 전체 증거·배포 진입 준비 검토

담당 [#110](https://github.com/edwardkim/ohmylemmings/issues/110), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1), 기준 main `2de60fd144ee664979f5e824993f4e1c9c3b2abd`, 검토일 2026-10-22(Asia/Seoul).

**검토 결과: G11 미완료·G12 진입 보류.** 기능 호환성과 네이티브 표시 보정의 기존 통과 범위는 유효하다. 웹 콜백 예산 초과와 표시 완료 마감·구간 메모리 증거 부족은 [#111](https://github.com/edwardkim/ohmylemmings/issues/111)에서 해결해야 한다. #110 종료는 이 검토 절차의 완료이며 G11 전체 합격을 뜻하지 않는다. #85의 요구를 제약으로 바꿔 면제하지 않는다.

## 확인한 증거와 적용 범위

|대상|재검사 결과|판정 범위|
|---|---|---|
|120개 초기·240개 성공/실패|레벨/목표/입력·독립 기준·상태/맵/프레임 SHA, SDL/WASM 각3반복과 재시작 집계 유효|#91~#104의 저장 실행 증거, 새 전수 플레이 아님|
|COMP-01~18·세부70개|현재350개 제품/검사 소스 pin·테스트 함수·증거 연결 유효|PASS-SCOPED, DOS 내부 모든 상태 동치 인증 아님|
|직업8·음악21·효과18|COMP 경계 및 자원/음향 증거에 연결, 원본50개 파일 SHA 재확인|기존 지원 계약·독립 기준·PCM 관측 범위|
|알고리즘380행|Native184/Web196,3 warmups+30표본의 median/p95/max/처리량 재계산 일치|#106의 같은 환경 결과, 새 측정·최적화 비교 아님|
|Native60Hz 보정|60초 전후3쌍, 수정3회 마감 초과/슬롯 생략/관측 underrun0|#107의 예정 present 완료 마감, 물리 scanout 아님|
|보정 동일성|합성/원본451파일·40PCM의 저장 동일성 결과 및 현재 host 소스 pin 유효|#107의 실제 대조, 이번에 게임 재생 반복 안 함|

이번 `--local` 검사는 COMP 색인의 로컬124개 pin, 성능 입력 원본50개, 채택 Web 원시3개와 Native 전후6개를 검사한다. 과거 수만 개 파생 캡처 전체를 새로 해시하거나 전수 재생한 검사는 아니다. 코어·자원·렌더러·오디오·호스트 코드는 변경하지 않았다. #106의 opt-in 관측과 #107의 표시 주기 변경 영향은 각 source_impact 기록으로 연결한다. 과거 CI·실행 숫자를 현재 실행 숫자로 바꾸지 않는다.

## 웹에서 남은 세 항목

|#106 채택 실행|콜백 수|콜백 최대 비용|60Hz 예산(정확히1/60초) 초과|
|---|---:|---:|---:|
|1|3601|9.3ms|0|
|2|3598|22.6ms|3|
|3(교체 채택)|3599|13.7ms|0|

원시 `callbacks`의 duration_ms를 정수 ns로 반올림해 공개 [g11_review.json](../../assets/manifests/g11_review.json)에 보존했다. `ns * 60 > 1_000_000_000`으로 비교한다. 두 번째 실행의 초과 비용은22.6/19.0/16.8ms다. 기존 약60fps와 관측 underrun0은 이 초과를 상쇄하지 않는다. 콜백 비용에는 observer와 OS 스케줄링 등이 섞이며, 엔진 자체의 병목이나 GC를 원인으로 확정하지 않는다.

1. **web-callback-budget:** 세 번 중 한 번에서 콜백3개가 표시 예산을 넘었다. #111에서 비용 분해·필요 보정·재측정을 수행한다.
2. **web-completion-deadline:** 기존 observer는 콜백 진입/기간을 저장하지만 원래 rAF timestamp와 예정 완료 마감을 저장하지 않는다. 콜백 예산 초과 수를 실제 완료 마감 위반 수나 물리 dropped frame 수로 대체할 수 없다. frame gap·진입 지연·완료 마감을 분리해 관측해야 한다.
3. **web-interval-memory:** 기존 값은 단조 증가 WASM committed 끝값(8,781,824bytes)과 근사 JS heap 끝값이다. PCM 선형 메모리1,310,720bytes는 포함 관계를 밝혀야 하며 합산해 독립 사용량으로 취급하지 않는다. JS/observer/GC를 분리한 구간 peak 증거가 없다. 기존 끝값을 peak로 이름만 바꾸지 않는다. 알고리즘/실시간 Web 메모리의 관측 범위와 보완 방법을 #111에서 명시한다.

AudioBufferSource API의 실제 hardware quantum, 물리 DAC/scanout 및 임의 기기 전체의 성능 보증은 현재 증거 범위 밖이다. Native VmHWM은 초기화/관측 버퍼를 포함한다. 이런 API/관측 제약과 위 미충족 요구를 구분한다.

## 재현과 자동 판정

```sh
source assets/local/dev-tools/env.sh
make check-python
python3 tools/g11_review.py --local
python3 tools/g11_review.py --require-ready
```

기본 검사는 원본 없이 공개 pin·COMP 집계·380행 통계·콜백 벡터·판정 일관성을 확인한다. `review_valid:true`는 증거 검토의 유효성을 뜻하며 `g11_ready:false`, `g12_entry_ready:false`를 함께 출력한다. `--local`은 로컬 원시 pin과 공개 콜백 벡터의 바이트/변환 일치를 추가 검사한다. `--require-ready`는 현재 예상대로 종료코드1이다. 보통 CI의 성공을 G11 성능 통과로 오해하지 않는다.

누락 모듈에서 실패하는 red를 확인한 뒤5개 테스트를 통과했다. 정확한60Hz ns 경계, 잘못된 표본/부족한 반복, 평균 FPS로 초과를 면제하는 잘못된 판정, 빠진 마감/peak 증거, 저장 판정 덮어쓰기를 검사한다. Python 전체195개 및 원본 없는 감사·motion 기준 검사를 수행한다. 제품 실행 코드 변경이 없어 로컬 Rust 전수와 긴 플레이를 반복하지 않으며 게시 CI의 기존 전체 Rust/WASM/브라우저 회귀를 확인한다. 정확한 head/checkout·로그/검사 수, self-review·PR/병합/main·기존 미리보기 소스 묶음 갱신은 [#110 최종 기록](https://github.com/edwardkim/ohmylemmings/issues/110)에 남긴다.

## 다음 절차

#111을 해결하고 G11 판정을 다시 검토한다. #1의 G11/G12 체크는 미완료로 유지한다. G12에 진입하면 지원 OS/버전·원본 취득 정책을 확정하고, Windows/Linux/웹을 깨끗한 환경에서 문서대로 빌드·패키징/실행해야 한다. 현재 Linux/웹 CI와 소스 전달은 그 일부이며 Windows 배포 빌드의 대체 증거가 아니다. 릴리스 체크섬·라이선스/대응 소스·원본 제외·설치/자산 가져오기·재시작/음향 smoke를 별도로 검증한다. 이번 검토는 릴리스를 만들거나 배포하지 않는다.
