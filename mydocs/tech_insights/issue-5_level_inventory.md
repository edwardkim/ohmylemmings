# DOS 전체 레벨 지원 목록

작업 이슈: [#5](https://github.com/edwardkim/ohmylemmings/issues/5), 상위 [#1](https://github.com/edwardkim/ohmylemmings/issues/1).

[호환성 검증 기준](issue-5_compatibility_baseline.md)과 [기계 판독용 목록](../../assets/manifests/dos_compatibility.json)을 함께 사용합니다.

## 읽는 방법

- 지원 대상은 Fun·Tricky·Taxing·Mayhem 각 30개, 총 120개 진입점입니다. 실제 플레이 호환성 통과 여부는 G11에서 기록합니다.
- 고정 Lemmix의 `SectionTable`과 Lemmings.ts의 `level.order`가 DAT 파일·구간·변형 여부 120개 모두 일치했습니다.
- DAT 10개를 해제한 기본 레코드는 80개이며 각 2,048바이트입니다. 40개 진입점은 `ODDTABLE.DAT`의 56바이트 레코드에서 초기 수치·제목을 덮어씁니다.
- 구간과 ODD 인덱스는 0부터 시작합니다. ODD는 표시 레벨의 전체 순번이 아니라 `파일 번호 × 8 + 구간`으로 찾습니다. `—`는 기본 레코드 사용입니다.
- `수/구출`은 개체 수/필요 구출 수, 시간은 분, `C/F/O/B/U/A/M/D`는 Climber/Floater/Bomber/Blocker/Builder/Basher/Miner/Digger 수량입니다.
- 제목과 초기 수치는 고정 원본 바이트에서 읽었습니다. 기본 레코드 해제 결과는 독립 Python 분석과 고정 TypeScript 디코더의 SHA-256을 대조했습니다.

## Fun

| ID | 제목 | DAT | 구간 | ODD | RR | 수/구출 | 분 | C/F/O/B/U/A/M/D |
|---|---|---|---:|---:|---:|---:|---:|---|
| fun-01 | Just dig! | LEVEL009.DAT | 1 | — | 50 | 10/1 | 5 | 0/0/0/0/0/0/0/10 |
| fun-02 | Only floaters can survive this | LEVEL009.DAT | 5 | — | 50 | 10/1 | 5 | 0/10/0/0/0/0/0/0 |
| fun-03 | Tailor-made for blockers | LEVEL009.DAT | 6 | — | 50 | 50/5 | 5 | 0/0/0/10/0/0/0/0 |
| fun-04 | Now use miners and climbers | LEVEL009.DAT | 2 | — | 1 | 10/10 | 5 | 10/0/0/0/0/0/1/0 |
| fun-05 | You need bashers this time | LEVEL009.DAT | 3 | — | 50 | 50/5 | 5 | 0/0/0/0/0/50/0/0 |
| fun-06 | A task for blockers and bombers | LEVEL009.DAT | 4 | — | 50 | 50/10 | 5 | 0/0/5/5/0/0/0/0 |
| fun-07 | Builders will help you here | LEVEL009.DAT | 7 | — | 50 | 50/25 | 5 | 0/0/0/0/20/0/0/0 |
| fun-08 | Not as complicated as it looks | LEVEL000.DAT | 6 | 6 | 88 | 80/76 | 5 | 20/20/20/20/20/20/20/20 |
| fun-09 | As long as you try your best | LEVEL001.DAT | 2 | 10 | 99 | 80/72 | 5 | 20/20/20/20/20/20/20/20 |
| fun-10 | Smile if you love lemmings | LEVEL003.DAT | 2 | 26 | 50 | 20/10 | 5 | 20/20/20/20/20/20/20/20 |
| fun-11 | Keep your hair on Mr. Lemming | LEVEL004.DAT | 2 | 34 | 50 | 60/50 | 5 | 20/20/20/20/20/20/20/20 |
| fun-12 | Patience | LEVEL000.DAT | 7 | 7 | 99 | 80/40 | 5 | 20/20/20/20/20/20/20/20 |
| fun-13 | We all fall down | LEVEL001.DAT | 6 | — | 1 | 20/20 | 3 | 0/0/0/0/0/0/0/20 |
| fun-14 | Origins and Lemmings | LEVEL001.DAT | 7 | 15 | 20 | 80/60 | 6 | 20/20/20/20/20/20/20/20 |
| fun-15 | Don't let your eyes deceive you | LEVEL002.DAT | 2 | 18 | 40 | 80/40 | 8 | 20/20/20/20/20/20/20/20 |
| fun-16 | Don't do anything too hasty | LEVEL002.DAT | 4 | 20 | 1 | 80/50 | 8 | 20/20/20/20/20/20/20/20 |
| fun-17 | Easy when you know how | LEVEL002.DAT | 7 | 23 | 99 | 50/20 | 5 | 20/20/20/20/20/20/20/20 |
| fun-18 | Let's block and blow | LEVEL004.DAT | 3 | 35 | 80 | 70/50 | 5 | 0/0/20/20/0/0/0/0 |
| fun-19 | Take good care of my Lemmings | LEVEL005.DAT | 1 | 41 | 20 | 80/56 | 5 | 20/20/20/20/20/20/20/20 |
| fun-20 | We are now at LEMCON ONE | LEVEL006.DAT | 3 | 51 | 10 | 50/30 | 5 | 20/20/20/20/20/20/20/20 |
| fun-21 | You Live and Lem | LEVEL008.DAT | 4 | 68 | 50 | 80/48 | 8 | 20/20/20/20/20/20/20/20 |
| fun-22 | A Beast of a level | LEVEL001.DAT | 3 | — | 50 | 80/64 | 5 | 20/20/20/20/20/20/20/20 |
| fun-23 | I've lost that Lemming feeling | LEVEL004.DAT | 1 | 33 | 50 | 80/20 | 8 | 20/20/20/20/20/20/20/20 |
| fun-24 | Konbanwa Lemming san | LEVEL005.DAT | 7 | 47 | 99 | 30/20 | 5 | 20/20/20/20/20/20/20/20 |
| fun-25 | Lemmings Lemmings everywhere | LEVEL006.DAT | 0 | 48 | 99 | 80/40 | 5 | 20/20/20/20/20/20/20/20 |
| fun-26 | Nightmare on Lem street | LEVEL007.DAT | 1 | 57 | 30 | 2/2 | 5 | 20/20/20/20/20/20/20/20 |
| fun-27 | Let's be careful out there | LEVEL004.DAT | 6 | 38 | 1 | 50/25 | 5 | 20/20/20/20/20/20/20/20 |
| fun-28 | If only they could fly | LEVEL006.DAT | 1 | 49 | 80 | 80/48 | 5 | 30/30/30/30/30/30/30/30 |
| fun-29 | worra lorra lemmings | LEVEL006.DAT | 5 | 53 | 90 | 80/48 | 8 | 30/30/30/30/30/30/30/30 |
| fun-30 | Lock up your Lemmings | LEVEL008.DAT | 2 | 66 | 10 | 60/40 | 5 | 20/20/20/20/20/20/20/20 |

## Tricky

| ID | 제목 | DAT | 구간 | ODD | RR | 수/구출 | 분 | C/F/O/B/U/A/M/D |
|---|---|---|---:|---:|---:|---:|---:|---|
| tricky-01 | This should be a doddle! | LEVEL000.DAT | 0 | — | 50 | 80/40 | 4 | 10/10/10/10/10/10/10/10 |
| tricky-02 | We all fall down | LEVEL001.DAT | 6 | 14 | 1 | 40/40 | 3 | 0/0/0/0/0/0/0/40 |
| tricky-03 | A ladder would be handy | LEVEL002.DAT | 1 | 17 | 50 | 80/40 | 6 | 20/20/20/20/40/20/20/20 |
| tricky-04 | Here's one I prepared earlier | LEVEL003.DAT | 0 | 24 | 55 | 80/16 | 8 | 40/40/40/40/40/40/40/40 |
| tricky-05 | Careless clicking costs lives | LEVEL003.DAT | 1 | 25 | 20 | 80/16 | 5 | 20/20/20/20/20/20/20/20 |
| tricky-06 | Lemmingology | LEVEL003.DAT | 3 | 27 | 50 | 5/4 | 5 | 20/20/20/20/20/20/20/20 |
| tricky-07 | Been there, seen it, done it | LEVEL003.DAT | 4 | 28 | 20 | 75/55 | 5 | 20/20/20/20/20/20/20/20 |
| tricky-08 | Lemming sanctuary in sight | LEVEL004.DAT | 7 | 39 | 40 | 80/48 | 8 | 0/0/0/20/50/0/0/0 |
| tricky-09 | They just keep on coming | LEVEL006.DAT | 2 | 50 | 50 | 75/70 | 5 | 20/20/20/20/20/20/20/20 |
| tricky-10 | There's a lot of them about | LEVEL007.DAT | 3 | 59 | 60 | 80/74 | 8 | 20/20/20/20/20/20/20/20 |
| tricky-11 | Lemmings in the attic | LEVEL007.DAT | 7 | 63 | 20 | 50/42 | 8 | 20/20/20/20/50/20/20/20 |
| tricky-12 | Bitter Lemming | LEVEL008.DAT | 0 | 64 | 1 | 50/40 | 8 | 50/50/20/20/50/20/20/20 |
| tricky-13 | Lemming Drops | LEVEL008.DAT | 3 | 67 | 1 | 80/56 | 8 | 20/20/20/20/50/20/20/20 |
| tricky-14 | MENACING !! | LEVEL000.DAT | 2 | — | 50 | 80/70 | 6 | 10/10/15/10/15/15/15/15 |
| tricky-15 | Ozone friendly Lemmings | LEVEL009.DAT | 1 | 73 | 50 | 10/6 | 5 | 0/0/5/0/0/0/0/0 |
| tricky-16 | Luvly Jubly | LEVEL009.DAT | 3 | 75 | 76 | 50/40 | 5 | 0/0/10/0/0/1/1/0 |
| tricky-17 | Diet Lemmingaid | LEVEL009.DAT | 4 | 76 | 50 | 50/48 | 5 | 0/0/2/0/0/0/0/0 |
| tricky-18 | It's Lemmingentry Watson | LEVEL009.DAT | 5 | 77 | 50 | 10/9 | 5 | 0/1/1/0/2/0/0/1 |
| tricky-19 | Postcard from Lemmingland | LEVEL009.DAT | 7 | 79 | 50 | 50/50 | 5 | 10/10/0/0/1/1/1/1 |
| tricky-20 | One way digging to freedom | LEVEL000.DAT | 3 | — | 60 | 80/76 | 4 | 3/3/6/2/2/4/2/2 |
| tricky-21 | All the 6&#96;s ........ | LEVEL000.DAT | 5 | — | 66 | 66/44 | 6 | 66/66/66/66/66/66/66/66 |
| tricky-22 | Turn around young lemmings! | LEVEL000.DAT | 6 | — | 88 | 80/72 | 5 | 0/1/15/0/20/1/0/0 |
| tricky-23 | From The Boundary Line | LEVEL000.DAT | 7 | — | 99 | 80/48 | 5 | 10/10/0/0/1/2/1/1 |
| tricky-24 | Tightrope City | LEVEL001.DAT | 0 | — | 85 | 80/75 | 2 | 5/5/5/5/3/5/5/5 |
| tricky-25 | Cascade | LEVEL001.DAT | 1 | — | 99 | 80/10 | 5 | 1/10/5/0/10/1/1/5 |
| tricky-26 | I have a cunning plan | LEVEL001.DAT | 2 | — | 99 | 80/80 | 5 | 2/0/5/10/20/2/2/2 |
| tricky-27 | The Island of the Wicker people | LEVEL001.DAT | 4 | — | 60 | 80/72 | 5 | 5/0/5/5/5/5/5/5 |
| tricky-28 | Lost something? | LEVEL001.DAT | 5 | — | 70 | 80/72 | 5 | 10/10/10/10/10/10/10/10 |
| tricky-29 | Rainbow Island | LEVEL002.DAT | 0 | — | 70 | 80/79 | 4 | 10/10/10/10/10/10/10/10 |
| tricky-30 | The Crankshaft | LEVEL001.DAT | 7 | — | 20 | 80/72 | 4 | 10/10/10/10/10/0/10/10 |

## Taxing

| ID | 제목 | DAT | 구간 | ODD | RR | 수/구출 | 분 | C/F/O/B/U/A/M/D |
|---|---|---|---:|---:|---:|---:|---:|---|
| taxing-01 | If at first you don&#96;t succeed.. | LEVEL002.DAT | 2 | — | 40 | 80/79 | 4 | 2/2/2/2/8/2/2/2 |
| taxing-02 | Watch out, there&#96;s traps about | LEVEL002.DAT | 3 | — | 70 | 80/64 | 5 | 10/5/5/10/10/5/5/5 |
| taxing-03 | Heaven can wait (we hope!!!!) | LEVEL002.DAT | 4 | — | 1 | 80/80 | 2 | 30/30/30/0/30/30/30/30 |
| taxing-04 | Lend a helping hand.... | LEVEL002.DAT | 5 | — | 50 | 40/30 | 7 | 0/0/5/4/20/2/2/2 |
| taxing-05 | The Prison! | LEVEL002.DAT | 6 | — | 50 | 60/45 | 5 | 0/0/5/4/20/10/0/3 |
| taxing-06 | Compression Method 1 | LEVEL002.DAT | 7 | — | 99 | 50/30 | 3 | 0/0/3/3/0/5/0/1 |
| taxing-07 | Every Lemming for himself!!! | LEVEL003.DAT | 0 | — | 55 | 80/78 | 3 | 1/0/5/1/6/1/0/0 |
| taxing-08 | The Art Gallery | LEVEL003.DAT | 1 | — | 20 | 80/80 | 4 | 10/10/10/0/10/10/0/0 |
| taxing-09 | Perseverance | LEVEL003.DAT | 2 | — | 50 | 20/20 | 4 | 2/1/1/2/2/1/1/2 |
| taxing-10 | Izzie Wizzie lemmings get busy | LEVEL003.DAT | 3 | — | 50 | 5/5 | 5 | 0/0/5/5/15/5/5/5 |
| taxing-11 | The ascending pillar scenario | LEVEL003.DAT | 4 | — | 20 | 50/50 | 4 | 2/1/0/0/20/5/5/5 |
| taxing-12 | Livin&#96; On The Edge | LEVEL003.DAT | 5 | — | 1 | 10/8 | 5 | 10/10/10/2/10/10/10/10 |
| taxing-13 | Upsidedown World | LEVEL003.DAT | 6 | — | 40 | 80/79 | 6 | 1/0/10/1/2/3/4/2 |
| taxing-14 | Hunt the Nessy.... | LEVEL003.DAT | 7 | — | 30 | 80/76 | 8 | 0/0/10/10/30/2/1/1 |
| taxing-15 | What an AWESOME level | LEVEL000.DAT | 1 | — | 50 | 80/64 | 6 | 20/20/20/20/20/20/20/20 |
| taxing-16 | Mary Poppins&#96; land | LEVEL004.DAT | 0 | — | 50 | 80/77 | 3 | 0/4/3/3/20/0/0/0 |
| taxing-17 | X marks the spot | LEVEL004.DAT | 1 | — | 50 | 80/72 | 7 | 5/0/5/0/20/5/6/0 |
| taxing-18 | Tribute to M.C.Escher | LEVEL004.DAT | 2 | — | 50 | 75/65 | 5 | 0/1/15/0/15/1/0/0 |
| taxing-19 | Bomboozal | LEVEL004.DAT | 3 | — | 80 | 70/64 | 2 | 0/0/15/0/0/0/0/0 |
| taxing-20 | Walk the web rope | LEVEL004.DAT | 4 | — | 50 | 80/70 | 5 | 20/20/20/20/20/20/20/20 |
| taxing-21 | Feel the heat! | LEVEL004.DAT | 5 | — | 70 | 20/20 | 2 | 3/3/3/3/4/3/3/3 |
| taxing-22 | Come on over to my place | LEVEL004.DAT | 6 | — | 1 | 50/40 | 5 | 0/2/0/1/10/1/1/2 |
| taxing-23 | King of the castle | LEVEL004.DAT | 7 | — | 40 | 80/76 | 5 | 0/0/0/0/20/0/0/0 |
| taxing-24 | Take a running jump..... | LEVEL005.DAT | 0 | — | 50 | 80/79 | 3 | 0/0/0/0/30/0/0/0 |
| taxing-25 | Follow the leader... | LEVEL005.DAT | 1 | — | 20 | 80/72 | 4 | 0/0/10/0/7/5/0/0 |
| taxing-26 | Triple Trouble | LEVEL005.DAT | 2 | — | 80 | 80/79 | 5 | 10/10/10/10/12/10/10/10 |
| taxing-27 | Call in the bomb squad | LEVEL005.DAT | 3 | — | 10 | 80/48 | 5 | 0/0/10/10/10/0/0/0 |
| taxing-28 | POOR WEE CREATURES! | LEVEL005.DAT | 4 | — | 1 | 80/56 | 5 | 1/1/4/4/8/3/1/0 |
| taxing-29 | How do I dig up the way? | LEVEL002.DAT | 1 | — | 80 | 80/76 | 4 | 10/10/10/10/5/10/10/10 |
| taxing-30 | We all fall down | LEVEL006.DAT | 7 | — | 1 | 60/60 | 3 | 0/0/0/0/0/0/0/60 |

## Mayhem

| ID | 제목 | DAT | 구간 | ODD | RR | 수/구출 | 분 | C/F/O/B/U/A/M/D |
|---|---|---|---:|---:|---:|---:|---:|---|
| mayhem-01 | Steel Works | LEVEL005.DAT | 5 | — | 15 | 80/72 | 8 | 0/5/10/5/30/0/0/5 |
| mayhem-02 | The Boiler Room | LEVEL005.DAT | 6 | — | 30 | 80/72 | 6 | 10/5/10/10/30/0/0/0 |
| mayhem-03 | It&#96;s hero time! | LEVEL005.DAT | 7 | — | 99 | 25/25 | 1 | 1/1/1/1/1/1/1/1 |
| mayhem-04 | The Crossroads | LEVEL006.DAT | 0 | — | 99 | 50/40 | 1 | 0/0/10/0/0/20/0/0 |
| mayhem-05 | Down, along, up. In that order | LEVEL006.DAT | 1 | — | 80 | 80/60 | 5 | 2/2/10/10/5/1/0/5 |
| mayhem-06 | One way or another | LEVEL006.DAT | 2 | — | 50 | 75/75 | 4 | 0/0/10/0/15/0/5/5 |
| mayhem-07 | Poles Apart | LEVEL006.DAT | 3 | — | 50 | 50/45 | 5 | 1/10/0/0/6/4/0/4 |
| mayhem-08 | Last one out is a rotten egg! | LEVEL006.DAT | 4 | — | 55 | 80/72 | 5 | 5/0/5/5/10/0/5/5 |
| mayhem-09 | Curse of the Pharaohs | LEVEL006.DAT | 5 | — | 90 | 80/79 | 4 | 0/0/20/1/12/5/0/1 |
| mayhem-10 | Pillars of Hercules | LEVEL006.DAT | 6 | — | 1 | 75/50 | 5 | 2/3/4/2/20/4/0/2 |
| mayhem-11 | We all fall down | LEVEL006.DAT | 7 | 55 | 1 | 80/80 | 3 | 0/0/0/0/0/0/0/80 |
| mayhem-12 | The Far Side | LEVEL007.DAT | 0 | — | 50 | 75/75 | 4 | 2/1/0/0/20/5/5/5 |
| mayhem-13 | The Great Lemming Caper | LEVEL007.DAT | 1 | — | 50 | 2/2 | 5 | 1/1/2/2/2/10/10/10 |
| mayhem-14 | Pea Soup | LEVEL007.DAT | 2 | — | 70 | 80/75 | 5 | 0/0/10/10/30/0/0/21 |
| mayhem-15 | The Fast Food Kitchen... | LEVEL007.DAT | 3 | — | 60 | 80/60 | 3 | 10/10/10/10/10/10/10/10 |
| mayhem-16 | Just a Minute... | LEVEL007.DAT | 4 | — | 10 | 50/50 | 1 | 0/1/5/0/0/5/0/5 |
| mayhem-17 | Stepping Stones | LEVEL007.DAT | 5 | — | 50 | 80/70 | 4 | 5/2/0/2/10/5/5/5 |
| mayhem-18 | And then there were four.... | LEVEL007.DAT | 6 | — | 1 | 80/72 | 9 | 2/0/20/10/30/1/2/1 |
| mayhem-19 | Time to get up! | LEVEL007.DAT | 7 | — | 20 | 50/46 | 5 | 2/0/4/0/20/0/0/0 |
| mayhem-20 | No added colours or Lemmings | LEVEL009.DAT | 2 | 74 | 85 | 50/50 | 5 | 2/0/0/1/1/1/2/0 |
| mayhem-21 | With a twist of lemming please | LEVEL008.DAT | 0 | — | 1 | 50/50 | 5 | 1/50/0/0/4/1/0/1 |
| mayhem-22 | A BeastII of a level | LEVEL000.DAT | 4 | — | 75 | 80/68 | 5 | 10/10/10/10/15/10/10/10 |
| mayhem-23 | Going up....... | LEVEL008.DAT | 1 | — | 70 | 80/64 | 9 | 0/1/20/20/30/5/0/0 |
| mayhem-24 | All or Nothing | LEVEL008.DAT | 2 | — | 99 | 50/50 | 1 | 0/0/0/0/0/3/0/0 |
| mayhem-25 | Have a nice day! | LEVEL008.DAT | 3 | — | 1 | 80/72 | 5 | 2/2/0/2/25/1/1/1 |
| mayhem-26 | The Steel Mines of Kessel | LEVEL008.DAT | 4 | — | 50 | 80/60 | 8 | 0/0/20/20/20/0/0/0 |
| mayhem-27 | Just a Minute (Part Two) | LEVEL008.DAT | 5 | — | 10 | 50/50 | 1 | 1/1/5/0/0/5/0/5 |
| mayhem-28 | Mind the step..... | LEVEL008.DAT | 6 | — | 1 | 1/1 | 7 | 0/0/0/0/25/10/0/15 |
| mayhem-29 | Save Me | LEVEL008.DAT | 7 | — | 50 | 80/64 | 5 | 0/0/0/6/15/2/0/2 |
| mayhem-30 | Rendezvous at the Mountain | LEVEL009.DAT | 0 | — | 20 | 80/60 | 9 | 10/1/10/10/30/10/10/4 |

## 특수 그래픽

VGASPEC 식별값은 원본 레코드 기준이며 파일 번호는 값에서 1을 뺍니다.

| 진입점 | 제목 | 파일 |
|---|---|---|
| fun-22 | A Beast of a level | VGASPEC3.DAT |
| tricky-14 | MENACING !! | VGASPEC1.DAT |
| taxing-15 | What an AWESOME level | VGASPEC0.DAT |
| mayhem-22 | A BeastII of a level | VGASPEC2.DAT |

이 프로필의 120개 진입점에는 SuperLemming 플래그가 없습니다. Oh No!, Holiday, 다른 플랫폼 판본과 사용자 제작 레벨은 별도 범위 정의가 필요합니다.
