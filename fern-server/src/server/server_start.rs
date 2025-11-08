use std::path::PathBuf;

use fern_runtime::{
    guest::{GuestConfig, new_guest},
    iroh_helpers::iroh_bundle,
};
use iroh::EndpointId;
use log::{error, info};

use crate::{Data, GuestInstance, data::GuestRow, server::InstanceMap};

pub async fn handle_start_start(
    data: &Data,
    bootstrap: Vec<EndpointId>,
    instance_map: &mut InstanceMap,
    db_path: Option<PathBuf>,
) -> anyhow::Result<()> {
    let mut offset = 0;
    let limit = 10;
    loop {
        let rows = GuestRow::all_with_pagination(data, limit, offset)?;
        offset += limit;

        if rows.is_empty() {
            break;
        }

        for guest_row in rows {
            let guest_id = guest_row.id.clone();
            let guest_name = guest_row.name.clone();

            let guest_config = GuestConfig {
                name: guest_name.clone(),
                db_path: db_path.clone(),
            };

            if let Ok(instance) = start_guest(guest_row, bootstrap.clone(), guest_config).await {
                info!("Started guest id={} name={}", guest_id, guest_name);
                instance_map.insert(guest_name, instance);
            } else {
                error!("Failed to start guest id={} name={}", guest_id, guest_name);
            }
        }
    }
    Ok(())
}

async fn start_guest(
    guest_row: GuestRow,
    bootstrap: Vec<EndpointId>,
    guest_config: GuestConfig,
) -> anyhow::Result<GuestInstance> {
    let (endpoint, router_builder) = iroh_bundle().await?;
    let mut guest = new_guest(guest_config, guest_row.module, (endpoint, router_builder, bootstrap))?;
    guest.initialize()?;

    let guest_instance = GuestInstance::new(guest, guest_row.module_hash, guest_row.id);

    Ok(guest_instance)
}
