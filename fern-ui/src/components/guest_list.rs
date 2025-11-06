use dioxus::prelude::*;
use crate::api::{list_guests, GuestInfo};

#[component]
pub fn GuestList() -> Element {
    // Fetch the list of guests from the server
    let guests = use_resource(move || async move {
        list_guests().await
    });

    rsx! {
        div {
            class: "bg-gray-800 border border-gray-700 rounded-lg p-6",
            
            // Header
            div {
                class: "flex items-center justify-between mb-4",
                h2 {
                    class: "text-xl font-semibold text-white flex items-center gap-2",
                    "👥 Connected Guests"
                }
                div {
                    class: "text-sm text-gray-400",
                    match &*guests.read() {
                        Some(Ok(guest_list)) => rsx! {
                            span { "{guest_list.len()} guest(s)" }
                        },
                        _ => rsx! { span { "Loading..." } }
                    }
                }
            }
            
            // Guest list content
            div {
                class: "space-y-3",
                match &*guests.read() {
                    Some(Ok(guest_list)) => {
                        if guest_list.is_empty() {
                            rsx! {
                                div {
                                    class: "text-center py-8 text-gray-400",
                                    div {
                                        class: "text-4xl mb-2",
                                        "🔍"
                                    }
                                    p { "No guests connected" }
                                    p {
                                        class: "text-sm mt-1",
                                        "Guests will appear here when they connect to the server"
                                    }
                                }
                            }
                        } else {
                            rsx! {
                                for guest in guest_list {
                                    GuestCard { guest: guest.clone() }
                                }
                            }
                        }
                    },
                    Some(Err(_)) => rsx! {
                        div {
                            class: "text-center py-8 text-red-400",
                            div {
                                class: "text-4xl mb-2",
                                "⚠️"
                            }
                            p { "Failed to load guests" }
                            p {
                                class: "text-sm mt-1",
                                "There was an error fetching the guest list"
                            }
                        }
                    },
                    None => rsx! {
                        div {
                            class: "space-y-3",
                            // Loading skeleton
                            for _ in 0..3 {
                                div {
                                    class: "bg-gray-700 rounded-lg p-4 animate-pulse",
                                    div {
                                        class: "flex items-center space-x-3",
                                        div {
                                            class: "w-10 h-10 bg-gray-600 rounded-full"
                                        }
                                        div {
                                            class: "flex-1 space-y-2",
                                            div {
                                                class: "h-4 bg-gray-600 rounded w-1/4"
                                            }
                                            div {
                                                class: "h-3 bg-gray-600 rounded w-3/4"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn GuestCard(guest: GuestInfo) -> Element {
    rsx! {
        div {
            class: "bg-gray-700 border border-gray-600 rounded-lg p-4 hover:bg-gray-650 transition-colors",
            div {
                class: "flex items-center justify-between",
                div {
                    class: "flex items-center space-x-3",
                    div {
                        class: "w-10 h-10 bg-blue-600 rounded-full flex items-center justify-center text-white font-semibold",
                        "{guest.name.chars().next().unwrap_or('?').to_uppercase()}"
                    }
                    div {
                        class: "flex-1",
                        h3 {
                            class: "text-white font-medium",
                            "{guest.name}"
                        }
                        p {
                            class: "text-gray-400 text-sm font-mono",
                            "ID: {guest.endpoint_id}"
                        }
                    }
                }
                div {
                    class: "flex items-center space-x-2",
                    div {
                        class: "w-2 h-2 bg-green-400 rounded-full animate-pulse"
                    }
                    span {
                        class: "text-green-400 text-sm font-medium",
                        "Connected"
                    }
                }
            }
        }
    }
}