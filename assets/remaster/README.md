# 2D 리마스터 디자인 자산

담당 [#137](https://github.com/edwardkim/ohmylemmings/issues/137), 첫 시안 [#138](https://github.com/edwardkim/ohmylemmings/issues/138).

- [Fun 01 손그림풍 시안](concepts/issue-138_fun01_handpainted_v1.png):1672×941 디자인 후보. 실제 HD 모드·충돌 mask·분리 atlas가 아니다.
- [생성 프롬프트](prompts/issue-138_fun01_handpainted_v1.txt): built-in image_gen에 사용한 최종 문구.
- [출처/규격/해시와 전체 제작 대응 목록](../manifests/remaster_art_direction.json).
- [아트/표시 계약](../../mydocs/plans/issue-138_hd_art_contract.md), [결과·검토 한계](../../mydocs/reports/issue-138_hd_art_direction.md).

이 폴더에는 새로 생성한 디자인 후보를 보관한다. 원본 캡처/바이너리는 Git에 넣지 않으며 ignored assets/local에서만 참조한다. 원본 데이터·캐릭터/상표의 권리와 기존 제3자 고지에 새 허가를 부여하지 않는다. 게임 글자·숫자·배치·물리 경계는 생성 그림 대신 명시한 계약과 실제 세션을 따른다.

[#144 HD-03a](https://github.com/edwardkim/ohmylemmings/issues/144)에서 [걷기16프레임](issue-144-walk-v1.png)과 [암석 재질](issue-144-rock-v1.png)을 별도 PNG로 제작했다. [팩 manifest](../manifests/remaster_pack.json)에 원본 slot/frame·실제 사각형·표시 크기/발 기준점과 hash를 연결한다. [걷기 프롬프트](prompts/issue-144_walk_v1.txt), [암석 프롬프트](prompts/issue-144_rock_v1.txt), [결과/재현 명령](../../mydocs/reports/issue-144_hd_assets.md)을 참고한다.

선택 PNG는 수정 없이 보존한다. 걷기 atlas의alpha1 잔여는 팩의 명시적 cutoff1에 따라 표시 시 제외하며, 나머지 alpha와 픽셀은 유지한다. 보행/색 번짐의 실제 게임 검토와 암석의 반복 경계 인증은 후속이다. 팩 후보의2애니메이션/16프레임과 실제 게임 통합 완료를 구분한다. `runtime_ready=false`이며 실제 앱에 통합한 HD 애니메이션은0개다.

[자산 미리보기 화면](previews/issue-144_assets.png)은 실제 브라우저의 PNG/발 기준점 검토 화면이다. 게임 플레이 캡처가 아니며, 재생 도구는 위 결과 보고서의 명령으로 만든다.

[#147 HD-03b](https://github.com/edwardkim/ohmylemmings/issues/147)에서 [굴착](issue-147-dig-v1.png)·[계단 좌우](issue-147-build-v1.png)·[OhNo/폭발](issue-147-bomb-v1.png)·[입출구](issue-147-facilities-v1.png)·[13조작 아이콘](issue-147-ui-v1.png)을 추가했다. 정확한5개 프롬프트·실제 규격/해시·65개 새 캐릭터 프레임 대응은 [결과 보고서](../../mydocs/reports/issue-147_hd_actions.md)와 현재 팩 manifest를 따른다. 현재 후보는7애니메이션81캐릭터 프레임·11시설 프레임·13아이콘/7atlas106자산이다. 나머지21애니메이션256프레임/다른 테마·시설/효과·전체 UI는 후속이다.

[동작/아이콘 미리보기](previews/issue-147_actions.png), [입구 미리보기](previews/issue-147_facilities.png)는 실제 PNG 검토 화면이다. 새 PNG도 원본 그대로 보존하며 일부 최대alpha254·여백 잔여cutoff1/4를 명시한다. 캐릭터 크기/위상과 출구 doorway·trigger 정합은 실제 게임 통합에서 검토한다. `runtime_ready=false`·실제 HD 앱 통합0개를 유지한다.

[#151 HD-04](https://github.com/edwardkim/ohmylemmings/issues/151)에서 Native/Web의 실제 HD 미리보기와 원본 모드·입력을 연결했다. 현재 `runtime_ready=true`는 **부분 HD 미리보기 사용 가능**을 뜻하며 `full_remaster_ready=false`다. 원본 시안 이미지는 계속 제작 참고용이다. 원본 입구의1닫힘→2~9열기→0열림 유지에 맞춰 자산 프레임 대응도 보정했다. [실제 렌더러의 자체 합성 장면](previews/issue-151_native_synthetic.png)은 원본 바이너리를 사용하지 않은 캡처이며, 실제 Fun01 캡처는 ignored assets/local에 보존한다. [결과/남은 원본 동작·효과](../../mydocs/reports/issue-151_hd_integration.md)를 참고한다.
