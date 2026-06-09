mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .manage(commands::Session::default())
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
    .invoke_handler(tauri::generate_handler![
      commands::app_version,
      commands::create_identity,
      commands::restore_identity,
      commands::current_identity,
      commands::has_identity,
      commands::export_mnemonic,
      commands::reset_identity,
      commands::connect_nwc,
      commands::connect_lnc,
      commands::set_lightning_address,
      commands::link_identity,
      commands::list_credentials,
      commands::disconnect_credential,
      commands::get_relays,
      commands::set_relays,
      commands::lock_session,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
