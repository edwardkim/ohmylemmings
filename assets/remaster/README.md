# 2D 리마스터 디자인 자산

담당 [#137](https://github.com/edwardkim/ohmylemmings/issues/137), 첫 시안 [#138](https://github.com/edwardkim/ohmylemmings/issues/138).

- [Fun 01 손그림풍 시안](concepts/issue-138_fun01_handpainted_v1.png):1672×941 디자인 후보. 실제 HD 모드·충돌 mask·분리 atlas가 아니다.
- [생성 프롬프트](prompts/issue-138_fun01_handpainted_v1.txt): built-in image_gen에 사용한 최종 문구.
- [출처/규격/해시와 전체 제작 대응 목록](../manifests/remaster_art_direction.json).
- [아트/표시 계약](../../mydocs/plans/issue-138_hd_art_contract.md), [결과·검토 한계](../../mydocs/reports/issue-138_hd_art_direction.md).

이 폴더에는 새로 생성한 디자인 후보를 보관한다. 원본 캡처/바이너리는 Git에 넣지 않으며 ignored assets/local에서만 참조한다. 원본 데이터·캐릭터/상표의 권리와 기존 제3자 고지에 새 허가를 부여하지 않는다. 게임 글자·숫자·배치·물리 경계는 생성 그림 대신 명시한 계약과 실제 세션을 따른다.

런타임용 리소스가 추가되면 분리된 상태/방향/frame·발 기준점·내부 사각형·hash·제작/검증 상태를 manifest에 연결한다. 현재 runtime_ready는 false이며 실제 HD 애니메이션 자산은0개다.
