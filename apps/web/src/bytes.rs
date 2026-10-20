//! Browser-owned flat logical files. No paths, fetches or filesystem access.
use std::{collections::BTreeMap, rc::Rc};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub const MAX_FILE_BYTES: usize = 1_048_576;

#[derive(Default)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct WebAssets {
    pub(crate) files: Rc<BTreeMap<String, Vec<u8>>>,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl WebAssets {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub fn new() -> Self {
        Self::default()
    }

    /// Copy only after validating. A loaded game retains its immutable snapshot.
    pub fn insert(&mut self, name: &str, bytes: &[u8]) -> Result<(), String> {
        let name = name.to_ascii_uppercase();
        if !known(&name) {
            return Err("unsupported logical asset name".into());
        }
        if bytes.len() > MAX_FILE_BYTES {
            return Err(format!("{name}: file exceeds 1 MiB"));
        }
        if self.files.contains_key(&name) {
            return Err(format!("{name}: duplicate asset"));
        }
        Rc::make_mut(&mut self.files).insert(name, bytes.to_vec());
        Ok(())
    }
    pub fn count(&self) -> usize {
        self.files.len()
    }
    pub fn accepts(name: &str) -> bool {
        known(&name.to_ascii_uppercase())
    }
}
fn known(name: &str) -> bool {
    matches!(
        name,
        "MAIN.DAT" | "ODDTABLE.DAT" | "VGALEMMI.EXE" | "ADLIB.DAT"
    ) || (0..10).any(|n| name == format!("LEVEL{n:03}.DAT"))
        || (0..4).any(|n| name == format!("VGASPEC{n}.DAT"))
        || (0..5).any(|n| name == format!("GROUND{n}O.DAT") || name == format!("VGAGR{n}.DAT"))
}
