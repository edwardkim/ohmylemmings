# Native 표시 목표40FPS

담당 [#240](https://github.com/edwardkim/ohmylemmings/issues/240). [실측](../reports/issue-238_native_gpu_fps.md)에 이어 메인테이너가40FPS면충분하다고판정했다.

Native의 절대 표시격자를60→40Hz(25ms)로 바꾸되게임tick/타이머/이동/애니메이션pacing은 유지한다. 늦은슬롯건너뛰기·숨김/복귀·강제UI/결과표시·render-every의 의미를보존한다. run/performance/F2진단에현재표시목표를기록한다.

Web은이미상태변화때만Canvas를그려자동갱신14~17회/s이므로새주기렌더러를추가하지않는다. rAF콜백빈도와실제재그리기를구분해문서화하고기존상태/음향검사를유지한다. 초기에는Native/Web40FPS상한을함께검토했으나현재구조상추가Web제한이필요없음을확인했다.

TDD로25ms격자·1초40회·지연슬롯·render-every·숨김복귀를검증한다. 원본/HD정상실행을새빈증거경로에서측정하고과거60Hz결과/원본은보존한다. #113의기존60Hz자료판독계산과과거판정을40Hz합격으로바꾸지않는다. 새기준의포괄인수/릴리즈는별도다.
