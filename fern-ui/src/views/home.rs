use crate::components::{Echo, Hero, GuestList, CreateGuestForm};
use dioxus::prelude::*;

/// The Home page component that will be rendered when the current route is `[Route::Home]`
#[component]
pub fn Home() -> Element {
    rsx! {
        Hero {}
        div {
            class: "grid grid-cols-1 lg:grid-cols-2 gap-6 mb-8",
            CreateGuestForm {}
            GuestList {}
        }
        Echo {}
    }
}
