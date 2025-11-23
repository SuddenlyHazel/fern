use std::sync::Arc;

use tokio::sync::Mutex;
use wasmtime::{
    Config, Engine, Store,
    component::{HasData, Linker, bindgen},
};
use wasmtime_wasi::{
    ResourceTable, WasiCtx, WasiCtxView, WasiView,
    p2::{DynOutputStream, pipe::MemoryOutputPipe},
};

pub mod iroh_helpers;
pub mod runtime;
pub mod runtime_config;
pub mod sqlite;

use sqlite::SqliteState;

bindgen!({
    path: "../fern-sdk/wit/",
    world: "fern",
    // Configure async behavior for imports and exports
    imports: {
        // Make all imports async and trappable
        default: async,
        //"wasi:sockets/network@0.2.7": async,
        //"wasi:sockets/tcp@0.2.7": async,
    },
    exports: {
        default: async
    },
    with: {
        // Note, bindgen! docs are incorrect. `fern:base/fern-sqlite.rows` is how
        // you access something within a package. Atleast given how we're defining
        // our world..
        "fern:base/sqlite.rows": sqlite::RowsResource,
        "fern:base/sqlite.database": sqlite::DatabaseResource,
        "wasi:clocks@0.2.8" : wasmtime_wasi::p2::bindings::clocks,
    }
});

pub struct GroupState {
    wasi_ctx: WasiCtx,
    resource_table: ResourceTable,
    sqlite_state: SqliteState,
}

impl WasiView for GroupState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi_ctx,
            table: &mut self.resource_table,
        }
    }
}

pub struct InstanceGroup {
    engine: Engine,
    linker: Linker<GroupState>,
    store: Arc<Mutex<Store<GroupState>>>,
    // Just for testing
}

impl HasData for GroupState {
    type Data<'a> = &'a mut GroupState;
}

pub fn new_instance_group() -> anyhow::Result<InstanceGroup> {
    let mut config = Config::new();
    config.async_support(true); // Required for async operations
    config.wasm_component_model(true); // Enable component model
    config.wasm_component_model_async(true); // Enable Component Model async ABI

    let engine = Engine::new(&config)?;

    let mut linker = Linker::<GroupState>::new(&engine);

    wasmtime_wasi::p2::add_to_linker_async(&mut linker)?;

    fern::base::sqlite::add_to_linker::<GroupState, SqliteState>(&mut linker, |t| {
        &mut t.sqlite_state
    })?;

    let wasi = WasiCtx::builder()
        .inherit_network() // REQUIRED for socket support
        // TODO we need to setup custom pipes for the ctx
        .inherit_stdio() // Allow stdio access
        .build();

    let sqlite_state = SqliteState::new();
    let store = Store::new(
        &engine,
        GroupState {
            wasi_ctx: wasi,
            resource_table: ResourceTable::new(),
            sqlite_state,
        },
    );

    let store = Arc::new(Mutex::new(store));
    Ok(InstanceGroup {
        linker,
        store,
        engine,
    })
}

#[cfg(test)]
mod test {
    use anyhow::Context;
    use wasmtime::component::Component;

    use crate::{Fern, new_instance_group};

    // Can run this with ..
    // cargo test --package fern-runtime-wasmtime --lib --all-features -- test::runtime_setup_test --exact --nocapture
    #[tokio::test]
    async fn runtime_setup_test() -> anyhow::Result<()> {
        env_logger::Builder::new()
            .filter(Some("fern_runtime_wasmtime"), log::LevelFilter::Debug)
            .init();
        let mut group = new_instance_group()?;

        // TODO remove before publishing
        let component_path = "/Users/hazel/src/fern/test_guest/wasmtime-component-test/target/wasm32-wasip2/debug/wasmtime_component_test.wasm";
        let component = Component::from_file(&group.engine, component_path)
            .context("failed to load component from file")?;

        let mut locked_store = group.store.lock().await;
        let mut store = &mut *locked_store;

        let fern_guest = Fern::instantiate_async(&mut store, &component, &group.linker)
            .await
            .context("Failed to init guest")?;

        assert!(
            fern_guest
                .fern_base_guest()
                .call_init(&mut store)
                .await
                .context("failed to init guest")?
        );
        assert!(
            fern_guest
                .fern_base_guest()
                .call_post_init(&mut store)
                .await
                .context("failed to post init guest")?
        );

        assert!(
            fern_guest
                .fern_base_guest()
                .call_shutdown(&mut store)
                .await
                .context("failed to call guest shutdown")?
        );

        // Not really testing anything
        // But, demostates how to persist state in
        // the guest I gess
        let tick_res = fern_guest.fern_base_guest().call_tick(&mut store).await?;
        assert!(tick_res.is_ok());

        let tick_res = fern_guest.fern_base_guest().call_tick(&mut store).await?;
        assert!(tick_res.is_ok());

        let tick_res = fern_guest.fern_base_guest().call_tick(&mut store).await?;
        assert!(tick_res.is_ok());

        let tick_res = fern_guest.fern_base_guest().call_tick(&mut store).await?;
        assert!(tick_res.is_ok());

        // Test components can share the pipe
        let fern_guest_two = Fern::instantiate_async(&mut store, &component, &group.linker)
            .await
            .context("Failed to init guest")?;

        assert!(
            fern_guest_two
                .fern_base_guest()
                .call_init(&mut store)
                .await
                .context("failed to init guest")?
        );

        Ok(())
    }
}
