# #175 물·불 위험 요소 HD

이슈: https://github.com/edwardkim/ohmylemmings/issues/175 · 상위 #137

원본 물/불 객체의 프레임·표시 크기·위험 trigger·순서/클리핑을 조사하고 새 그림을 연결한다. 시뮬레이션·음악 합성은 바꾸지 않는다. TDD와 Native/Web 위험 사건/화면 검증, 출처·프롬프트·해시, 감사·self-review·CI·실제 병합으로 완료한다. 기계 함정·나머지 장식/효과·전체 UI는 후속이다. 사용자 CDP19222/8080과 preview-172를 유지한다.

## 구현·로컬 검증 진행

새8PNG/9객체84프레임과 키 범위·누락 검사를 구현했다. 기존 Object 렌더링에 HD/원본 위험 프레임 진단을 더했다. 전체84프레임2회 반복 검사에서 상태/사운드가 불변이었다. Rust HD parser18·Python 대상31테스트 통과. 원본31page 바이트 일치, 새7,970,037 RGBA 샘플 일치, 전체 카탈로그173,879,960바이트/39page·모든25scope 최대119,886,132바이트/24page를 확인했다.

실제 SDL9합성 코스991상태에서 익사5·vaporize4, 원본/HD 상태·입력·마스크·사운드가 일치했다. 분사기4픽셀 trigger를 actor 발 위치에 맞추도록 **검증용 코스 배치만** 보정했다. 원본 메타데이터·제품 시뮬레이션은 바꾸지 않았다. Web 동일 코스/전체84위상·원본120장면·전체 로컬 검사·감사·self-review·게시 CI와 실제 병합은 진행 중이다.

재현: `python3 tools/check_hd_hazards.py --pack assets/local/web/hd-assets --output assets/local/<새 폴더>` → `tools/web/hd_hazards.cjs`에 별도 CDP/코스 cases.json을 전달. 사용자19222/8080·preview-172는 유지한다.

최종 완료: PR #176 head `75ae0af61fe4d8e06c9be668e289daa00c6833e3` / CI39939672234의7개 SUCCESS, 실제 merge `775608d4c47c6b62eff291052144e4a2c6e76883`와 검토 head 전체 tree 일치를 확인했다. 로컬 전체 검사와9개 위험 코스/원본120장면을 통과했다. 위 진행 중/미완료 문구는 당시 기록이다. [최종 결과·재현·한계](../reports/issue-175_hd_hazards.md)를 따른다.
