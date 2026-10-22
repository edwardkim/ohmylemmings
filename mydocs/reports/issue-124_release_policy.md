# 버전·태그 관리와 첫 다운로드 릴리즈

담당 [#124](https://github.com/edwardkim/ohmylemmings/issues/124), 상위 [#115](https://github.com/edwardkim/ohmylemmings/issues/115).

사용자의 토이프로젝트 Releases 배포 승인과 버전·태그 관리 지시를 반영했다. [정책](../plans/issue-124_release_policy.md)은 workspace 단일 버전/SemVer·정식/시험판·annotated v태그·게시 후 불변·수정 시 새 버전·변경 내역/승격 절차를 정한다. 첫 버전은 기존0.1.0. 버전/상속9개/lock/버전별 이슈 연결 노트를 CI가 검사하며 패키징/CI의 고정0.1.0 경로를 제거했다. Mac의 숫자 bundle 필드는 base버전, 전체 시험판 접미사는 GetInfoString/manifest/archive에 유지한다.

태그 Release workflow는 같은 소스 트리의 성공한 CI7개와4개 플랫폼 artifact wrapper digest·archive/원본/대응 소스·설치 요약을 확인한다. 모든 대응 소스 파일이 현재 tag checkout과 같아야 한다. draft 업로드를 다시 다운로드한 뒤 공개하고 익명 공개 URL의 바이트도 확인한다. 기존 릴리즈/파일을 덮어쓰지 않는다. 빌드/태그 커밋·트리·CI·asset hashes는 RELEASE-MANIFEST, 실행 안내는 PLAY.md, 개별/전체 SHA와 별도 대응 소스 archive를 제공한다.

버전/TDD5개와 승격/게시6개 red→green, Mac 시험판 숫자 필드1개 추가와 기존 패키징 회귀를 수행했다. 잘못된 버전/태그·lock/노트 누락, 다른 트리/실패·부족 CI, 원본과 다른 대응 소스, 플랫폼/버전 불일치, lightweight 태그, 손상된 업로드의 publish 차단을 확인했다. 전체 Python236/버전 정책/감사 PASS. CI·실제 게시 검증/다운로드 링크는 #124에서 정확한 source/run/tag와 함께 기록한다.

게임 코어/렌더/OPL/60fps 경로를 바꾸지 않는다. #113 성능 이월과 Mac ad-hoc/미공증, 기존 라이선스/원본 출처를 유지한다. 사용자는 첫 플레이 배포를 승인했으며 인증서/전체 라이선스를 추가 승인 조건으로 만들지 않는다. G11 인증을 새로 주장하지 않는다.
