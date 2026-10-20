//! Host-neutral version-1 snapshots and immutable asset metadata. No I/O.
use crate::{Assets, session::Session};

pub fn state(s: &Session) -> String {
    // Debug values below contain only fixed ASCII field/enum names, numbers,
    // booleans and punctuation, so Rust's quoted String output is valid JSON.
    let actors = format!(
        "{:?}",
        s.actors.iter().map(|a| a.state()).collect::<Vec<_>>()
    );
    let inputs = format!("{:?}", s.inputs);
    let n = s.scheduler.nuke_state();
    let nuke = format!(
        "{{\"is_nuked\":{},\"assigning\":{},\"next_index\":{},\"max_lemmings\":{}}}",
        n.is_nuked, n.assigning, n.next_index, n.max_lemmings
    );
    format!(
        "{{\"version\":1,\"nuke\":{nuke},\"session\":{},\"digger\":{},\"builder\":{},\"blocker\":{},\"bomber\":{},\"climber\":{},\"floater\":{},\"exit\":{},\"hazards\":{},\"completion\":{},\"remaining_bashers\":{},\"remaining_miners\":{},\"actor_details\":{actors:?},\"inputs\":{inputs:?}}}",
        s.snapshot(),
        s.digger_snapshot(),
        s.builder_snapshot(),
        s.blocker_snapshot(),
        s.bomber_snapshot(),
        s.climber_snapshot(),
        s.floater_snapshot(),
        s.exit_snapshot(),
        s.hazards_snapshot(),
        s.completion_snapshot(),
        s.remaining_bashers,
        s.remaining_miners,
    )
}

pub fn asset_info(a: &Assets) -> String {
    let entrances: Vec<_> = a.entrances.iter().map(|p| [p.x, p.y]).collect();
    let required = a
        .required
        .map_or_else(|| "null".to_owned(), |n| n.to_string());
    let skills = [
        a.climbers, a.floaters, a.bombers, a.blockers, a.builders, a.bashers, a.miners, a.diggers,
    ];
    format!(
        "{{\"width\":{},\"height\":{},\"minutes\":{},\"total\":{},\"rate\":{},\"required\":{required},\"skills\":{skills:?},\"entrances\":{entrances:?},\"screen_x\":{},\"palette\":{:?},\"music_special\":{},\"title\":{:?}}}",
        a.width,
        a.height,
        a.config.time_limit_minutes,
        a.config.total_lemmings,
        a.config.initial_release_rate,
        a.screen_x,
        a.palette,
        a.music_special,
        a.title
    )
}

pub fn materials(a: &Assets) -> Vec<u8> {
    a.steel
        .iter()
        .zip(&a.one_way_left)
        .zip(&a.one_way_right)
        .map(|((&s, &l), &r)| {
            if s != 0 {
                137
            } else if l != 0 {
                135
            } else if r != 0 {
                136
            } else {
                0
            }
        })
        .collect()
}
