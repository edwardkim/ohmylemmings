# G11-06 measurement tools

Scope and results: [#106 report](../../mydocs/reports/issue-106_performance.md), [fixed baseline](../../assets/manifests/performance_baseline.json). Original binaries/derived images/PCM stay in ignored `assets/local/`. No dependencies beyond the pinned development tools, SDL and an existing Chromium CDP session are added.

## Native and original-free checks

```sh
source assets/local/dev-tools/env.sh
python3 -m unittest tools.tests.test_performance -v
make check-desktop
python3 tools/performance.py --prepare assets/local/perf-inputs
cargo build -p lemmings-game --example performance_probe --release --locked --offline
# Each output directory must be new. Original directory contains the verified50files.
target/release/examples/performance_probe assets/local/dos-original assets/local/perf-inputs/cases.tsv assets/local/perf-native full > assets/local/perf-native.jsonl
# One file read sample in a fresh process; repeat3warm+30measured with new paths.
target/release/examples/performance_probe assets/local/dos-original read assets/local/perf-read-1 full
# Actual visible/audio-on run: repeat3, use new JSON names, no builds/tests concurrently.
target/release/lemmings-desktop --assets assets/local/dos-original --difficulty mayhem --level 22 --replay assets/local/perf-inputs/mayhem-22.txt --performance assets/local/perf-sdl-1.json --performance-seconds 60
```

`scene` replaces `full` to measure only selected scenes. `synthetic` replaces the asset directory, with TSV `synthetic 1 0 32 -` and mode `smoke`, for the six original-free stages. Each JSONL algorithm row contains33 `[elapsed_ns, operations, digest, initialization_ns]` samples. Remove the first3, call `tools.performance.summarize(values, operations)` for median/nearest-rank p95/max/throughput. Hashing and dumps occur outside elapsed windows. The Rust adapter rejects a tick after completion. PCM is mono44100Hz in512-frame chunks, one second each in full mode.

SDL opt-in JSON records tick `[at_ns,step,cost_ns,late_ns,alive]`, frame `[at_ns,gap_ns,cpu_ns,upload_ns,present_ns]` and audio `[at_ns,queue_frames,playing,rate,quantum,cost_ns]`. Upload samples contain32 updates/clear/copy calls each, with3warmups+30measured. VmHWM includes setup and the recorder. Audio underruns are queue polling observations. Keep normal options unchanged for comparison. Never treat `--fast` capture/check wall time as product FPS.

## Browser preparation and execution

Build a new staging directory after the native command above. This uses the existing build and pinned wasm-bindgen; original bytes and the decoded AdLib image are only copied locally.

```sh
python3 - <<'PY'
from pathlib import Path
import hashlib,json,shutil,subprocess
from tools.web_build import build,TOOLS
out=Path('assets/local/web/performance');build(out)
subprocess.run(['cargo','build','-p','lemmings-audio-probe','--lib','--release','--target','wasm32-unknown-unknown','--locked','--offline'],check=True)
subprocess.run([str(TOOLS/'wasm-bindgen'),'--target','web','--out-dir',str(out/'pcm'),'--out-name','audio_probe','target/wasm32-unknown-unknown/release/lemmings_audio_probe.wasm'],check=True)
native=Path('assets/local/perf-native');sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
cases=json.loads(Path('assets/local/perf-inputs/cases.json').read_text())
for c in cases:
    c['expected']=[sha(native/c['id']/name) for name in ['state.json','terrain.bin','bricks.bin','objects.bin','frame.rgb']]
    c['commands']=[[int(w[0]),int(w[1]),w[2],int(w[3]) if len(w)>3 else 0] for w in map(str.split,c['input_text'].splitlines())]
sounds=[dict(id='combined',music=0,effects=list(range(18)))]+[dict(id=f'music-{i:02}',music=i,effects=[]) for i in range(21)]+[dict(id=f'effect-{i:02}',music=-1,effects=[i]) for i in range(18)]
for s in sounds:s['sha256']=sha(native/(s['id']+'.pcm'))
files=sorted(p.name for p in Path('assets/local/dos-original').iterdir() if p.name.startswith(('LEVEL','GROUND','VGAGR','VGASPEC')) or p.name in ['MAIN.DAT','ADLIB.DAT','VGALEMMI.EXE'])
(out/'files').mkdir(exist_ok=True)
for name in files:shutil.copyfile(Path('assets/local/dos-original')/name,out/'files'/name)
shutil.copyfile(native/'adlib-image.bin',out/'adlib-image.bin')
(out/'plan.json').write_text(json.dumps(dict(cases=cases,sounds=sounds,files=files,levels=[f'{r}-{n:02}' for r in ['fun','tricky','taxing','mayhem'] for n in range(1,31)])))
for name in ['algorithm.js','live.js']:shutil.copyfile(Path('tools/performance')/name,out/name)
(out/'algorithm.html').write_text('<!doctype html><meta charset="utf-8"><p id="status">Measuring</p><canvas id="game" width="320" height="200"></canvas><script type="module" src="algorithm.js"></script>')
(out/'live.html').write_text((out/'index.html').read_text().replace('src="./app.js"','src="./live.js"'))
PY
# Serve assets/local/web on port8080 using the existing local server.
# Use the already authorized Chromium CDP session at127.0.0.1:19222.
node tools/performance/cdp.cjs algorithm assets/local/perf-browser-algorithm
node tools/performance/cdp.cjs live assets/local/perf-browser-live
# If only lifecycle needs repeating, reuse the live target without replaying the three intervals:
node tools/performance/cdp.cjs lifecycle assets/local/perf-browser-live reuse
```

Keep the performance tab visible. `live` activates sound through a trusted click, executes the normal application and records3×60s. A visibility interruption fails the interval and preserves its JSON. Use a fresh output directory and `new 1` for one replacement only; preserve the excluded record/reason. Lifecycle waits for async observations, records failures before asserting and tests real tab visibility separately. CDP requests do not alter remote/firewall policy or install a browser. The algorithm exposes `window.__performanceResult`; live output includes raw per-event arrays and final hashes. Browser PCM/tick/frame/Canvas copies cross the normal JS/WASM bridge; memory and API scope differ from native.

CI uses `ci_shell.py -c COMMAND` as the GNU make shell to append `PERF_COMMAND` JSON while preserving command output and exit status. Wall/child CPU/peak RSS include process startup, compilation+link and check/capture/I/O work. The capture contrast in `--smoke` is one diagnostic pair, not a statistical FPS or isolated disk benchmark.
