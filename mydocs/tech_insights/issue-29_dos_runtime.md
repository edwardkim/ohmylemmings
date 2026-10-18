# DOS 실행 전제와 런타임 설정

이슈: [#29](https://github.com/edwardkim/ohmylemmings/issues/29), 관련 [#28](https://github.com/edwardkim/ohmylemmings/issues/28).

## 실제 초기 실행

DOSBox 0.74-3에서 원본 VGALEMMI /v는 PC 선택 후 디스크 1 부재 메시지로 종료했습니다. C 디렉터리와 A floppy 디렉터리 마운트 모두 같았습니다. 원본 50개에는 키 디스크 이미지·RUSSELL.DAT이 없습니다. EXE의 변경 없이 생성되는 런타임 파일과 실행 전제를 조사합니다.

## 증거 보관

assets/local/issue-28-exploration/의 화면·로그 및 생성 RUSSELL.DAT을 보존합니다. 후속 실행은 manifest의 50개만 새 작업 폴더로 복사하고 실행 전후 검증합니다. 초기 원본 폴더의 추가 생성 파일은 증거 폴더로 이동했습니다.

## 확인한 진입 조건

실패 실행이 생성한 RUSSELL.DAT은 256바이트, 첫 바이트 9입니다. 격리 복사본에서 offset 0만 0으로 seed하면 같은 VGALEMMI.EXE가 메뉴·Fun01로 진입했습니다. 세 번 모두 같은 seed를 사용했고 게임이 갱신한 런타임 파일도 각 해시로 보존했습니다. 입력 50개는 실행 전후 동일합니다. 이 파일은 원본 확보 manifest에 추가하지 않았습니다.

- 생성 파일 SHA-256: `82abd5f27dfa78c0329f7f0ff67755725f28a92e5d8c96f642a3e7df77e333bf`.
- seed SHA-256: `1bedf588c463140458c4bbb39337858e3ed33f857b2d576f5ef1c7c1e12ee2d1`.
- 변경: 단일 offset 0, 9→0. EXE/원본 DAT 패치 없음.
- 초기 실패: assets/local/issue-28-dos-proof-2/bootstrap/capture/dosbox_000.png.
- 재현: [capture_dos.py](../../tools/capture_dos.py)는 새 50개 복사본의 실패/생성 단계를 먼저 실행한 뒤 새 세 복사본에 seed합니다. 기존 폴더를 덮어쓰지 않습니다.

원본 키 디스크를 확보하거나 그 매체를 에뮬레이트한 결과가 아닙니다. 원본 매체의 무설정 기동은 미검증입니다. 작업 범위인 실행 전제 식별을 완료했으며 [#28 결과](../reports/issue-28_dos_comparison.md)의 조건에 반영했습니다.
