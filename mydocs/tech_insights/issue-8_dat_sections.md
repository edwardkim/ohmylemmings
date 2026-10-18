# G02-01 DOS DAT 구간 형식과 읽기 계약

작업 이슈: [GitHub #8](https://github.com/edwardkim/ohmylemmings/issues/8). 상위: [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

## 입력과 고정 출처

[원본 매니페스트](../../assets/manifests/dos_original.json)의 `dos-original-vga` 프로필을 대상으로 합니다. [소스 매니페스트](../../assets/manifests/reference_sources.json)에 고정된 다음 구현을 대조했습니다.

- Lemmix `40e9bc34451f0e9127fd53b290a3877e700d143d`: [헤더 레코드](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Dos.Compression.pas#L28-L35), [구간 읽기](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Dos.Compression.pas#L255-L291), [압축기의 헤더·XOR 생성](https://github.com/ericlangedijk/Lemmix/blob/40e9bc34451f0e9127fd53b290a3877e700d143d/src/Dos.Compression.pas#L649-L661).
- Lemmings.ts `10c7c8b9167258503d02589cf048e2d83a43dd0a`: [FileContainer의 필드 읽기](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/file/file-container.ts#L35-L78), [역방향 비트 읽기](https://github.com/tomsoftware/Lemmings.ts/blob/10c7c8b9167258503d02589cf048e2d83a43dd0a/src/game/resources/file/bit-reader.ts#L12-L53).

## 10바이트 헤더

| 구간 시작 기준 위치 | 바이트 | 의미와 지원 범위 |
|---|---|---|
| 0 | 1 | 마지막 본문 바이트의 유효 비트 수, 0~8 허용 |
| 1 | 1 | 본문 전체 바이트의 XOR 체크섬 |
| 2 | 2 | 첫 예약 워드, 지원 프로필에서 0 |
| 4 | 2 | 해제 크기, big-endian u16, 1~65535 |
| 6 | 2 | 둘째 예약 워드, 지원 프로필에서 0 |
| 8 | 2 | 헤더 포함 구간 크기, big-endian u16, 11~65535 |

본문 크기는 `section_size - 10`입니다. 예약 워드를 크기의 상위 16비트로 합치지 않습니다. 이 프로필의 빈 본문·빈 출력·예약 워드가 있는 변형은 지원하지 않으며 오류로 반환합니다. 이는 지원 정책이며 모든 DAT 변형의 보편적 제약으로 주장하지 않습니다.

Lemmix의 비트 초기화는 `aBitCnt + 1` 뒤 감소하며, TypeScript는 비트 수가 0이면 앞 바이트부터 읽습니다. 따라서 0을 8로 바꾸거나 오류로 처리하면 안 됩니다. 로컬 원본에서는 0~7이 관찰됐으며 8 허용은 두 디코더의 읽기 동작을 근거로 합니다.

## Rust API와 오류

[dat.rs](../../crates/lemmings-resources/src/dat.rs)의 `DatSections::new(&bytes)`는 `Iterator<Item = Result<DatSection, DatError>>`를 반환합니다. `DatSection`은 원본 파일의 헤더 시작 `offset()`, 검증한 `header()`, 빌린 `payload()`를 제공합니다. 외부 의존성·할당·파일 I/O가 없으며 `no_std`와 WASM을 유지합니다.

- 빈 입력은 정상적인 구간 0개입니다. 본문 경계와 XOR 검사는 구간을 반환하기 전에 수행합니다.
- 오류는 절단 헤더, 잘못된 비트 수, 지원하지 않는 예약 워드, 작은 구간 크기, 빈 출력, 절단 본문, 체크섬 불일치로 구분합니다. 모든 오류의 `offset`은 해당 구간 헤더의 파일 기준 위치입니다.
- 첫 오류를 한 번 반환한 뒤 계속 `None`을 반환하는 `FusedIterator`입니다. 오류 뒤에서 새 헤더를 찾지 않습니다.
- 파일 끝까지 순회해야 뒤쪽 데이터도 검사됩니다. 앞 구간을 얻었다는 사실만으로 전체 파일이 유효해지는 것은 아닙니다.
- 작업량은 순회한 바이트 수에 비례하며 내부 공간은 일정합니다. 크기를 검사한 뒤 빼고, 남은 길이를 검사한 뒤 위치를 전진시킵니다.

이번 검증은 구간 구조·본문 XOR까지만 확인합니다. XOR는 압축 명령·해제 결과의 유효성을 보장하지 않습니다. 압축 해제와 [고정 해제 해시](../../assets/manifests/dos_compatibility.json) 대조는 G02-02입니다.

## 원본 관찰과 재현

| 파일군 | 파일 수 | 구간 수 |
|---|---|---|
| ADLIB.DAT | 1 | 1 |
| LEVEL000~009.DAT | 10 | 80 |
| MAIN.DAT | 1 | 7 |
| VGAGR0~4.DAT | 5 | 10 |
| VGASPEC0~3.DAT | 4 | 4 |
| 합계 | 21 | 102 |

102개 구간의 예약 워드는 모두 0, 비트 수별 구간 수는 `0:14, 1:15, 2:18, 3:11, 4:16, 5:12, 6:9, 7:7`입니다. 본문 XOR가 모두 일치하고 파일 끝까지 정확히 연결됩니다. GROUND 5개와 ODDTABLE은 이 packed DAT 구간 순회 대상이 아닙니다.

[리소스 확보](../../assets/README.md) 후 저장소 루트에서 메타데이터를 출력할 수 있습니다. 예제의 호스트 파일 I/O는 라이브러리 밖에 있습니다.

```sh
python3 tools/reference_assets.py verify
cargo run -p lemmings-resources --example inspect_dat --locked --offline -- \
  assets/local/dos-original/ADLIB.DAT \
  assets/local/dos-original/LEVEL*.DAT \
  assets/local/dos-original/MAIN.DAT \
  assets/local/dos-original/VGAGR*.DAT \
  assets/local/dos-original/VGASPEC*.DAT
```

표준 출력은 파일·구간 번호·위치·비트·체크섬·해제 크기·구간 크기·본문 크기의 TSV입니다. 손상 구간은 파일과 위치를 포함한 오류 및 실패 종료 코드로 확인합니다. 테스트/CI는 원본 파일을 사용하지 않습니다.

검증 결과: [보고서](../reports/issue-8_dat_sections.md), [진행 기록](../working/issue-8_dat_sections.md), [작업일지](../orders/20261018.md#issue-8).
