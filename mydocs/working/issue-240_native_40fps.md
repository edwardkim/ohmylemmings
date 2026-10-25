# 40FPS 표시 목표 진행

담당 [#240](https://github.com/edwardkim/ohmylemmings/issues/240). [계획](../plans/issue-240_native_40fps.md).

Native 표시격자와진단/현행문서를조정한다. Web의14~17회변경기반Canvas표시는유지하고rAF와구분한다. 검증/실측을진행한다.

25ms 표시격자·1초40회·지연/간격배수/복귀의5개테스트로TDD red/green을확인했다. 실제SDL software시작검사도통과했다. run/performance/F2에presentation_base_fps를기록한다. 전체Rust/Python/CI와실제화면측정은후속이다.
