//! Borrowed DOS explosion offsets:51 frames,80 signed(dx,dy) pairs per frame.
use core::fmt;
pub const EXPLOSION_PARTICLE_DATA_SIZE: usize = 51 * 80 * 2;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParticleDataError {
    pub actual: usize,
}
impl fmt::Display for ParticleDataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "particle table needs {EXPLOSION_PARTICLE_DATA_SIZE} bytes, got {}",
            self.actual
        )
    }
}
impl core::error::Error for ParticleDataError {}
#[derive(Clone, Copy, Debug)]
pub struct ExplosionParticles<'a> {
    bytes: &'a [u8],
}
impl<'a> ExplosionParticles<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, ParticleDataError> {
        if bytes.len() != EXPLOSION_PARTICLE_DATA_SIZE {
            return Err(ParticleDataError {
                actual: bytes.len(),
            });
        }
        Ok(Self { bytes })
    }
    /// Either -128 component marks an absent particle. No palette or I/O policy.
    pub fn offset(self, frame: usize, particle: usize) -> Option<(i8, i8)> {
        if frame >= 51 || particle >= 80 {
            return None;
        }
        let at = (frame * 80 + particle) * 2;
        let x = self.bytes[at] as i8;
        let y = self.bytes[at + 1] as i8;
        if x == i8::MIN || y == i8::MIN {
            None
        } else {
            Some((x, y))
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn signed_offsets_absent_components_and_last_frame_are_preserved() {
        let mut bytes = [128; EXPLOSION_PARTICLE_DATA_SIZE];
        bytes[..8].copy_from_slice(&[255, 127, 0, 128, 128, 0, 129, 1]);
        bytes[EXPLOSION_PARTICLE_DATA_SIZE - 2..].copy_from_slice(&[1, 255]);
        let view = ExplosionParticles::parse(&bytes).unwrap();
        assert_eq!(view.offset(0, 0), Some((-1, 127)));
        assert_eq!(view.offset(0, 1), None);
        assert_eq!(view.offset(0, 2), None);
        assert_eq!(view.offset(0, 3), Some((-127, 1)));
        assert_eq!(view.offset(50, 79), Some((1, -1)));
        assert_eq!(view.offset(51, 0), None);
        assert_eq!(view.offset(0, 80), None);
        assert_eq!(view.offset(usize::MAX, usize::MAX), None);
    }
    #[test]
    fn truncated_and_extra_particle_bytes_are_rejected() {
        let bytes = [0; EXPLOSION_PARTICLE_DATA_SIZE + 1];
        for n in [
            0,
            EXPLOSION_PARTICLE_DATA_SIZE - 1,
            EXPLOSION_PARTICLE_DATA_SIZE + 1,
        ] {
            assert_eq!(
                ExplosionParticles::parse(&bytes[..n]).err(),
                Some(ParticleDataError { actual: n })
            );
        }
    }
}
