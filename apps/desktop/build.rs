use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=resources/app-icon.ico");
    println!("cargo:rerun-if-changed=build.rs");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let version = env::var("CARGO_PKG_VERSION").expect("Cargo version");
    let numbers: Vec<_> = version.split('-').next().unwrap().split('.').collect();
    let numeric = format!("{},{},{},0", numbers[0], numbers[1], numbers[2]);
    let icon = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .join("resources/app-icon.ico")
        .display()
        .to_string()
        .replace('\\', "/");
    let rc = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("app.rc");
    fs::write(
        &rc,
        format!(
            r#"
101 ICON "{icon}"
1 VERSIONINFO
FILEVERSION {numeric}
PRODUCTVERSION {numeric}
FILEFLAGSMASK 0x3fL
FILEFLAGS 0
FILEOS 0x40004L
FILETYPE 1
FILESUBTYPE 0
BEGIN
    BLOCK "StringFileInfo"
    BEGIN
        BLOCK "040904b0"
        BEGIN
            VALUE "FileDescription", "Oh My Lemmings\0"
            VALUE "FileVersion", "{version}\0"
            VALUE "ProductName", "Oh My Lemmings\0"
            VALUE "ProductVersion", "{version}\0"
            VALUE "OriginalFilename", "OhMyLemmings.exe\0"
            VALUE "LegalCopyright", "Oh My Lemmings contributors (MIT)\0"
        END
    END
    BLOCK "VarFileInfo"
    BEGIN
        VALUE "Translation", 0x0409, 1200
    END
END
"#
        ),
    )
    .expect("Write Windows resources");
    embed_resource::compile(rc, embed_resource::NONE)
        .manifest_required()
        .expect("Windows app icon/version resources must compile");
}
