// // Find or create config file
// let config_file = ConfigFile::new(&app.config)?;
// if let Some(config_file) = config_file {
//     app.merge_config(config_file);
// }

// if let Some(dir) = &app.dir {
//     let mut set = ModsSet::new(dir)?;

//     for mod_ident in app.remove.iter() {
//         set.remove(mod_ident)?;
//     }

//     if app.dedup {
//         set.dedup()?;
//     }

//     if app.disable_all {
//         set.disable_all(app.include_base_mod);
//     }

//     if app.enable_all {
//         set.enable_all();
//     }

//     for mod_ident in app.disable.iter() {
//         set.disable(mod_ident)?;
//     }

//     set.enable_list(app.enable)?;

//     set.write_mod_list()?;
//     Ok(())
// } else {
//     Err("Must specify a path either with the --dir flag or in fmm.toml".into())
// }
