use std::{
    fs,
    path::{Path, PathBuf},
};

use asbestos_payload::vfs::_resolve_path;
use asbestos_shared::protocol::{Mapping, MappingKind, Mappings};

#[test]
fn test_redirect_mapping() {
    let dir = ensure_test_dir("test_redirect_mapping");

    let game = ensure_dir(dir.join("game"));
    let mods = ensure_dir(dir.join("mods"));
    let mod_loader = ensure_dir(mods.join("mod_loader"));
    let mod_loader_config = ensure_dir(mod_loader.join("config"));

    let mappings = Mappings {
        mappings: vec![Mapping {
            kind: MappingKind::Redirect,
            from: mod_loader.clone(),
            to: game.clone(),
        }],
    };

    let res = _resolve_path(&mod_loader, &mappings).unwrap();
    assert_eq!(res, game);
    let res = _resolve_path(&mod_loader_config, &mappings).unwrap();
    assert_eq!(res, game.join("config"));
}

#[test]
fn test_mount_mapping() {
    let dir = ensure_test_dir("test_mount_mapping");

    let game = ensure_dir(dir.join("game"));
    let game_plugins = ensure_dir(game.join("plugins"));
    let mods = ensure_dir(dir.join("mods"));
    let total_conversion_mod = ensure_dir(mods.join("total_conversion"));
    let total_conversion_mod_config = ensure_dir(total_conversion_mod.join("config"));
    let total_conversion_mod_config_stuff = ensure_dir(total_conversion_mod_config.join("stuff"));

    let game_plugins_total_conversion = game_plugins.join("total_conversion");
    let game_plugins_total_conversion_config = game_plugins_total_conversion.join("config");
    let game_plugins_total_conversion_config_stuff =
        game_plugins_total_conversion_config.join("stuff");

    let mappings = Mappings {
        mappings: vec![Mapping {
            kind: MappingKind::Mount,
            from: total_conversion_mod.clone(),
            to: game_plugins.clone(),
        }],
    };

    let res = _resolve_path(&game_plugins_total_conversion, &mappings).unwrap();
    assert_eq!(res, total_conversion_mod);
    let res = _resolve_path(&game_plugins_total_conversion_config, &mappings).unwrap();
    assert_eq!(res, total_conversion_mod_config);
    let res = _resolve_path(&game_plugins_total_conversion_config_stuff, &mappings).unwrap();
    assert_eq!(res, total_conversion_mod_config_stuff);
}

fn ensure_test_dir<P: AsRef<Path>>(d: P) -> PathBuf {
    let dir = test_dir().join(d);
    if dir.exists() {
        if dir.is_dir() {
            if let Err(err) = fs::remove_dir_all(&dir) {
                panic!(
                    r#"Could not remove directory and its contents at "{}": {}"#,
                    dir.display(),
                    err
                );
            }
        } else if dir.is_file() {
            if let Err(err) = fs::remove_file(&dir) {
                panic!(r#"Could not remove file at "{}": {}"#, dir.display(), err);
            }
        }
    }
    dunce::canonicalize(ensure_dir(dir)).unwrap()
}

fn test_dir() -> &'static Path {
    Path::new(option_env!("CARGO_TARGET_DIR ").unwrap_or("../target"))
}

fn ensure_dir<P: AsRef<Path>>(d: P) -> P {
    let dir = d.as_ref();
    if dir.is_file() {
        panic!(r#""{}" should not be a file"#, dir.display());
    }
    if !dir.exists() {
        if let Err(err) = fs::create_dir(dir) {
            panic!(
                r#"Could not create directory at "{}": {}"#,
                dir.display(),
                err
            );
        }
    }
    d
}

#[allow(dead_code)]
fn ensure_file<P: AsRef<Path>>(f: P) -> P {
    let file = f.as_ref();
    if file.is_dir() {
        panic!(r#""{}" should not be a directory"#, file.display());
    }
    if !file.exists() {
        if let Err(err) = fs::write(file, &[]) {
            panic!(r#"Could not crete file at "{}": {}"#, file.display(), err);
        }
    }
    f
}
