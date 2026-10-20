// Exercise the exported WebGame command queue in actual WASM; no device I/O.
export function checkSoundCommands(WebGame, assert, throws) {
    const game = new WebGame('builder', 'reference');
    game.step(55);
    const before = game.state(), frame = game.frame();
    let batch = JSON.parse(game.drain_sound_events());
    assert(batch.generation === 0 && batch.reset && batch.dropped === 0, 'initial sound generation');
    assert(JSON.stringify(batch.events.map(e => [e.iteration, e.command])) ===
        JSON.stringify([[15,['effect',2]],[35,['effect',1]],[55,['music',5]]]), 'startup sound order');
    assert(game.state() === before && game.frame().every((v,i) => v === frame[i]), 'sound drain changed simulation/frame');
    assert(JSON.parse(game.drain_sound_events()).events.length === 0, 'sound drain replayed events');
    game.reset('builder','reference');
    batch = JSON.parse(game.drain_sound_events());
    assert(batch.generation === 1 && batch.reset && batch.events.length === 0, 'reset retained old sounds');
    game.command('Pause',0);
    game.key('KeyD',false);game.key('KeyD',true);game.release_input();game.step(1);
    batch = JSON.parse(game.drain_sound_events());
    assert(batch.events.length === 1 && batch.events[0].cue === 'SkillButtonSelect' &&
        batch.events[0].iteration === 0, 'paused/repeated skill selection');
    game.key('KeyD',false);
    const pending = game.sound_state(), state = game.state();
    throws(() => game.reset('unknown','reference'), 'bad reset accepted');
    assert(game.sound_state() === pending && game.state() === state, 'failed reset changed sound/state');
    game.step(1);game.drain_sound_events();
    for (let i=0;i<1100;i++) game.key('KeyD',false);
    game.step(1);batch = JSON.parse(game.drain_sound_events());
    assert(batch.events.length === 1024 && batch.dropped === 76 &&
        batch.events.every(e => e.iteration === 0), 'UI burst queue/drop bound');
    game.free();
    const win = new WebGame('win','reference');
    win.step(1000);batch = JSON.parse(win.drain_sound_events());
    assert(batch.events.at(-1).command[0] === 'stop', 'result did not stop sound');
    win.step(1000);
    assert(JSON.parse(win.drain_sound_events()).events.length === 0, 'finished repeated sound');
    win.transition('continue');batch = JSON.parse(win.drain_sound_events());
    assert(batch.generation === 1 && batch.reset && batch.music === 6, 'success music/generation');
    win.restart_level();batch = JSON.parse(win.drain_sound_events());
    assert(batch.generation === 2 && batch.reset && batch.music === 5, 'retry music/generation');
    win.free();
    return {startup_events:3,ui_burst:1100,retained:1024,dropped:76,lifecycle:true,simulation_unchanged:true};
}
