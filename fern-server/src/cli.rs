use iocraft::prelude::*;

use crate::GuestInfo;

#[derive(Default, Props)]
pub struct GuestsTableProps {
    pub guests: Option<Vec<GuestInfo>>,
}

#[component]
pub fn GuestsTable<'a>(props: &GuestsTableProps) -> impl Into<AnyElement<'a>> {
    element! {
        View(
            margin_top: 1,
            margin_bottom: 1,
            flex_direction: FlexDirection::Column,
            width: 120,
            border_style: BorderStyle::Round,
            border_color: Color::Cyan,
        ) {
            View(border_style: BorderStyle::Single, border_edges: Edges::Bottom, border_color: Color::Grey) {
                View(width: 15, justify_content: JustifyContent::Center, padding_right: 2) {
                    Text(content: "Name", weight: Weight::Bold, decoration: TextDecoration::Underline)
                }

                View(width: 50, justify_content: JustifyContent::Center, padding_left: 1, padding_right: 1) {
                    Text(content: "Endpoint ID", weight: Weight::Bold, decoration: TextDecoration::Underline)
                }

                View(width: 50, justify_content: JustifyContent::Center, padding_left: 1) {
                    Text(content: "Module Hash", weight: Weight::Bold, decoration: TextDecoration::Underline)
                }
            }

            #(props.guests.as_ref().map(|guests| guests.iter().enumerate().map(|(i, guest)| {
                let endpoint_str = guest.endpoint_id.to_string();
                let module_hash_str = guest.module_hash.clone();
                
                // Truncate long strings for better display
                let endpoint_display = if endpoint_str.len() > 48 {
                    format!("{}...", &endpoint_str[..45])
                } else {
                    endpoint_str
                };
                
                let module_display = if module_hash_str.len() > 48 {
                    format!("{}...", &module_hash_str[..45])
                } else {
                    module_hash_str
                };
                
                element! {
                    View(background_color: if i % 2 == 0 { None } else { Some(Color::DarkGrey) }, padding_top: 0, padding_bottom: 0) {
                        View(width: 15, justify_content: JustifyContent::Start, padding_right: 2) {
                            Text(content: guest.name.to_string())
                        }

                        View(width: 50, justify_content: JustifyContent::Start, padding_left: 1, padding_right: 1) {
                            Text(content: endpoint_display)
                        }

                        View(width: 50, justify_content: JustifyContent::Start, padding_left: 1) {
                            Text(content: module_display)
                        }
                    }
                }
            }).collect::<Vec<_>>()).into_iter().flatten())
        }
    }
}
