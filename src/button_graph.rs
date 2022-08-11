use std::{collections::HashMap, ffi::OsStr, path::PathBuf};

use eframe::egui::{
    plot::{BoxElem, BoxPlot, BoxSpread, Legend, Plot},
    Context, Ui, Window,
};
use log::info;

use crate::{BoxedResult, ControllerButtonEvent, EguiView};

pub struct ControllerButtonGraph {
    csv_data: Option<HashMap<String, Vec<BoxElem>>>,
    data_path: Option<PathBuf>,
    plot_id: String,
}

impl Default for ControllerButtonGraph {
    fn default() -> Self {
        Self {
            csv_data: None,
            data_path: None,
            plot_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

impl ControllerButtonGraph {
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
        info!("loading button data from {}", data_path.display());
        let data = csv::Reader::from_path(&data_path)?
            .deserialize::<ControllerButtonEvent>()
            .try_fold::<_, _, BoxedResult<HashMap<String, Vec<BoxElem>>>>(
                HashMap::new(),
                |mut acc, result| {
                    let event = result?;
                    let duration = event.release_time - event.press_time;
                    let box_elem = BoxElem::new(
                        0.5,
                        BoxSpread::new(
                            event.press_time,
                            event.press_time,
                            event.press_time,
                            event.release_time,
                            event.release_time,
                        ),
                    )
                    .whisker_width(0.0)
                    .name(format!("{} ({:.2}s)", event.button, duration));
                    match acc.get_mut(&event.button) {
                        Some(vec) => vec.push(box_elem),
                        None => {
                            acc.insert(event.button, vec![box_elem]);
                        }
                    }
                    Ok(acc)
                },
            )?;
        self.data_path = Some(data_path);
        self.csv_data = Some(data);
        Ok(())
    }
}

impl EguiView for ControllerButtonGraph {
    fn show(&mut self, ctx: &Context, is_open: &mut bool) {
        let title = if let Some(path) = self.data_path.as_ref() {
            path.as_path()
                .file_name()
                .unwrap_or_else(|| OsStr::new("Button Graph"))
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
            ui.label("No data loaded");
            return;
        }
        let data = self.csv_data.as_ref().unwrap();
        let box_plots: Vec<BoxPlot> = data
            .iter()
            .enumerate()
            .map(|(i, (key, vec))| {
                let mapped_boxes: Vec<BoxElem> = vec
                    .iter()
                    .cloned()
                    .map(|mut e| {
                        e.argument = i as f64;
                        e
                    })
                    .collect();
                let formatter = |elem: &BoxElem, _plot: &BoxPlot| elem.name.clone();
                BoxPlot::new(mapped_boxes)
                    .name(key)
                    .horizontal()
                    .element_formatter(Box::new(formatter))
            })
            .collect();

        Plot::new(self.plot_id.clone())
            .legend(Legend::default())
            .show(ui, |plot_ui| {
                box_plots
                    .into_iter()
                    .for_each(|box_plot| plot_ui.box_plot(box_plot));
            });
    }
}
