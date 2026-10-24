# #194 v0.2.0 정식 게시·Pages

이슈: https://github.com/edwardkim/ohmylemmings/issues/194

메인테이너가0.2.0 릴리즈 작업 시작을 지시했다. 기준 main316d869의 HD/BGM 수정·버전 적용을 유지하고 공개 노트/다운로드·배포 제한을 확정한다. #165 청취 미확인을 명시하며 릴리즈 지시를 청취 정상으로 간주하지 않는다.

Pages의 기존50MiB 다운로드·100MiB 확장·100파일 제한은 약190MB대/103파일 HD Web 패키지를 거절했다. 먼저 실제 형태의103파일 합성 HD 카탈로그와200MiB 다운로드 메타데이터에서 실패를 확인했다.256MiB 다운로드/512MiB 확장/256파일로 범위를 조정하고 변조·누락·출처·해시·full game·최신 안정 릴리즈 검사를 유지했다. 초과 확장은 전체 검증/출력 전에 거절하는 경계를 별도로 검사했다. 사이트 예산은 [GitHub 공식1GB 제한](https://docs.github.com/en/pages/getting-started-with-github-pages/github-pages-limits) 이내다.

Release workflow에 main 후보를 검증만 하는 `prepare_only` 수동 경로를 추가한다. 태그/릴리즈/Pages를 만들지 않고 동일 tree CI 산출물 전체를 prepare하고 manifest/체크섬 증거를 남긴다. 이 사전 검증 성공 후에만 annotated 태그를 push한다. 기존 tag/default publish 경로는 draft 업로드·재다운로드 일치 후 공개하고 Pages를 승격한다. Pages 검증 도구와 자산은 입력한 공개 태그에서 checkout하여 source와 일치시킨다. HD 전체 다운로드/검증을 위해 게시 job30분·Pages job20분으로 한정하며 GitHub Pages의 실제 deployment 제한을 바꾸지 않는다.

공개용0.2.0 노트는 기존191 파일을 게시 전 확정하며 이후 태그/asset/버전별 노트를 덮어쓰지 않는다.0.1.0은 보존한다. source tree·CI7개·preflight·annotated tag·draft/익명 다운로드·공개 Pages 버전/해시·실제 Web 플레이 결과는 작업 종료 시 이슈 및 후속 완료 기록에 고정한다. 이 소스 스냅샷은 게시 준비 단계의 기록이다.
