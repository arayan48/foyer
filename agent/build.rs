// Logo Foyer sur l'exécutable Windows de l'agent.
fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut r = winresource::WindowsResource::new();
        r.set_icon("../app/src-tauri/icons/icon.ico");
        r.compile().expect("icône Windows");
    }
}
