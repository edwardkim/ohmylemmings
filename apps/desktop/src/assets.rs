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

pub fn linux_directory(executable: &Path) -> PathBuf {
    let bundled = bundled_directory(executable);
    if bundled.exists() {
        bundled
    } else {
        "assets/local/dos-original".into()
    }
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

pub fn presentation_root(executable: &Path) -> PathBuf {
    macos_bundle_directory(executable)
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| {
            executable
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf()
        })
}

pub fn packaged_hd_directory(executable: &Path) -> Result<Option<PathBuf>, io::Error> {
    let root = presentation_root(executable);
    let marker = root.join("HD-PRESENTATION");
    match fs::read(&marker) {
        Ok(data) if data == b"hd\n" => Ok(Some(root.join("hd-assets"))),
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{}: invalid HD presentation profile", marker.display()),
        )),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
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
    fn packaged_hd_is_independent_of_cwd_and_requires_a_valid_profile() {
        let root = std::env::temp_dir().join(format!("lemmings-hd-{}", std::process::id()));
        for executable in [
            root.join("한글 game/OhMyLemmings.exe"),
            root.join("한글/OhMyLemmings.app/Contents/MacOS/OhMyLemmings"),
        ] {
            let resources = presentation_root(&executable);
            fs::create_dir_all(&resources).unwrap();
            assert_eq!(packaged_hd_directory(&executable).unwrap(), None);
            fs::write(resources.join("HD-PRESENTATION"), b"hd\n").unwrap();
            assert_eq!(
                packaged_hd_directory(&executable).unwrap(),
                Some(resources.join("hd-assets"))
            );
            fs::write(resources.join("HD-PRESENTATION"), b"bad").unwrap();
            assert!(packaged_hd_directory(&executable).is_err());
        }
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn linux_bundle_uses_sibling_data_and_preserves_unbundled_development() {
        let root = std::env::temp_dir().join(format!("lemmings-linux-{}", std::process::id()));
        let executable = root.join("한글 game/lemmings-desktop");
        let data = executable.parent().unwrap().join("data");
        assert_eq!(
            linux_directory(&executable),
            PathBuf::from("assets/local/dos-original")
        );
        fs::create_dir_all(&data).unwrap();
        assert_eq!(linux_directory(&executable), data);
        fs::remove_dir_all(root).unwrap();
    }
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
