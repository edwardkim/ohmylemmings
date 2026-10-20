# G11-01 전체 120레벨 점검표

이슈: [#85](https://github.com/edwardkim/ohmylemmings/issues/85) · [계획](../plans/issue-85_compatibility_validation.md) · [기계 판독 목록](../../assets/manifests/compatibility_plan.json).

기준 main `7b64dad34d90fa27249b9d76a6339dd1faed3aa2`와 [기준 목록](../../assets/manifests/dos_compatibility.json)을 대조했다. 이 표는 현재 실행 결과가 아닌 소스 지원 범위와 기존 증거의 색인이다.

120개 모두 [지형](../../assets/manifests/level_terrain.json)·[객체](../../assets/manifests/level_objects.json)의 기존 정적 대조 대상이다. 동적 객체·직업·승패·플랫폼 실행의 전수 통과를 뜻하지 않는다. `범위 내` 29개도 실행 통과가 아니다. `난이도`는 Fun 외 선택/ID 연결 미구현 90개, `특수`는 공통 장면 거부 4개이며 3개가 중복되어 차단된 진입점은 91개다.

스타일/특수는 원본 헤더 값이며 특수 0은 일반 장면이다. 부품과 ODD는 0 기준이다. G11 열은 로드·성공·실패·재시작·3회 재생/양 플랫폼 상태 대조 묶음의 새 검증 상태로, 120개 모두 미검증이다. Fun01의 기존 성공/실패/재시작과 Fun02 진입 증거는 [#60](../reports/issue-60_native_playthrough.md)·[#70](../reports/issue-70_web_playthrough.md), 일부 통합 음향은 [#82](../reports/issue-82_audio_playthrough.md)에 보존한다.

| ID | 제목 | DAT:부품 | ODD | 스타일/특수 | 현재 소스 범위/차단 원인 | 기존 호스트 증거 | G11 실행 묶음 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| fun-01 | Just dig! | LEVEL009.DAT:1 | — | 0/0 | 범위 내 | Fun01 완주/음향 일부 | 미검증 |
| fun-02 | Only floaters can survive this | LEVEL009.DAT:5 | — | 3/0 | 범위 내 | Fun02 진입만 | 미검증 |
| fun-03 | Tailor-made for blockers | LEVEL009.DAT:6 | — | 3/0 | 범위 내 | — | 미검증 |
| fun-04 | Now use miners and climbers | LEVEL009.DAT:2 | — | 2/0 | 범위 내 | — | 미검증 |
| fun-05 | You need bashers this time | LEVEL009.DAT:3 | — | 4/0 | 범위 내 | — | 미검증 |
| fun-06 | A task for blockers and bombers | LEVEL009.DAT:4 | — | 1/0 | 범위 내 | — | 미검증 |
| fun-07 | Builders will help you here | LEVEL009.DAT:7 | — | 2/0 | 범위 내 | — | 미검증 |
| fun-08 | Not as complicated as it looks | LEVEL000.DAT:6 | 6 | 3/0 | 범위 내 | — | 미검증 |
| fun-09 | As long as you try your best | LEVEL001.DAT:2 | 10 | 2/0 | 범위 내 | — | 미검증 |
| fun-10 | Smile if you love lemmings | LEVEL003.DAT:2 | 26 | 3/0 | 범위 내 | — | 미검증 |
| fun-11 | Keep your hair on Mr. Lemming | LEVEL004.DAT:2 | 34 | 2/0 | 범위 내 | — | 미검증 |
| fun-12 | Patience | LEVEL000.DAT:7 | 7 | 3/0 | 범위 내 | — | 미검증 |
| fun-13 | We all fall down | LEVEL001.DAT:6 | — | 1/0 | 범위 내 | — | 미검증 |
| fun-14 | Origins and Lemmings | LEVEL001.DAT:7 | 15 | 3/0 | 범위 내 | — | 미검증 |
| fun-15 | Don't let your eyes deceive you | LEVEL002.DAT:2 | 18 | 3/0 | 범위 내 | — | 미검증 |
| fun-16 | Don't do anything too hasty | LEVEL002.DAT:4 | 20 | 1/0 | 범위 내 | — | 미검증 |
| fun-17 | Easy when you know how | LEVEL002.DAT:7 | 23 | 2/0 | 범위 내 | — | 미검증 |
| fun-18 | Let's block and blow | LEVEL004.DAT:3 | 35 | 1/0 | 범위 내 | — | 미검증 |
| fun-19 | Take good care of my Lemmings | LEVEL005.DAT:1 | 41 | 2/0 | 범위 내 | — | 미검증 |
| fun-20 | We are now at LEMCON ONE | LEVEL006.DAT:3 | 51 | 2/0 | 범위 내 | — | 미검증 |
| fun-21 | You Live and Lem | LEVEL008.DAT:4 | 68 | 0/0 | 범위 내 | — | 미검증 |
| fun-22 | A Beast of a level | LEVEL001.DAT:3 | — | 0/4 | 특수 | — | 미검증 |
| fun-23 | I've lost that Lemming feeling | LEVEL004.DAT:1 | 33 | 1/0 | 범위 내 | — | 미검증 |
| fun-24 | Konbanwa Lemming san | LEVEL005.DAT:7 | 47 | 2/0 | 범위 내 | — | 미검증 |
| fun-25 | Lemmings Lemmings everywhere | LEVEL006.DAT:0 | 48 | 4/0 | 범위 내 | — | 미검증 |
| fun-26 | Nightmare on Lem street | LEVEL007.DAT:1 | 57 | 3/0 | 범위 내 | — | 미검증 |
| fun-27 | Let's be careful out there | LEVEL004.DAT:6 | 38 | 2/0 | 범위 내 | — | 미검증 |
| fun-28 | If only they could fly | LEVEL006.DAT:1 | 49 | 1/0 | 범위 내 | — | 미검증 |
| fun-29 | worra lorra lemmings | LEVEL006.DAT:5 | 53 | 3/0 | 범위 내 | — | 미검증 |
| fun-30 | Lock up your Lemmings | LEVEL008.DAT:2 | 66 | 1/0 | 범위 내 | — | 미검증 |
| tricky-01 | This should be a doddle! | LEVEL000.DAT:0 | — | 0/0 | 난이도 | — | 미검증 |
| tricky-02 | We all fall down | LEVEL001.DAT:6 | 14 | 1/0 | 난이도 | — | 미검증 |
| tricky-03 | A ladder would be handy | LEVEL002.DAT:1 | 17 | 0/0 | 난이도 | — | 미검증 |
| tricky-04 | Here's one I prepared earlier | LEVEL003.DAT:0 | 24 | 0/0 | 난이도 | — | 미검증 |
| tricky-05 | Careless clicking costs lives | LEVEL003.DAT:1 | 25 | 4/0 | 난이도 | — | 미검증 |
| tricky-06 | Lemmingology | LEVEL003.DAT:3 | 27 | 1/0 | 난이도 | — | 미검증 |
| tricky-07 | Been there, seen it, done it | LEVEL003.DAT:4 | 28 | 3/0 | 난이도 | — | 미검증 |
| tricky-08 | Lemming sanctuary in sight | LEVEL004.DAT:7 | 39 | 1/0 | 난이도 | — | 미검증 |
| tricky-09 | They just keep on coming | LEVEL006.DAT:2 | 50 | 0/0 | 난이도 | — | 미검증 |
| tricky-10 | There's a lot of them about | LEVEL007.DAT:3 | 59 | 1/0 | 난이도 | — | 미검증 |
| tricky-11 | Lemmings in the attic | LEVEL007.DAT:7 | 63 | 2/0 | 난이도 | — | 미검증 |
| tricky-12 | Bitter Lemming | LEVEL008.DAT:0 | 64 | 2/0 | 난이도 | — | 미검증 |
| tricky-13 | Lemming Drops | LEVEL008.DAT:3 | 67 | 3/0 | 난이도 | — | 미검증 |
| tricky-14 | MENACING !! | LEVEL000.DAT:2 | — | 0/2 | 난이도+특수 | — | 미검증 |
| tricky-15 | Ozone friendly Lemmings | LEVEL009.DAT:1 | 73 | 0/0 | 난이도 | — | 미검증 |
| tricky-16 | Luvly Jubly | LEVEL009.DAT:3 | 75 | 4/0 | 난이도 | — | 미검증 |
| tricky-17 | Diet Lemmingaid | LEVEL009.DAT:4 | 76 | 1/0 | 난이도 | — | 미검증 |
| tricky-18 | It's Lemmingentry Watson | LEVEL009.DAT:5 | 77 | 3/0 | 난이도 | — | 미검증 |
| tricky-19 | Postcard from Lemmingland | LEVEL009.DAT:7 | 79 | 2/0 | 난이도 | — | 미검증 |
| tricky-20 | One way digging to freedom | LEVEL000.DAT:3 | — | 1/0 | 난이도 | — | 미검증 |
| tricky-21 | All the 6`s ........ | LEVEL000.DAT:5 | — | 1/0 | 난이도 | — | 미검증 |
| tricky-22 | Turn around young lemmings! | LEVEL000.DAT:6 | — | 3/0 | 난이도 | — | 미검증 |
| tricky-23 | From The Boundary Line | LEVEL000.DAT:7 | — | 3/0 | 난이도 | — | 미검증 |
| tricky-24 | Tightrope City | LEVEL001.DAT:0 | — | 2/0 | 난이도 | — | 미검증 |
| tricky-25 | Cascade | LEVEL001.DAT:1 | — | 3/0 | 난이도 | — | 미검증 |
| tricky-26 | I have a cunning plan | LEVEL001.DAT:2 | — | 2/0 | 난이도 | — | 미검증 |
| tricky-27 | The Island of the Wicker people | LEVEL001.DAT:4 | — | 0/0 | 난이도 | — | 미검증 |
| tricky-28 | Lost something? | LEVEL001.DAT:5 | — | 0/0 | 난이도 | — | 미검증 |
| tricky-29 | Rainbow Island | LEVEL002.DAT:0 | — | 2/0 | 난이도 | — | 미검증 |
| tricky-30 | The Crankshaft | LEVEL001.DAT:7 | — | 3/0 | 난이도 | — | 미검증 |
| taxing-01 | If at first you don`t succeed.. | LEVEL002.DAT:2 | — | 3/0 | 난이도 | — | 미검증 |
| taxing-02 | Watch out, there`s traps about | LEVEL002.DAT:3 | — | 0/0 | 난이도 | — | 미검증 |
| taxing-03 | Heaven can wait (we hope!!!!) | LEVEL002.DAT:4 | — | 1/0 | 난이도 | — | 미검증 |
| taxing-04 | Lend a helping hand.... | LEVEL002.DAT:5 | — | 3/0 | 난이도 | — | 미검증 |
| taxing-05 | The Prison! | LEVEL002.DAT:6 | — | 1/0 | 난이도 | — | 미검증 |
| taxing-06 | Compression Method 1 | LEVEL002.DAT:7 | — | 2/0 | 난이도 | — | 미검증 |
| taxing-07 | Every Lemming for himself!!! | LEVEL003.DAT:0 | — | 0/0 | 난이도 | — | 미검증 |
| taxing-08 | The Art Gallery | LEVEL003.DAT:1 | — | 4/0 | 난이도 | — | 미검증 |
| taxing-09 | Perseverance | LEVEL003.DAT:2 | — | 3/0 | 난이도 | — | 미검증 |
| taxing-10 | Izzie Wizzie lemmings get busy | LEVEL003.DAT:3 | — | 1/0 | 난이도 | — | 미검증 |
| taxing-11 | The ascending pillar scenario | LEVEL003.DAT:4 | — | 3/0 | 난이도 | — | 미검증 |
| taxing-12 | Livin` On The Edge | LEVEL003.DAT:5 | — | 1/0 | 난이도 | — | 미검증 |
| taxing-13 | Upsidedown World | LEVEL003.DAT:6 | — | 0/0 | 난이도 | — | 미검증 |
| taxing-14 | Hunt the Nessy.... | LEVEL003.DAT:7 | — | 0/0 | 난이도 | — | 미검증 |
| taxing-15 | What an AWESOME level | LEVEL000.DAT:1 | — | 0/1 | 난이도+특수 | — | 미검증 |
| taxing-16 | Mary Poppins` land | LEVEL004.DAT:0 | — | 1/0 | 난이도 | — | 미검증 |
| taxing-17 | X marks the spot | LEVEL004.DAT:1 | — | 1/0 | 난이도 | — | 미검증 |
| taxing-18 | Tribute to M.C.Escher | LEVEL004.DAT:2 | — | 2/0 | 난이도 | — | 미검증 |
| taxing-19 | Bomboozal | LEVEL004.DAT:3 | — | 1/0 | 난이도 | — | 미검증 |
| taxing-20 | Walk the web rope | LEVEL004.DAT:4 | — | 4/0 | 난이도 | — | 미검증 |
| taxing-21 | Feel the heat! | LEVEL004.DAT:5 | — | 1/0 | 난이도 | — | 미검증 |
| taxing-22 | Come on over to my place | LEVEL004.DAT:6 | — | 2/0 | 난이도 | — | 미검증 |
| taxing-23 | King of the castle | LEVEL004.DAT:7 | — | 1/0 | 난이도 | — | 미검증 |
| taxing-24 | Take a running jump..... | LEVEL005.DAT:0 | — | 1/0 | 난이도 | — | 미검증 |
| taxing-25 | Follow the leader... | LEVEL005.DAT:1 | — | 2/0 | 난이도 | — | 미검증 |
| taxing-26 | Triple Trouble | LEVEL005.DAT:2 | — | 1/0 | 난이도 | — | 미검증 |
| taxing-27 | Call in the bomb squad | LEVEL005.DAT:3 | — | 3/0 | 난이도 | — | 미검증 |
| taxing-28 | POOR WEE CREATURES! | LEVEL005.DAT:4 | — | 4/0 | 난이도 | — | 미검증 |
| taxing-29 | How do I dig up the way? | LEVEL002.DAT:1 | — | 0/0 | 난이도 | — | 미검증 |
| taxing-30 | We all fall down | LEVEL006.DAT:7 | — | 1/0 | 난이도 | — | 미검증 |
| mayhem-01 | Steel Works | LEVEL005.DAT:5 | — | 0/0 | 난이도 | — | 미검증 |
| mayhem-02 | The Boiler Room | LEVEL005.DAT:6 | — | 1/0 | 난이도 | — | 미검증 |
| mayhem-03 | It`s hero time! | LEVEL005.DAT:7 | — | 2/0 | 난이도 | — | 미검증 |
| mayhem-04 | The Crossroads | LEVEL006.DAT:0 | — | 4/0 | 난이도 | — | 미검증 |
| mayhem-05 | Down, along, up. In that order | LEVEL006.DAT:1 | — | 1/0 | 난이도 | — | 미검증 |
| mayhem-06 | One way or another | LEVEL006.DAT:2 | — | 0/0 | 난이도 | — | 미검증 |
| mayhem-07 | Poles Apart | LEVEL006.DAT:3 | — | 2/0 | 난이도 | — | 미검증 |
| mayhem-08 | Last one out is a rotten egg! | LEVEL006.DAT:4 | — | 1/0 | 난이도 | — | 미검증 |
| mayhem-09 | Curse of the Pharaohs | LEVEL006.DAT:5 | — | 3/0 | 난이도 | — | 미검증 |
| mayhem-10 | Pillars of Hercules | LEVEL006.DAT:6 | — | 2/0 | 난이도 | — | 미검증 |
| mayhem-11 | We all fall down | LEVEL006.DAT:7 | 55 | 1/0 | 난이도 | — | 미검증 |
| mayhem-12 | The Far Side | LEVEL007.DAT:0 | — | 0/0 | 난이도 | — | 미검증 |
| mayhem-13 | The Great Lemming Caper | LEVEL007.DAT:1 | — | 3/0 | 난이도 | — | 미검증 |
| mayhem-14 | Pea Soup | LEVEL007.DAT:2 | — | 2/0 | 난이도 | — | 미검증 |
| mayhem-15 | The Fast Food Kitchen... | LEVEL007.DAT:3 | — | 1/0 | 난이도 | — | 미검증 |
| mayhem-16 | Just a Minute... | LEVEL007.DAT:4 | — | 2/0 | 난이도 | — | 미검증 |
| mayhem-17 | Stepping Stones | LEVEL007.DAT:5 | — | 2/0 | 난이도 | — | 미검증 |
| mayhem-18 | And then there were four.... | LEVEL007.DAT:6 | — | 0/0 | 난이도 | — | 미검증 |
| mayhem-19 | Time to get up! | LEVEL007.DAT:7 | — | 2/0 | 난이도 | — | 미검증 |
| mayhem-20 | No added colours or Lemmings | LEVEL009.DAT:2 | 74 | 2/0 | 난이도 | — | 미검증 |
| mayhem-21 | With a twist of lemming please | LEVEL008.DAT:0 | — | 2/0 | 난이도 | — | 미검증 |
| mayhem-22 | A BeastII of a level | LEVEL000.DAT:4 | — | 0/3 | 난이도+특수 | — | 미검증 |
| mayhem-23 | Going up....... | LEVEL008.DAT:1 | — | 4/0 | 난이도 | — | 미검증 |
| mayhem-24 | All or Nothing | LEVEL008.DAT:2 | — | 1/0 | 난이도 | — | 미검증 |
| mayhem-25 | Have a nice day! | LEVEL008.DAT:3 | — | 3/0 | 난이도 | — | 미검증 |
| mayhem-26 | The Steel Mines of Kessel | LEVEL008.DAT:4 | — | 0/0 | 난이도 | — | 미검증 |
| mayhem-27 | Just a Minute (Part Two) | LEVEL008.DAT:5 | — | 2/0 | 난이도 | — | 미검증 |
| mayhem-28 | Mind the step..... | LEVEL008.DAT:6 | — | 2/0 | 난이도 | — | 미검증 |
| mayhem-29 | Save Me | LEVEL008.DAT:7 | — | 3/0 | 난이도 | — | 미검증 |
| mayhem-30 | Rendezvous at the Mountain | LEVEL009.DAT:0 | — | 0/0 | 난이도 | — | 미검증 |
