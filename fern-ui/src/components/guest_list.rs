use std::time::Duration;

use crate::api::{GuestInfo, UpdateGuest, list_guests, listen_list_guests, update_guest};
use dioxus::{html::time, prelude::*};

#[component]
pub fn GuestList() -> Element {
    // Fetch the list of guests from the server
    let mut guests = use_signal(Vec::new);

    // Auto-refresh every second. Using SSE would be cool
    use_future(move || async move {
        let mut guest_stream = listen_list_guests().await?;
         while let Some(Ok(event)) = guest_stream.recv().await {
            guests.clear();
            for g in event {
                guests.push(g);
            }
        }

        dioxus::Ok(())
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
                    span { "{guests.len()} guest(s)" }
                }
            }

            // Guest list content
            div {
                class: "space-y-3",
                if guests.is_empty() {
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
                } else {
                    for guest in &*guests.read() {
                        GuestCard { guest: guest.clone() }
                    }
                }
            }
        }
    }
}

#[component]
fn GuestCard(guest: GuestInfo) -> Element {
    let mut show_update_form = use_signal(|| false);
    let mut file_data = use_signal(|| Option::<Vec<u8>>::None);
    let mut file_name = use_signal(|| String::new());
    let mut is_updating = use_signal(|| false);
    let mut update_result = use_signal(|| Option::<Result<bool, String>>::None);

    let handle_file_change = move |evt: Event<FormData>| {
        let files = evt.files();
        if !files.is_empty() {
            let file = files[0].clone();
            let file_name_clone = file.name();
            file_name.set(file_name_clone);

            spawn(async move {
                match file.read_bytes().await {
                    Ok(contents) => {
                        file_data.set(Some(contents.to_vec()));
                    }
                    Err(_) => {
                        // Handle error - could set an error state here
                    }
                }
            });
        }
    };

    let guest_name_for_update = guest.name.clone();
    let handle_update = move |_| {
        if file_data().is_none() {
            update_result.set(Some(Err("Module file is required".to_string())));
            return;
        }

        let guest_name = guest_name_for_update.clone();
        let module_data = file_data().unwrap();

        is_updating.set(true);
        update_result.set(None);

        spawn(async move {
            let request = UpdateGuest {
                name: guest_name,
                module: module_data,
            };

            match update_guest(request).await {
                Ok(success) => {
                    update_result.set(Some(Ok(success)));
                    if success {
                        file_data.set(None);
                        file_name.set(String::new());
                        show_update_form.set(false);
                    }
                }
                Err(e) => {
                    update_result.set(Some(Err(format!("Failed to update guest: {}", e))));
                }
            }
            is_updating.set(false);
        });
    };

    let cancel_update = move |_| {
        show_update_form.set(false);
        file_data.set(None);
        file_name.set(String::new());
        update_result.set(None);
    };

    rsx! {
        div {
            class: "bg-gray-700 border border-gray-600 rounded-lg p-4 hover:bg-gray-650 transition-colors",

            // Main guest info
            div {
                class: "flex items-center justify-between gap-4",
                div {
                    class: "flex items-center space-x-3 min-w-0 flex-1",
                    div {
                        class: "w-10 h-10 bg-blue-600 rounded-full flex items-center justify-center text-white font-semibold flex-shrink-0",
                        "{guest.name.chars().next().unwrap_or('?').to_uppercase()}"
                    }
                    div {
                        class: "min-w-0 flex-1",
                        h3 {
                            class: "text-white font-medium truncate",
                            "{guest.name}"
                        }
                        p {
                            class: "text-gray-400 text-sm font-mono truncate",
                            "EndpointId: {guest.endpoint_id}"
                        }
                        p {
                            class: "text-gray-400 text-sm font-mono truncate",
                            "Module Hash: {guest.module_hash}"
                        }
                    }
                    div {
                        class: "flex items-center space-x-2 flex-shrink-0",
                        div {
                            class: "w-2 h-2 bg-green-400 rounded-full animate-pulse"
                        }
                        span {
                            class: "text-green-400 text-sm font-medium",
                            "Connected"
                        }
                    }
                }
                button {
                    class: "px-3 py-1 bg-blue-600 hover:bg-blue-700 text-white text-xs rounded transition-colors flex-shrink-0",
                    onclick: move |_| show_update_form.set(!show_update_form()),
                    if show_update_form() { "Cancel" } else { "Update" }
                }
            }

            // Update form (shown when update button is clicked)
            if show_update_form() {
                div {
                    class: "mt-4 pt-4 border-t border-gray-600",
                    div {
                        class: "space-y-3",

                        // File upload
                        div {
                            label {
                                class: "block text-sm font-medium text-gray-300 mb-2",
                                "New Module File (.wasm)"
                            }
                            input {
                                r#type: "file",
                                accept: ".wasm",
                                class: "w-full px-3 py-2 bg-gray-600 border border-gray-500 rounded-md text-white text-sm file:mr-4 file:py-1 file:px-2 file:rounded file:border-0 file:text-xs file:font-medium file:bg-blue-600 file:text-white hover:file:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                                onchange: handle_file_change,
                                disabled: is_updating(),
                            }
                            if !file_name().is_empty() {
                                div {
                                    class: "mt-1 text-xs text-gray-400",
                                    "Selected: {file_name()}"
                                }
                            }
                        }

                        // Action buttons
                        div {
                            class: "flex space-x-2",
                            button {
                                class: if is_updating() {
                                    "px-3 py-2 bg-gray-600 text-gray-400 rounded text-sm cursor-not-allowed"
                                } else {
                                    "px-3 py-2 bg-green-600 hover:bg-green-700 text-white rounded text-sm transition-colors"
                                },
                                onclick: handle_update,
                                disabled: is_updating(),
                                if is_updating() {
                                    "Updating..."
                                } else {
                                    "Update Guest"
                                }
                            }
                            button {
                                class: "px-3 py-2 bg-gray-600 hover:bg-gray-700 text-white rounded text-sm transition-colors",
                                onclick: cancel_update,
                                disabled: is_updating(),
                                "Cancel"
                            }
                        }

                        // Result display
                        if let Some(result) = update_result() {
                            div {
                                class: "mt-3",
                                match result {
                                    Ok(true) => rsx! {
                                        div {
                                            class: "p-2 bg-green-900 border border-green-700 rounded text-green-300 text-sm",
                                            "✅ Guest updated successfully!"
                                        }
                                    },
                                    Ok(false) => rsx! {
                                        div {
                                            class: "p-2 bg-yellow-900 border border-yellow-700 rounded text-yellow-300 text-sm",
                                            "⚠️ Update completed but may not have been successful"
                                        }
                                    },
                                    Err(error) => rsx! {
                                        div {
                                            class: "p-2 bg-red-900 border border-red-700 rounded text-red-300 text-sm",
                                            "❌ {error}"
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
