mod pdk;

use extism_pdk::*;
use pdk::*;
use serde_json::Map;

use crate::pdk::types::OutboundGossipMsg;

// Guest handler for incoming gossip messages
pub(crate) fn gossip_message_handler(input: types::InboundGossipMsg) -> Result<(), Error> {
    guest_info(format!("Received {input:#?}"));

    let v = input.content.get("hello").unwrap().as_str().unwrap();

    let mut content = Map::new();
    let id = extism_pdk::config::get("id").unwrap().unwrap();

    content.insert(
        "hello".to_string(),
        serde_json::Value::String(format!("{v} {id}")),
    );
    broadcast_msg(OutboundGossipMsg { content });
    Ok(())
}
