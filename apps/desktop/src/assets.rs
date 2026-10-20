//! Filesystem adapter for shared game assets.
use lemmings_game::MissingAsset;
pub use lemmings_game::assets::Assets;
use std::{error::Error, fs, io, path::Path};

pub fn load(root: &Path) -> Result<Assets, Box<dyn Error>> {
    load_level(root, 1)
}
pub fn load_level(root: &Path, number: u8) -> Result<Assets, Box<dyn Error>> {
    let mut source = |name: &str| -> Result<Option<Vec<u8>>, Box<dyn Error>> {
        let path = root.join(name);
        match fs::read(&path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => {
                Err(io::Error::new(error.kind(), format!("{}: {error}", path.display())).into())
            }
        }
    };
    Assets::from_source(&mut source, number).map_err(|error| {
        if let Some(missing) = error.downcast_ref::<MissingAsset>() {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "{}: required asset missing",
                    root.join(&missing.name).display()
                ),
            )
            .into()
        } else {
            error
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_or_truncated_original_files_report_the_path() {
        let directory =
            std::env::temp_dir().join(format!("lemmings-assets-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let error = load(&directory).err().unwrap().to_string();
        assert!(error.contains("LEVEL009.DAT"));
        fs::write(directory.join("LEVEL009.DAT"), [0; 4]).unwrap();
        assert!(load(&directory).is_err());
        fs::remove_dir_all(directory).unwrap();
    }
}
