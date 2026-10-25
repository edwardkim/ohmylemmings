//! Startup renderer selection; SDL detection/I/O stays in the desktop adapter.
use sdl2::{render::Canvas, video::Window};

const ACCELERATED: u32 = sdl2::sys::SDL_RendererFlags::SDL_RENDERER_ACCELERATED as u32;
const SOFTWARE: u32 = sdl2::sys::SDL_RendererFlags::SDL_RENDERER_SOFTWARE as u32;

#[derive(Clone, Copy)]
struct Driver {
    index: u32,
    name: &'static str,
    accelerated: bool,
}
struct Selection<T> {
    value: T,
    accelerated: bool,
    reason: String,
}

/// A listed backend is only a candidate: initialization must succeed on this window.
fn select<T>(
    drivers: &[Driver],
    preferred: Option<&str>,
    mut create: impl FnMut(Option<u32>) -> Result<T, String>,
) -> Result<Selection<T>, String> {
    let forced_software = preferred.is_some_and(|name| name.eq_ignore_ascii_case("software"));
    let mut errors = Vec::new();
    if !forced_software {
        let mut candidates: Vec<_> = drivers.iter().filter(|driver| driver.accelerated).collect();
        candidates.sort_by_key(|driver| {
            !preferred.is_some_and(|name| name.eq_ignore_ascii_case(driver.name))
        });
        for driver in candidates {
            match create(Some(driver.index)) {
                Ok(value) => {
                    return Ok(Selection {
                        value,
                        accelerated: true,
                        reason: errors.join("; "),
                    });
                }
                Err(error) => errors.push(format!("{}: {error}", driver.name)),
            }
        }
    }
    let reason = if forced_software {
        "software requested".to_owned()
    } else if errors.is_empty() {
        "no accelerated renderer".to_owned()
    } else {
        errors.join("; ")
    };
    let value = create(None).map_err(|error| format!("{reason}; software: {error}"))?;
    Ok(Selection {
        value,
        accelerated: false,
        reason,
    })
}

pub fn create(window: Window, video_driver: &str) -> Result<Canvas<Window>, String> {
    let available: Vec<_> = sdl2::render::drivers().enumerate().collect();
    let drivers: Vec<_> = available
        .iter()
        .map(|(index, info)| Driver {
            index: *index as u32,
            name: info.name,
            accelerated: info.flags & ACCELERATED != 0,
        })
        .collect();
    let software = available
        .iter()
        .find(|(_, info)| info.flags & SOFTWARE != 0)
        .map(|(index, _)| *index as u32);
    let preferred = sdl2::hint::get("SDL_RENDER_DRIVER");
    let selected = select(&drivers, preferred.as_deref(), |index| {
        // Keep the Window alive when a consuming CanvasBuilder fails. Each failed
        // canvas is dropped before trying another backend on the same SDL window.
        let mut builder = window.clone().into_canvas();
        let flag = if let Some(index) = index {
            builder = builder.index(index).accelerated();
            ACCELERATED
        } else {
            builder = builder.software();
            if let Some(index) = software {
                builder = builder.index(index);
            }
            SOFTWARE
        };
        let canvas = builder.build().map_err(|error| error.to_string())?;
        if canvas.info().flags & flag == 0 {
            return Err("renderer did not provide the requested acceleration mode".to_owned());
        }
        Ok(canvas)
    })?;
    let info = selected.value.info();
    eprintln!(
        "LEMMINGS_RENDERER mode={} video_driver={video_driver:?} renderer={:?} flags={} max_texture={}x{} reason={:?}",
        if selected.accelerated {
            "accelerated"
        } else {
            "software"
        },
        info.name,
        info.flags,
        info.max_texture_width,
        info.max_texture_height,
        selected.reason,
    );
    Ok(selected.value)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn drivers() -> Vec<Driver> {
        vec![
            Driver {
                index: 0,
                name: "software",
                accelerated: false,
            },
            Driver {
                index: 1,
                name: "opengl",
                accelerated: true,
            },
            Driver {
                index: 2,
                name: "opengles2",
                accelerated: true,
            },
        ]
    }
    #[test]
    fn acceleration_is_preferred_even_when_software_is_listed_first() {
        let result = select(&drivers(), None, Ok).unwrap();
        assert_eq!(result.value, Some(1));
        assert!(result.accelerated);
        assert!(result.reason.is_empty());
    }
    #[test]
    fn failed_accelerated_driver_does_not_hide_another_usable_driver() {
        let mut attempts = Vec::new();
        let result = select(&drivers(), None, |index| {
            attempts.push(index);
            if index == Some(1) {
                Err("device lost".to_owned())
            } else {
                Ok(index)
            }
        })
        .unwrap();
        assert_eq!(attempts, [Some(1), Some(2)]);
        assert_eq!(result.value, Some(2));
        assert!(result.accelerated);
    }
    #[test]
    fn all_accelerated_initializations_failing_falls_back_with_reason() {
        let result = select(&drivers(), None, |index| {
            if index.is_some() {
                Err("no device".to_owned())
            } else {
                Ok(index)
            }
        })
        .unwrap();
        assert_eq!(result.value, None);
        assert!(!result.accelerated);
        assert!(result.reason.contains("opengl: no device"));
        assert!(result.reason.contains("opengles2: no device"));
    }
    #[test]
    fn missing_acceleration_uses_software() {
        let result = select(&drivers()[..1], None, Ok).unwrap();
        assert_eq!(result.value, None);
        assert!(!result.accelerated);
        assert_eq!(result.reason, "no accelerated renderer");
    }
    #[test]
    fn explicit_software_request_does_not_probe_acceleration() {
        let result = select(&drivers(), Some("SOFTWARE"), |index| {
            assert!(index.is_none());
            Ok(index)
        })
        .unwrap();
        assert!(!result.accelerated);
        assert_eq!(result.reason, "software requested");
    }
    #[test]
    fn preferred_accelerated_driver_is_tried_first() {
        let result = select(&drivers(), Some("OPENGLES2"), Ok).unwrap();
        assert_eq!(result.value, Some(2));
    }
    #[test]
    fn complete_failure_reports_accelerated_and_software_errors() {
        let result = select::<()>(&drivers(), None, |_| Err("unavailable".to_owned()));
        let error = result.err().unwrap();
        assert!(error.contains("opengl: unavailable"));
        assert!(error.contains("software: unavailable"));
    }
}
