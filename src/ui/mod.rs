mod color;
mod components;
mod state;

use std::sync::{Arc, Mutex};

use egui::{Color32, Label, Response, TextStyle, Widget};
use globset::GlobSetBuilder;

use self::color::ToColor32;
use self::components::common::CommonProps;
use self::components::constants;
use self::components::level_menu_button::LevelMenuButton;
use self::components::table::Table;
use self::components::table_cell::TableCell;
use self::components::table_header::TableHeader;
use self::components::target_menu_button::TargetMenuButton;
use self::state::LogsState;
use crate::string::Ellipse;
use crate::time::DateTimeFormatExt;
use crate::tracing::collector::EventCollector;
use crate::tracing::CollectedEvent;

pub struct Logs {
    collector: EventCollector,
}

impl Logs {
    #[must_use]
    pub const fn new(collector: EventCollector) -> Self {
        Self { collector }
    }
}

impl Widget for Logs {
    fn ui(self, ui: &mut egui::Ui) -> Response {
        let state = ui.memory_mut(|mem| {
            let state_mem_id = ui.id();
            mem.data
                .get_temp_mut_or_insert_with(state_mem_id, || {
                    Arc::new(Mutex::new(LogsState::default()))
                })
                .clone()
        });
        let mut state = state.lock().unwrap();

        // TODO: cache the globset
        let glob = {
            let mut glob = GlobSetBuilder::new();
            for target in state.target_filter.targets.clone() {
                glob.add(target);
            }
            glob.build().unwrap()
        };

        let events = self.collector.events();
        let filtered_events = events
            .iter()
            .filter(|event| state.level_filter.get(event.level) && !glob.is_match(&event.target))
            .collect::<Vec<_>>();

        let row_height = constants::SEPARATOR_SPACING
            + ui.style().text_styles.get(&TextStyle::Small).unwrap().size;

        Table::default()
            .on_clear(|| {
                self.collector.clear();
            })
            .header(|ui| {
                TableHeader::default()
                    .common_props(CommonProps::default().min_width(100.0))
                    .children(|ui| {
                        ui.label("Time");
                    })
                    .show(ui);
                TableHeader::default()
                    .common_props(CommonProps::default().min_width(80.0))
                    .children(|ui| {
                        LevelMenuButton::default()
                            .state(&mut state.level_filter)
                            .show(ui)
                    })
                    .show(ui);
                TableHeader::default()
                    .common_props(CommonProps::default().min_width(120.0))
                    .children(|ui| {
                        TargetMenuButton::default()
                            .state(&mut state.target_filter)
                            .show(ui)
                    })
                    .show(ui);
                TableHeader::default()
                    .common_props(CommonProps::default().min_width(120.0))
                    .children(|ui| {
                        ui.label("Content");
                    })
                    .show(ui);
            })
            .row_height(row_height)
            .row(|ui, event: &CollectedEvent| {
                TableCell::default()
                    .common_props(CommonProps::default().min_width(100.0))
                    .children(|ui| {
                        ui.colored_label(Color32::GRAY, event.time.format_short())
                            .on_hover_text(event.time.format_detailed());
                    })
                    .show(ui);
                TableCell::default()
                    .common_props(CommonProps::default().min_width(80.0))
                    .children(|ui| {
                        ui.colored_label(event.level.to_color32(), event.level.as_str());
                    })
                    .show(ui);
                TableCell::default()
                    .common_props(CommonProps::default().min_width(120.0))
                    .children(|ui| {
                        ui.colored_label(Color32::GRAY, event.target.truncate_graphemes(18))
                            .on_hover_text(&event.target);
                    })
                    .show(ui);
                TableCell::default()
                    .common_props(CommonProps::default().min_width(120.0))
                    .children(|ui| {
                        let mut short_message = String::new()
                            + event.fields.get("message")
                                .map_or("", |msg| msg.as_str());
                        let mut complete_message = String::new()
                            + event.fields.get("message")
                                .map_or("", |msg| msg.as_str());
                        for ff in event.fields.iter() {
                            if ff.0 == "message" {
                                continue;
                            }
                            if ff.0.starts_with("log.") {
                                continue;
                            }
                            complete_message += format!("\n {}: {}", ff.0, ff.1).as_str();
                            short_message += format!(", {}: {}", ff.0, ff.1).as_str();
                        }
                        complete_message += "\n\n";
                        for ff in event.fields.iter() {
                            if !ff.0.starts_with("log.") {
                                continue;
                            }
                            complete_message += format!("\n {}: {}", ff.0, ff.1).as_str();
                        }

                        ui.style_mut().visuals.override_text_color = Some(Color32::WHITE);
                        ui.add(Label::new(short_message).wrap(false))
                            .on_hover_text(complete_message);
                    })
                    .show(ui);
            })
            .show(ui, filtered_events.iter())
    }
}
