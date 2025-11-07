use dioxus::prelude::*;
use crate::api::{create_guest, CreateGuest};

#[component]
pub fn CreateGuestForm() -> Element {
    let mut name = use_signal(|| String::new());
    let mut file_data = use_signal(|| Option::<Vec<u8>>::None);
    let mut file_name = use_signal(|| String::new());
    let mut file_value = use_signal(|| vec![]);
    let mut is_creating = use_signal(|| false);
    let mut create_result = use_signal(|| Option::<Result<String, String>>::None);
    let mut file_input_key = use_signal(|| 0u32);

    let handle_file_change = move |evt: Event<FormData>| {
        let files = evt.files();
        if !files.is_empty() {
            let file = files[0].clone();
            let file_name_clone = file.name();
            file_name.set(file_name_clone.clone());
            file_value.set(vec![file_name_clone]);
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
        } else {
            // Clear file data when no file is selected
            file_name.set(String::new());
            file_data.set(None);
            file_value.clear();
        }
    };

    let handle_submit = move |_| {
        if name().trim().is_empty() {
            create_result.set(Some(Err("Guest name is required".to_string())));
            return;
        }
        
        if file_data().is_none() {
            create_result.set(Some(Err("Module file is required".to_string())));
            return;
        }

        let guest_name = name();
        let module_data = file_data().unwrap();
        
        is_creating.set(true);
        create_result.set(None);
        
        spawn(async move {
            let request = CreateGuest {
                name: guest_name,
                module: module_data,
            };
            
            match create_guest(request).await {
                Ok(endpoint_id) => {
                    create_result.set(Some(Ok(endpoint_id)));
                    name.set(String::new());
                    file_data.set(None);
                    file_name.set(String::new());
                    file_value.clear();
                    // Force re-render of file input to clear selection
                    file_input_key.set(file_input_key() + 1);
                }
                Err(e) => {
                    create_result.set(Some(Err(format!("Failed to create guest: {}", e))));
                }
            }
            is_creating.set(false);
        });
    };

    rsx! {
        div {
            class: "bg-gray-800 border border-gray-700 rounded-lg p-6",
            
            // Header
            div {
                class: "flex items-center gap-2 mb-6",
                h2 {
                    class: "text-xl font-semibold text-white",
                    "➕ Create New Guest"
                }
            }
            
            // Form
            div {
                class: "space-y-4",
                
                // Guest name input
                div {
                    label {
                        class: "block text-sm font-medium text-gray-300 mb-2",
                        "Guest Name"
                    }
                    input {
                        r#type: "text",
                        class: "w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                        placeholder: "Enter guest name...",
                        value: "{name}",
                        oninput: move |evt| name.set(evt.value()),
                        disabled: is_creating(),
                    }
                }
                
                // File upload
                div {
                    key: "{file_input_key()}",
                    label {
                        class: "block text-sm font-medium text-gray-300 mb-2",
                        "Module File (.wasm)"
                    }
                    div {
                        class: "relative",
                        input {
                            r#type: "file",
                            accept: ".wasm",
                            class: "w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white file:mr-4 file:py-1 file:px-2 file:rounded file:border-0 file:text-sm file:font-medium file:bg-blue-600 file:text-white hover:file:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                            onchange: handle_file_change,
                            disabled: is_creating(),
                        }
                        if !file_name().is_empty() {
                            div {
                                class: "mt-2 text-sm text-gray-400",
                                "Selected: {file_name()}"
                            }
                        }
                    }
                }
                
                // Submit button
                button {
                    class: if is_creating() {
                        "w-full px-4 py-2 bg-gray-600 text-gray-400 rounded-md cursor-not-allowed"
                    } else {
                        "w-full px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-md font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-gray-800"
                    },
                    onclick: handle_submit,
                    disabled: is_creating(),
                    if is_creating() {
                        "Creating Guest..."
                    } else {
                        "Create Guest"
                    }
                }
                
                // Result display
                if let Some(result) = create_result() {
                    div {
                        class: "mt-4",
                        match result {
                            Ok(endpoint_id) => rsx! {
                                div {
                                    class: "p-3 bg-green-900 border border-green-700 rounded-md",
                                    div {
                                        class: "flex items-center gap-2",
                                        span {
                                            class: "text-green-400 text-sm font-medium",
                                            "✅ Guest created successfully!"
                                        }
                                    }
                                    div {
                                        class: "mt-2 text-xs text-green-300",
                                        "Endpoint ID: "
                                        code {
                                            class: "bg-green-800 px-1 py-0.5 rounded font-mono",
                                            "{endpoint_id}"
                                        }
                                    }
                                }
                            },
                            Err(error) => rsx! {
                                div {
                                    class: "p-3 bg-red-900 border border-red-700 rounded-md",
                                    div {
                                        class: "flex items-center gap-2",
                                        span {
                                            class: "text-red-400 text-sm font-medium",
                                            "❌ {error}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                // Help text
                div {
                    class: "text-xs text-gray-500 mt-4",
                    p { "Upload a WebAssembly (.wasm) module file to create a new guest instance." }
                    p { "The guest will be assigned a unique endpoint ID for communication." }
                }
            }
        }
    }
}