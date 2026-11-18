use std::sync::atomic::{AtomicBool, Ordering};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, WindowEvent};

/// Fermer la fenêtre = la cacher (Foyer reste dans la barre), comme Discord.
static CLOSE_TO_TRAY: AtomicBool = AtomicBool::new(true);

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

#[tauri::command]
fn set_close_to_tray(on: bool) {
  CLOSE_TO_TRAY.store(on, Ordering::Relaxed);
}

fn show(app: &AppHandle) {
  if let Some(w) = app.get_webview_window("main") {
    let _ = w.show();
    let _ = w.unminimize();
    let _ = w.set_focus();
  }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    // relancer Foyer alors qu'il est caché : on réaffiche la fenêtre existante
    .plugin(tauri_plugin_single_instance::init(|app, _, _| show(app)))
    .plugin(tauri_plugin_updater::Builder::new().build())
    .plugin(tauri_plugin_process::init())
    .plugin(tauri_plugin_autostart::init(
      tauri_plugin_autostart::MacosLauncher::LaunchAgent,
      Some(vec!["--minimized"]),
    ))
    .invoke_handler(tauri::generate_handler![local_token, set_close_to_tray])
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      let open = MenuItem::with_id(app, "open", "Ouvrir Foyer", true, None::<&str>)?;
      let quit = MenuItem::with_id(app, "quit", "Quitter Foyer", true, None::<&str>)?;
      let menu = Menu::with_items(app, &[&open, &quit])?;
      TrayIconBuilder::with_id("foyer")
        .icon(app.default_window_icon().cloned().expect("icône"))
        .tooltip("Foyer")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, e| match e.id.as_ref() {
          "open" => show(app),
          "quit" => app.exit(0),
          _ => {}
        })
        .on_tray_icon_event(|tray, e| {
          if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = e {
            show(tray.app_handle());
          }
        })
        .build(app)?;
      // lancé au démarrage de l'ordinateur : rester discret dans la barre
      if std::env::args().any(|a| a == "--minimized") {
        if let Some(w) = app.get_webview_window("main") {
          let _ = w.hide();
        }
      }
      Ok(())
    })
    .on_window_event(|w, e| {
      if let WindowEvent::CloseRequested { api, .. } = e {
        if CLOSE_TO_TRAY.load(Ordering::Relaxed) {
          api.prevent_close();
          let _ = w.hide();
        }
      }
    })
    .run(tauri::generate_context!())
    .expect("error while building tauri application");
}
