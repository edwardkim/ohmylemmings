//! Verification adapter only: library PCM versus independent direct OPL delivery.
mod game_sound;
pub use game_sound::trace as game_sound_trace;
use lemmings_audio::{AdlibPcm, AudioRate, MAX_PCM_FRAMES};
use lemmings_opl::Opl2;
use lemmings_resources::{AdlibCommand, AdlibImage, AdlibPlayer, AdlibWrite};

fn actions(sequence: &str) -> Result<Vec<(&str, usize)>, String> {
    let mut result = Vec::new();
    let mut total = 0_usize;
    for action in sequence.split(',') {
        let (command, count) = action.split_once(':').ok_or("missing action count")?;
        if command.is_empty() {
            return Err("empty action".into());
        }
        let count: usize = count.parse().map_err(|_| "bad action count")?;
        total = total.checked_add(count).ok_or("frame total overflow")?;
        if total > 16_000_000 {
            return Err("probe frame bound exceeded".into());
        }
        result.push((command, count));
    }
    Ok(result)
}
fn index(command: &str) -> Result<usize, String> {
    command
        .get(1..)
        .ok_or("bad command")?
        .parse()
        .map_err(|_| "bad index".into())
}
fn parts(partitions: &[u32]) -> Result<(), String> {
    if partitions.is_empty()
        || partitions
            .iter()
            .any(|&v| v == 0 || v as usize > MAX_PCM_FRAMES)
    {
        Err("invalid partitions".into())
    } else {
        Ok(())
    }
}

pub fn render(
    data: &[u8],
    sample_rate: u32,
    num: u32,
    den: u32,
    sequence: &str,
    partitions: &[u32],
) -> Result<Vec<i16>, String> {
    parts(partitions)?;
    let actions = actions(sequence)?;
    let image = AdlibImage::parse(data).map_err(|e| e.to_string())?;
    let mut pcm = AdlibPcm::new(
        image,
        sample_rate,
        AudioRate::new(num, den).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let mut output = Vec::new();
    let mut scratch = [AdlibWrite::default(); 4096];
    let mut chunk = 0;
    for (command, count) in actions {
        match command.as_bytes()[0] {
            b'm' => pcm.apply(AdlibCommand::Music(
                u8::try_from(index(command)?).map_err(|_| "music index exceeds byte")?,
            )),
            b'e' => pcm.apply(AdlibCommand::Effect(
                u8::try_from(index(command)?).map_err(|_| "effect index exceeds byte")?,
            )),
            b'r' => pcm.reset_music(index(command)?),
            b's' if command == "s" => pcm.apply(AdlibCommand::Stop),
            b'n' if command == "n" => Ok(()),
            _ => return Err("unknown command".into()),
        }
        .map_err(|e| e.to_string())?;
        let begin = output.len();
        output.resize(begin + count, 0);
        let mut at = begin;
        while at < output.len() {
            let len = (partitions[chunk % partitions.len()] as usize).min(output.len() - at);
            pcm.render(&mut output[at..at + len], &mut scratch, 4096)
                .map_err(|e| e.to_string())?;
            at += len;
            chunk += 1;
        }
    }
    Ok(output)
}
/// Separate reference path: unmodified OPL API with absolute rational sample
/// deadlines, no AudioPcm, interval-remainder recurrence or render preflight.
pub fn direct(
    data: &[u8],
    sample_rate: u32,
    num: u32,
    den: u32,
    sequence: &str,
) -> Result<Vec<i16>, String> {
    AudioRate::new(num, den).map_err(|e| e.to_string())?;
    if !(8000..=192000).contains(&sample_rate) {
        return Err("invalid sample rate".into());
    }
    let actions = actions(sequence)?;
    let image = AdlibImage::parse(data).map_err(|e| e.to_string())?;
    let mut driver = AdlibPlayer::new(image);
    let mut chip = Opl2::new(sample_rate).map_err(|e| e.to_string())?;
    let mut writes = [AdlibWrite::default(); 4096];
    let mut output = Vec::new();
    let mut frame = 0_u64;
    let mut call = 0_u64;
    for (command, count) in actions {
        match command.as_bytes()[0] {
            b'm' => driver
                .request_music(index(command)?)
                .map_err(|e| e.to_string())?,
            b'e' => driver
                .request_effect(index(command)?)
                .map_err(|e| e.to_string())?,
            b'r' => {
                let i = index(command)?;
                image.music(i).map_err(|e| e.to_string())?;
                driver = AdlibPlayer::new(image);
                driver.request_music(i).map_err(|e| e.to_string())?;
                chip = Opl2::new(sample_rate).map_err(|e| e.to_string())?;
                frame = 0;
                call = 0;
            }
            b's' if command == "s" => driver.request_stop(),
            b'n' if command == "n" => (),
            _ => return Err("unknown command".into()),
        }
        for _ in 0..count {
            if u128::from(frame)
                >= u128::from(call) * u128::from(sample_rate) * u128::from(den) / u128::from(num)
            {
                let n = driver.step(&mut writes, 4096).map_err(|e| e.to_string())?;
                for w in &writes[..n] {
                    chip.write_reg(w.register, w.value);
                }
                call += 1;
            }
            let (l, _r) = chip.render_frame();

            output.push(l.clamp(-32768, 32767) as i16);
            frame += 1;
        }
    }
    Ok(output)
}
#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;
    #[wasm_bindgen]
    pub fn game_sound_trace(partitions: &[u32]) -> Result<String, JsValue> {
        super::game_sound_trace(partitions).map_err(|e| JsValue::from_str(&e))
    }
    #[wasm_bindgen]
    pub fn audio_pcm(
        data: &[u8],
        sample_rate: u32,
        num: u32,
        den: u32,
        sequence: &str,
        partitions: &[u32],
    ) -> Result<Vec<i16>, JsValue> {
        super::render(data, sample_rate, num, den, sequence, partitions)
            .map_err(|e| JsValue::from_str(&e))
    }
}
