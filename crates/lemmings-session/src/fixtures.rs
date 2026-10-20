//! Private geometry for the migrated session tests; no resource/render dependency.
//! Matches the synthetic data used by the former desktop Session tests.
use lemmings_core::*;

pub struct Fixture {
    pub width: usize,
    pub height: usize,
    pub terrain: Vec<u8>,
    pub entrances: Vec<EntrancePoint>,
    pub config: StartupConfig,
    pub object_map: Vec<u8>,
    pub explosion_mask: [u8; 44],
    pub bash_masks: [u8; 160],
    pub mine_masks: [u8; 104],
    pub traps: TrapStates,
}
impl Fixture {
    pub fn synthetic() -> Self {
        let (width, height) = (640, 160);
        let mut bash_masks = [0; 160];
        for direction in 0..2 {
            for frame in 0..4 {
                for y in 0..10 {
                    for x in 0..16 {
                        let selected = if direction == 0 {
                            x == 8 + frame * 2 || x == 9 + frame * 2
                        } else {
                            x == 6 - frame * 2 || x == 7 - frame * 2
                        };
                        if selected {
                            bash_masks[(direction * 4 + frame) * 20 + y * 2 + x / 8] |=
                                128 >> (x % 8);
                        }
                    }
                }
            }
        }
        Self {
            width,
            height,
            terrain: (0..width * height)
                .map(|i| u8::from(i / width >= 100 && i % width < 260))
                .collect(),
            entrances: vec![EntrancePoint { x: 40, y: 40 }],
            config: StartupConfig {
                time_limit_minutes: 5,
                total_lemmings: 10,
                initial_release_rate: 50,
                entrance_count: 1,
                pause_glitch: true,
            },
            object_map: vec![OBJECT_NONE; DOS_OBJECT_MAP_WIDTH * DOS_OBJECT_MAP_HEIGHT],
            explosion_mask: [255; 44],
            bash_masks,
            mine_masks: [255; 104],
            traps: [None; 16],
        }
    }
    pub fn synthetic_basher() -> Self {
        let mut scene = Self::synthetic();
        for y in 70..100 {
            scene.terrain[y * scene.width + 80..y * scene.width + 150].fill(1);
            scene.terrain[y * scene.width + 5..y * scene.width + 50].fill(1);
        }
        scene
    }
    pub fn synthetic_builder() -> Self {
        let mut scene = Self::synthetic();
        for y in 0..scene.height {
            for x in 0..scene.width {
                scene.terrain[y * scene.width + x] = u8::from(
                    (y >= 100 && x < 500) || ((60..100).contains(&y) && (160..170).contains(&x)),
                );
            }
        }
        scene
    }
    pub fn synthetic_exit(drop: bool) -> Self {
        let mut scene = Self::synthetic();
        let floor = if drop { 140 } else { 100 };
        if drop {
            scene.entrances = vec![EntrancePoint { x: 40, y: 0 }];
        }
        for y in 0..scene.height {
            for x in 0..scene.width {
                scene.terrain[y * scene.width + x] = u8::from(y >= floor && x < 500);
            }
        }
        let mut map = ObjectMapMut::new(
            DOS_OBJECT_MAP_WIDTH,
            DOS_OBJECT_MAP_HEIGHT,
            -16,
            -16,
            &mut scene.object_map,
        )
        .unwrap();
        for y in if drop { 60..144 } else { 96..104 } {
            for x in if drop { 64..72 } else { 96..104 } {
                map.write(x, y, OBJECT_EXIT);
            }
        }
        scene
    }
    pub fn synthetic_hazards(kind: u8, drop: bool) -> Self {
        let mut scene = Self::synthetic_exit(drop);
        scene.object_map.fill(OBJECT_NONE);
        let code = if kind < 128 { 0 } else { kind };
        let mut map = ObjectMapMut::new(
            DOS_OBJECT_MAP_WIDTH,
            DOS_OBJECT_MAP_HEIGHT,
            -16,
            -16,
            &mut scene.object_map,
        )
        .unwrap();
        for y in if drop { 60..144 } else { 96..104 } {
            for x in if drop { 64..72 } else { 96..104 } {
                map.write(x, y, code);
            }
        }
        if kind < 128 {
            scene.traps[0] = Some(
                TrapState::new(if kind == 1 { 48 } else { 12 }, 0)
                    .unwrap()
                    .with_continuous(kind == 2),
            );
        }
        scene
    }
}
