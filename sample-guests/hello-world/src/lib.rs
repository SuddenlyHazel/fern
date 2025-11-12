mod pdk;

use std::arch::wasm32;

use extism_pdk::*;
use pdk::*;

// Guest handler for incoming gossip messages
pub(crate) fn gossip_message_handler(_input: types::InboundGossipMsg) -> Result<(), Error> {
    
    guest_info("helloworld::gossip_message_handler".into());
    Ok(())
}

// Handle called on guest upon initializing
pub(crate) fn init() -> Result<(), Error> {
    guest_info("helloworld::init".into());
    Ok(())
}

// Handle called on guest prior to the module being shutdown
pub(crate) fn shutdown() -> Result<(), Error> {
    guest_info("helloworld::shutdown".into());

    Ok(())
}

// Handle called on guest functions per tick (5 times a second best effort)
pub(crate) fn tick() -> Result<(), Error> {
    guest_info("helloworld::tock".into());

    Ok(())
}
