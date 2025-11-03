use extism::{PTR, PluginBuilder, UserData, host_fn};
use log::{error, info, warn};

pub fn attach_guest_debug(builder: PluginBuilder) -> PluginBuilder {
    let user_data = UserData::new(());
    builder
        .with_function("guest_info", [PTR], [], user_data.clone(), guest_info)
        .with_function("guest_warn", [PTR], [], user_data.clone(), guest_warn)
        .with_function("guest_error", [PTR], [], user_data.clone(), guest_error)
}

host_fn!(guest_info(_user_data: (); message: String) -> () {
    info!("{}", message);
    Ok(())
});

host_fn!(guest_warn(_user_data: (); message: String) -> () {
    warn!("{}", message);
    Ok(())
});

host_fn!(guest_error(_user_data: (); message: String) -> () {
    error!("{}", message);
    Ok(())
});
