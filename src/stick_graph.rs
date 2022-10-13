use std::{ffi::OsStr, path::PathBuf};

use eframe::egui::{
    plot::{Legend, Line, Plot, PlotPoints, Points},
    Context, Slider, Ui, Window,
};
use log::{info, warn};

use crate::{util::f64_to_formatted_time, ControllerStickEvent, CsvLoad, EguiView};

#[derive(Clone)]
struct ControllerStickData {
    left_values: Vec<[f64; 2]>,
    right_values: Vec<[f64; 2]>,
    timestamps: Vec<f64>,
}

pub struct ControllerStickGraph {
    csv_data: Option<ControllerStickData>,
    data_offset: u8,
    data_path: Option<PathBuf>,
    plot_id: uuid::Uuid,
    show_lines: bool,
    slider_timestamp: usize,
}

impl Default for ControllerStickGraph {
    fn default() -> Self {
        Self {
            csv_data: None,
            data_offset: 50,
            data_path: None,
            plot_id: uuid::Uuid::new_v4(),
            show_lines: true,
            slider_timestamp: 0,
        }
    }
}

impl CsvLoad for ControllerStickGraph {
    fn load(&mut self, data_path: PathBuf) -> csv::Result<()> {
        info!("Loading stick data from {}", data_path.display());
        let (left_values, right_values, timestamps) = csv::Reader::from_path(&data_path)?
            .deserialize::<ControllerStickEvent>()
            .try_fold::<_, _, Result<_, csv::Error>>(
                (
                    Vec::<[f64; 2]>::new(),
                    Vec::<[f64; 2]>::new(),
                    Vec::<f64>::new(),
                ),
                |mut acc, result| {
                    let event = result?;
                    acc.0.push([event.left_x, event.left_y]);
                    acc.1.push([event.right_x, event.right_y]);
                    acc.2.push(event.time);
                    Ok(acc)
                },
            )?;
        self.data_path = Some(data_path);
        self.csv_data = Some(ControllerStickData {
            left_values,
            right_values,
            timestamps,
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
        let left_offset_timestamp = self.slider_timestamp.saturating_sub(midpoint.into());
        let right_offset_timestamp = self.slider_timestamp.saturating_add(midpoint.into());

        let ls_sliced = &data.left_values
            [left_offset_timestamp..right_offset_timestamp.min(data.left_values.len())];
        let rs_sliced = &data.right_values
            [left_offset_timestamp..right_offset_timestamp.min(data.right_values.len())];

        let ls_values = PlotPoints::new(ls_sliced.to_vec());
        // shift the right stick values to the right so they don't overlap the left stick
        let rs_values = PlotPoints::new(rs_sliced.iter().map(|v| [v[0] + 2.5, v[1]]).collect());

        ui.horizontal(|ui| {
            ui.label("Time");
            let left_len = data.left_values.len();
            ui.add(Slider::new(&mut self.slider_timestamp, 0..=left_len));
            let base_time = data.timestamps.first().unwrap_or(&0.0);
            let start_time_string =
                f64_to_formatted_time(data.timestamps[left_offset_timestamp] - base_time);
            let end_time_string = f64_to_formatted_time(
                data.timestamps[right_offset_timestamp.min(left_len - 1)] - base_time,
            );
            ui.label(format!("{} - {}", start_time_string, end_time_string));
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
            ui.checkbox(&mut self.show_lines, "Show lines");
        });
        Plot::new(self.plot_id)
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
