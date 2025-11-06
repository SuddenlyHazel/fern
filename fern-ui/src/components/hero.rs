use dioxus::prelude::*;
use crate::api::endpoint_address;

#[component]
pub fn Hero() -> Element {
    // Fetch the endpoint address from the server
    let endpoint = use_resource(move || async move {
        endpoint_address().await
    });

    rsx! {
        div {
            class: "max-w-6xl mx-auto p-6 space-y-8",
            
            // Main header section
            div {
                class: "flex justify-between items-start mb-8",
                div {
                    class: "space-y-2",
                    h1 {
                        class: "text-4xl font-bold text-white flex items-center gap-2",
                        "🌿 Fern"
                    }
                    p {
                        class: "text-xl text-gray-300",
                        "Your weird distributed wasm stack"
                    }
                    // Display endpoint address
                    div {
                        class: "mt-2",
                        match endpoint() {
                            Some(Ok(addr)) => rsx! {
                                div {
                                    class: "flex items-center gap-2 text-sm",
                                    span { class: "text-gray-400", "Endpoint:" }
                                    code {
                                        class: "bg-gray-800 px-2 py-1 rounded text-green-400 font-mono text-xs",
                                        "{addr}"
                                    }
                                }
                            },
                            Some(Err(_)) => rsx! {
                                span { class: "text-red-400 text-sm", "Failed to load endpoint" }
                            },
                            None => rsx! {
                                span { class: "text-gray-500 text-sm", "Loading endpoint..." }
                            }
                        }
                    }
                }
                
                div {
                    class: "flex items-center",
                    div {
                        class: "px-3 py-1 bg-green-600 text-white rounded-full text-sm font-medium flex items-center gap-2",
                        span { class: "w-2 h-2 bg-green-300 rounded-full animate-pulse" }
                        "Server Running"
                    }
                }
            }
            
            
            // Quick actions for admin interface
            div {
                class: "bg-gray-800 border border-gray-700 rounded-lg p-6",
                h3 {
                    class: "text-lg font-semibold text-white mb-4",
                    "Quick Actions"
                }
                div {
                    class: "grid grid-cols-2 md:grid-cols-4 gap-3",
                    button {
                        class: "bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-md text-sm font-medium transition-colors flex items-center justify-center gap-2",
                        "📦 Manage Modules"
                    }
                    button {
                        class: "bg-gray-600 hover:bg-gray-700 text-white px-4 py-2 rounded-md text-sm font-medium transition-colors flex items-center justify-center gap-2",
                        "🔍 View Logs"
                    }
                    button {
                        class: "bg-gray-600 hover:bg-gray-700 text-white px-4 py-2 rounded-md text-sm font-medium transition-colors flex items-center justify-center gap-2",
                        "⚙️ Configuration"
                    }
                    button {
                        class: "bg-gray-600 hover:bg-gray-700 text-white px-4 py-2 rounded-md text-sm font-medium transition-colors flex items-center justify-center gap-2",
                        "📊 Metrics"
                    }
                }
            }
            
            // System info summary
            div {
                class: "bg-gray-800 border border-gray-700 rounded-lg p-6",
                h3 {
                    class: "text-lg font-semibold text-white mb-4",
                    "System Information"
                }
                div {
                    class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                    div {
                        class: "flex justify-between items-center py-2",
                        span { class: "text-gray-400 text-sm", "Runtime:" }
                        span { class: "text-white text-sm font-medium", "Extism + Iroh" }
                    }
                    div {
                        class: "flex justify-between items-center py-2",
                        span { class: "text-gray-400 text-sm", "Status:" }
                        span { class: "text-yellow-400 text-sm font-medium", "Proof of Concept" }
                    }
                    div {
                        class: "flex justify-between items-center py-2",
                        span { class: "text-gray-400 text-sm", "Plugins:" }
                        span { class: "text-green-400 text-sm font-medium", "Ready to Deploy" }
                    }
                }
            }
        }
    }
}
