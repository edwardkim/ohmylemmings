//! Native-only file selection. The host owns modal time; the game never sees it.
use sdl2::{
    event::Event,
    keyboard::{Keycode, Mod},
    video::{FullscreenType, Window},
};
use std::path::{Path, PathBuf};

pub fn requested(event: &Event) -> bool {
    let Event::KeyDown {
        keycode: Some(key),
        keymod,
        repeat: false,
        ..
    } = event
    else {
        return false;
    };
    let modifiers = *keymod
        & (Mod::LSHIFTMOD
            | Mod::RSHIFTMOD
            | Mod::LCTRLMOD
            | Mod::RCTRLMOD
            | Mod::LALTMOD
            | Mod::RALTMOD
            | Mod::LGUIMOD
            | Mod::RGUIMOD);
    match *key {
        Keycode::F4 => modifiers.is_empty(),
        Keycode::O => {
            !modifiers.is_empty()
                && (modifiers & !(Mod::LCTRLMOD | Mod::RCTRLMOD | Mod::LGUIMOD | Mod::RGUIMOD))
                    .is_empty()
        }
        _ => false,
    }
}

pub fn pick(window: &mut Window, current: Option<&Path>) -> Result<Option<PathBuf>, String> {
    let fullscreen = window.fullscreen_state();
    if fullscreen != FullscreenType::Off {
        window.set_fullscreen(FullscreenType::Off)?;
    }
    let mut dialog = rfd::FileDialog::new()
        .set_parent(window)
        .set_title("Choose MP3 background music")
        .add_filter("MP3 audio", &["mp3", "MP3"]);
    if let Some(folder) = current.and_then(Path::parent).filter(|p| p.is_dir()) {
        dialog = dialog.set_directory(folder);
    }
    let selected = dialog.pick_file();
    let restored = if fullscreen != FullscreenType::Off {
        window.set_fullscreen(fullscreen)
    } else {
        Ok(())
    };
    window.raise();
    restored.map(|_| selected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdl2::keyboard::{Keycode, Mod};

    fn key(keycode: Keycode, keymod: Mod, repeat: bool) -> sdl2::event::Event {
        sdl2::event::Event::KeyDown {
            timestamp: 0,
            window_id: 1,
            keycode: Some(keycode),
            scancode: None,
            keymod,
            repeat,
        }
    }

    #[test]
    fn open_shortcuts_accept_caps_lock_and_both_control_keys() {
        for (k, m) in [
            (Keycode::F4, Mod::NOMOD),
            (Keycode::F4, Mod::CAPSMOD),
            (Keycode::O, Mod::LCTRLMOD),
            (Keycode::O, Mod::RCTRLMOD | Mod::NUMMOD),
            (Keycode::O, Mod::LGUIMOD),
        ] {
            assert!(requested(&key(k, m, false)));
        }
    }

    #[test]
    fn alt_f4_plain_o_and_repeated_keys_never_open_a_dialog() {
        for (k, m, repeat) in [
            (Keycode::F4, Mod::LALTMOD, false),
            (Keycode::F4, Mod::RALTMOD, false),
            (Keycode::F4, Mod::NOMOD, true),
            (Keycode::O, Mod::LCTRLMOD, true),
            (Keycode::O, Mod::NOMOD, false),
            (Keycode::O, Mod::LCTRLMOD | Mod::LSHIFTMOD, false),
        ] {
            assert!(!requested(&key(k, m, repeat)));
        }
    }
}
