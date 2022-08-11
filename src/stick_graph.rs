use std::{ffi::OsStr, path::PathBuf};

use eframe::egui::{
    plot::{Legend, Line, Plot, Points, Value, Values},
    Context, Slider, Ui, Window,
};
use log::{info, warn};

use crate::{BoxedResult, ControllerStickEvent, EguiView};

#[derive(Clone)]
struct ControllerStickData {
    left_values: Vec<Value>,
    right_values: Vec<Value>,
}

pub struct ControllerStickGraph {
    csv_data: Option<ControllerStickData>,
    data_offset: u8,
    data_path: Option<PathBuf>,
    plot_id: String,
    show_lines: bool,
    slider_timestamp: usize,
}

impl Default for ControllerStickGraph {
    fn default() -> Self {
        Self {
            csv_data: None,
            data_offset: 50,
            data_path: None,
            plot_id: uuid::Uuid::new_v4().to_string(),
            show_lines: true,
            slider_timestamp: 0,
        }
    }
}

impl ControllerStickGraph {
    /// Load CSV data into this graph
    ///
    /// # Arguments
    ///
    /// * `data_path` - Path to the CSV file to load
    ///
    /// # Errors
    ///
    /// This function will return an error if the CSV data is invalid or not found.
    pub fn load(&mut self, data_path: PathBuf) -> BoxedResult<()> {
        info!("Loading stick data from {}", data_path.display());
        let (ls_events, rs_events) = csv::Reader::from_path(&data_path)?
            .deserialize::<ControllerStickEvent>()
            .try_fold::<_, _, BoxedResult<(Vec<Value>, Vec<Value>)>>(
                (Vec::new(), Vec::new()),
                |mut acc, result| {
                    let event = result?;
                    acc.0.push(Value::new(event.left_x, event.left_y));
                    acc.1.push(Value::new(event.right_x, event.right_y));
                    Ok(acc)
                },
            )?;
        self.data_path = Some(data_path);
        self.csv_data = Some(ControllerStickData {
            left_values: ls_events,
            right_values: rs_events,
        });
        Ok(())
    }
}

impl EguiView for ControllerStickGraph {
    fn show(&mut self, ctx: &Context, is_open: &mut bool) {
        let title = if let Some(path) = self.data_path.as_ref() {
            path.as_path()
                .file_name()
                .unwrap_or_else(|| OsStr::new("Stick Graph"))
                .to_string_lossy()
                .into_owned()
        } else {
            "No data loaded".to_string()
        };
        Window::new(title)
            .resizable(true)
            .collapsible(true)
            .title_bar(true)
            .open(is_open)
            .show(ctx, |ui| self.ui(ui));
    }

    fn ui(&mut self, ui: &mut Ui) {
        if self.csv_data.is_none() {
            ui.label("No stick data loaded");
            return;
        }
        let data = self.csv_data.as_ref().unwrap();
        // use a bit shift since egui is immediate mode
        let midpoint = self.data_offset >> 1; // divide by 2
        let ls_sliced = &data.left_values[self.slider_timestamp.saturating_sub(midpoint.into())
            ..self
                .slider_timestamp
                .saturating_add(midpoint.into())
                .min(data.left_values.len())];
        let rs_sliced = &data.right_values[self.slider_timestamp.saturating_sub(midpoint.into())
            ..self
                .slider_timestamp
                .saturating_add(midpoint.into())
                .min(data.right_values.len())];
        let ls_values = Values::from_values(ls_sliced.to_vec());
        let rs_values = Values::from_values(
            rs_sliced
                .iter()
                .map(|element| Value::new(element.x + 2.5, element.y))
                .collect::<Vec<Value>>(),
        );
        ui.horizontal(|ui| {
            ui.label("Time");
            let left_len = data.left_values.len();
            ui.add(Slider::new(&mut self.slider_timestamp, 0..=left_len));
            ui.checkbox(&mut self.show_lines, "Show lines");
            if ls_sliced.len() == usize::MAX {
                let text = "Warning: too much data to visualize! not all of it will be shown";
                warn!("{}", text);
                ui.label(text);
            }
        });
        ui.horizontal(|ui| {
            ui.label("Number of points");
            ui.add(Slider::new(&mut self.data_offset, u8::MIN..=u8::MAX))
                .on_hover_text("Higher values may affect performance");
        });
        Plot::new(self.plot_id.clone())
            .data_aspect(1.0)
            .legend(Legend::default())
            .show(ui, |plot_ui| {
                let point_radius = 1.0;
                if self.show_lines {
                    plot_ui.line(Line::new(ls_values).name("Left Stick"));
                    plot_ui.line(Line::new(rs_values).name("Right Stick"));
                } else {
                    plot_ui.points(
                        Points::new(ls_values)
                            .radius(point_radius)
                            .name("Left Stick"),
                    );
                    plot_ui.points(
                        Points::new(rs_values)
                            .radius(point_radius)
                            .name("Right Stick"),
                    );
                }
            });
    }
}
