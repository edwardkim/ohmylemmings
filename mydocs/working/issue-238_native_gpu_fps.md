# 이 PC의 Native GPU FPS 측정

담당 [#238](https://github.com/edwardkim/ohmylemmings/issues/238). 제품기준 #235, main60d2373.

1920×1080/60Hz 실제 모니터·Wayland에서 release binary를 fullscreen HD Fun01/동일 replay/420논리틱(약30초)으로 실행한다. GPU와 명시software를 ABBA 순서로 각2회 측정하고 첫5초를 제외한다. --fast/캡처/기존performance 옵션은 사용하지 않는다. 기존performance가HD를거부하므로 게임바이너리변경없이 LD_PRELOAD observer로 SDL_RenderPresent 전후 CLOCK_MONOTONIC만 미리 할당한 배열에 보관하고 종료시 기록한다. 관측은 물리scanout이 아닌 present 완료 간격이다. software baseline과 이 PC의 현재 동작을 비교하며60fps 맞추기 튜닝이나 #113 인수는 아니다. 원본 유래 자료/observer/원시측정은 ignored assets/local/issue-238-fps/에 보관한다.

첫ABBA측정은 메인테이너가 VNC 접속 중이었음을 확인했다. 원시결과를 유지하고 사용자 지시에 따라 VNC 연결해제 후 같은 조건/순서를 다시 측정한다. wayvncctl이 현재 client0을 반환하고 compositor의 RECORD 차단 사유도 사라졌다. 서버 종료나 시스템 설정 변경은 하지 않는다. 두번째 자료는 ignored assets/local/issue-238-fps/no-vnc/에 분리한다.

완료: 실제 Native HD1920×1080/Wayland/사운드/동일입력으로 VNC접속·해제 각각GPU/software2회씩 총8회 측정했다. GPU59.94→59.98FPS,software44.30→45.78FPS. 상세조건/원시증거hash/표시완료와물리FPS의구분은 [보고서](../reports/issue-238_native_gpu_fps.md)를 따른다.
