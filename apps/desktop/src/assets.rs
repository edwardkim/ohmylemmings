//! Filesystem adapter for shared game assets.
use lemmings_core::{Difficulty, LevelId};
use lemmings_game::MissingAsset;
pub use lemmings_game::assets::Assets;
use std::{
    error::Error,
    fs, io,
    path::{Path, PathBuf},
};

pub fn bundled_directory(executable: &Path) -> PathBuf {
    executable
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("data")
}

pub fn macos_bundle_directory(executable: &Path) -> Option<PathBuf> {
    let directory = executable.parent()?;
    let contents = directory.parent()?;
    if directory.file_name()? == "MacOS" && contents.file_name()? == "Contents" {
        Some(contents.join("Resources/data"))
    } else {
        None
    }
}

pub fn load(root: &Path) -> Result<Assets, Box<dyn Error>> {
    load_level(root, 1)
}
pub fn load_level(root: &Path, number: u8) -> Result<Assets, Box<dyn Error>> {
    load_id(root, LevelId::new(Difficulty::Fun, number)?)
}
pub fn load_id(root: &Path, level: LevelId) -> Result<Assets, Box<dyn Error>> {
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
    Assets::from_level_source(&mut source, level).map_err(|error| {
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
    fn macos_bundle_uses_resources_and_development_keeps_its_default() {
        let app = Path::new("한글 game/OhMyLemmings.app/Contents/MacOS/OhMyLemmings");
        assert_eq!(
            macos_bundle_directory(app),
            Some(PathBuf::from(
                "한글 game/OhMyLemmings.app/Contents/Resources/data"
            ))
        );
        assert_eq!(
            macos_bundle_directory(Path::new("target/release/lemmings-desktop")),
            None
        );
        assert_eq!(
            macos_bundle_directory(Path::new("other/MacOS/OhMyLemmings")),
            None
        );
    }
    #[test]
    fn bundled_assets_resolve_beside_the_executable() {
        let executable = Path::new("installation folder")
            .join("한글")
            .join("OhMyLemmings.exe");
        assert_eq!(
            bundled_directory(&executable),
            executable.parent().unwrap().join("data")
        );
    }
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
