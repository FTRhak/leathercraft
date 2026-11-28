//! Embeds the app icon (`assets/leathercraft.ico`) into the Windows executable, so Explorer,
//! shortcuts and the taskbar show it. Only the desktop binary needs it, so core-only builds
//! (`--no-default-features`) skip the resource compiler.

fn main() {
    println!("cargo:rerun-if-changed=assets/leathercraft.rc");
    println!("cargo:rerun-if-changed=assets/leathercraft.ico");
    let windows = std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows");
    let desktop = std::env::var_os("CARGO_FEATURE_DESKTOP").is_some();
    if windows && desktop {
        embed_resource::compile("assets/leathercraft.rc", embed_resource::NONE)
            .manifest_optional()
            .expect("failed to embed the app icon");
    }
}
