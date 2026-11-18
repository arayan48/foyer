/// Jeton de l'agent installé sur cette même machine, s'il existe.
#[tauri::command]
fn local_token() -> Option<String> {
  let data = std::env::var("FOYER_DATA").unwrap_or_else(|_| {
    if cfg!(windows) { r"C:\Foyer".into() } else { "/var/lib/foyer".into() }
  });
  std::fs::read_to_string(std::path::Path::new(&data).join("token"))
    .ok()
    .map(|t| t.trim().to_string())
    .filter(|t| !t.is_empty())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_updater::Builder::new().build())
    .plugin(tauri_plugin_process::init())
    .invoke_handler(tauri::generate_handler![local_token])
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while building tauri application");
}
