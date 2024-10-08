use bindgen::Bindings;
use std::env;
use std::path::PathBuf;

const WEECHAT_BUNDLED_ENV: &str = "WEECHAT_BUNDLED";
const WEECHAT_PLUGIN_FILE_ENV: &str = "WEECHAT_PLUGIN_FILE";

fn build(file: &str) -> Result<Bindings, ()> {
    const INCLUDED_TYPES: &[&str] = &[
        "t_weechat_plugin",
        "t_gui_buffer",
        "t_gui_nick",
        "t_gui_nick_group",
        "t_hook",
        "t_weechat_hdata",
    ];
    const INCLUDED_VARS: &[&str] = &[
        "WEECHAT_PLUGIN_API_VERSION",
    ];
    let mut builder = bindgen::Builder::default().rustfmt_bindings(true);

    builder = builder.header(file);

    for t in INCLUDED_TYPES {
        builder = builder.whitelist_type(t);
    }

    for v in INCLUDED_VARS {
        builder = builder.whitelist_var(v);
    }

    builder.generate()
}

fn main() {
    let bundled = env::var(WEECHAT_BUNDLED_ENV).map_or(false, |bundled| {
        match bundled.to_lowercase().as_ref() {
            "1" | "true" | "yes" => true,
            "0" | "false" | "no" => false,
            _ => panic!("Invalid value for WEECHAT_BUNDLED, must be true/false"),
        }
    });

    let plugin_file = env::var(WEECHAT_PLUGIN_FILE_ENV);


    let bindings = if bundled {
        build("src/weechat-plugin.h").expect("Unable to generate bindings")
    } else {
        match plugin_file {
            Ok(file) => {
                let path = PathBuf::from(file)
                    .canonicalize()
                    .expect("Can't canonicalize path");
                build(path.to_str().unwrap_or_default()).expect(&format!(
                    "Unable to generate bindings with the provided {:?}",
                    path
                ))
            }
            Err(_) => {
                let bindings = build("src/wrapper.h");

                match bindings {
                    Ok(b) => b,
                    Err(_) => build("src/weechat-plugin.h").expect("Unable to generate bindings"),
                }
            }
        }
    };

    println!("cargo:rerun-if-env-changed={}", WEECHAT_BUNDLED_ENV);
    println!("cargo:rerun-if-env-changed={}", WEECHAT_PLUGIN_FILE_ENV);

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
