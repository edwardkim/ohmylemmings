# #200 사용자 K-pop BGM 교체 검토 상태

이슈: https://github.com/edwardkim/ohmylemmings/issues/200

2026-10-24 검토/문서화 완료. 기준 main `fa306e1cdee5a93b822e76bd628e491900d12625`. [계획](../plans/issue-200_custom_bgm_review.md) · [기술 판단·권고·구현 시 인수](../reports/issue-200_custom_bgm_review.md).

현재 BGM/효과음은 하나의 OPL mono PCM/공통 음량으로 나온다. 출력 계층의 effects-only 모드와 외부 음악 경로·독립 gain/수명주기를 추가하면 Web/Native에서 교체 가능하다. 첫 구현은 사용자 MP3 파일 선택을 권고한다. 공식 Spotify 정책의 게임/동기화/혼합 금지와 공개 음원 포함 시 별도 사용 권한 확인을 구분했다.

제품 코드·의존성·테스트 음원·릴리즈/Pages 파일은 변경하지 않았다. 검토 문서를 self-review/링크 검사 후 병합하고 검토 이슈를 종료한다. 사용자 파일 재생/디코더 성능/전 기기 호환성은 구현 시 검증할 항목이다. 구현은 별도 이슈로 추적한다.
