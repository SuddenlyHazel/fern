use extism::{Manifest, Plugin, PluginBuilder, Wasm};

use crate::guest_fns;

const PRE_INIT_FN : &str = "preInit";
const POST_INIT_FN : &str = "postInit";
const MESSAGE_FN : &str = "messageHandler";

const SQL_TEST : &str = "testEnhancedSql";

pub fn new_guest(guest_module: impl Into<Wasm>) -> anyhow::Result<Plugin> {
    let manifest = Manifest::new([guest_module]);
    let builder = PluginBuilder::new(manifest).with_wasi(true);

    let builder = guest_fns::kv::attach_guest_kv(builder);
    let builder = guest_fns::sqlite_improved::attach_guest_sqlite_improved(builder);
    let builder = guest_fns::debug::attach_guest_debug(builder);

    let plugin = builder.build()?;
    
    Ok(plugin)
}

#[test]
fn test_rust_guest() {
    use log::info;

    env_logger::builder().filter_level(log::LevelFilter::Info).init();
    let test_module =
        include_bytes!("../../test_guest/test-rs-revised/target/wasm32-wasip1/release/plugin.wasm");
    let mut guest = new_guest(test_module.to_vec()).expect("failed to create guest");
    let r = guest.call::<&str, serde_json::Value>(SQL_TEST, "hello");
    info!("{r:#?}");
}
