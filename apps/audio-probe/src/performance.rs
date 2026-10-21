//! Owned streaming adapter for timing the existing PCM engine through WASM.
use lemmings_audio::{AdlibPcm, AudioRate, MAX_PCM_FRAMES, PcmContinuation};
use lemmings_resources::{AdlibImage, AdlibWrite};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct PerformancePcm {
    bytes: Vec<u8>,
    state: Option<PcmContinuation>,
    scratch: Vec<AdlibWrite>,
}
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PerformancePcm {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub fn new(data: &[u8], music: i32, effects: &[u8]) -> Result<Self, String> {
        if !(-1..=20).contains(&music) || effects.iter().any(|&i| i >= 18) {
            return Err("invalid sound index".into());
        }
        let bytes = data.to_vec();
        let mut engine = AdlibPcm::new(
            AdlibImage::parse(&bytes).map_err(|e| e.to_string())?,
            44100,
            AudioRate::DOS_VGA,
        )
        .map_err(|e| e.to_string())?;
        if music >= 0 {
            engine
                .request_music(music as usize)
                .map_err(|e| e.to_string())?;
        }
        for &i in effects {
            engine
                .request_effect(usize::from(i))
                .map_err(|e| e.to_string())?;
        }
        let state = Some(engine.into_continuation());
        Ok(Self {
            bytes,
            state,
            scratch: vec![AdlibWrite::default(); 4096],
        })
    }
    pub fn render(&mut self, frames: u32) -> Result<Vec<i16>, String> {
        if frames as usize > MAX_PCM_FRAMES {
            return Err("chunk exceeds bound".into());
        }
        let image = AdlibImage::parse(&self.bytes).map_err(|e| e.to_string())?;
        let mut engine = AdlibPcm::resume(image, self.state.take().expect("owned continuation"));
        let mut output = vec![0; frames as usize];
        let result = engine.render(&mut output, &mut self.scratch, 4096);
        self.state = Some(engine.into_continuation());
        result.map_err(|e| e.to_string())?;
        Ok(output)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn streaming_benchmark_keeps_pcm_independent_of_chunk_boundaries() {
        let image = lemmings_game::audio::demo_image();
        let mut a = PerformancePcm::new(&image, 0, &[0]).unwrap();
        let mut b = PerformancePcm::new(&image, 0, &[0]).unwrap();
        let whole = a.render(4096).unwrap();
        let parts = (0..8)
            .flat_map(|_| b.render(512).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(whole, parts);
    }
    #[test]
    fn benchmark_rejects_bad_assets_ids_and_oversized_chunks() {
        assert!(PerformancePcm::new(&[], 0, &[]).is_err());
        let image = lemmings_game::audio::demo_image();
        assert!(PerformancePcm::new(&image, 21, &[]).is_err());
        assert!(PerformancePcm::new(&image, 0, &[18]).is_err());
        let mut p = PerformancePcm::new(&image, 0, &[]).unwrap();
        assert!(p.render(8193).is_err());
    }
}
