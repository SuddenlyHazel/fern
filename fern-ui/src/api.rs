use dioxus::{
    fullstack::serde::{Deserialize, Serialize},
    prelude::*,
};

#[get("/api/server/endpoint", ext: crate::AppStateExtension)]
pub async fn endpoint_address() -> Result<String> {
    let addr = ext.0.server.node_address().await?;
    // The body of server function like this comment are only included on the server. If you have any server-only logic like
    // database queries, you can put it here. Any imports for the server function should either be imported inside the function
    // or imported under a `#[cfg(feature = "server")]` block.
    Ok(format!("{addr}"))
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct GuestInfo {
    pub name: String,
    pub endpoint_id: String,
}
#[get("/api/server/guests", ext: crate::AppStateExtension)]
pub async fn list_guests() -> Result<Vec<GuestInfo>> {
    let guests = ext
        .0
        .server
        .guest_info()
        .await?
        .into_iter()
        .map(|v| GuestInfo {
            name: v.name,
            endpoint_id: v.endpoint_id.to_string(),
        })
        .collect();
    Ok(guests)
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct CreateGuest {
    pub name: String,
    pub module: Vec<u8>,
}

#[post("/api/server/guest", ext: crate::AppStateExtension)]
pub async fn create_guest(req: CreateGuest) -> Result<String> {
    let res = ext.server.create_module(req.name, req.module).await?;
    Ok(res.endpoint_id.to_string())
}
