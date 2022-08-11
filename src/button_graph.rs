use std::{collections::HashMap, ffi::OsStr, ops::RangeInclusive, path::PathBuf};

use chrono::{DateTime, NaiveDateTime, Utc};
use eframe::egui::{
    plot::{BoxElem, BoxPlot, BoxSpread, Legend, Plot},
    ComboBox, Context, Ui, Window,
};
use log::info;

use crate::{BoxedResult, ControllerButtonEvent, ControllerType, EguiView};

const DATETIME_FORMAT: &str = "%b %e, %Y %H:%M:%S";
const TIME_FORMAT: &str = "%H:%M:%S";

pub struct ControllerButtonGraph {
    csv_data: Option<HashMap<gilrs::Button, Vec<ControllerButtonEvent>>>,
    data_path: Option<PathBuf>,
    plot_id: String,
    show_date: bool,
    controller_type: ControllerType,
}

impl Default for ControllerButtonGraph {
    fn default() -> Self {
        Self {
            csv_data: None,
            data_path: None,
            show_date: false,
            controller_type: ControllerType::default(),
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
            .try_fold::<_, _, BoxedResult<HashMap<gilrs::Button, Vec<ControllerButtonEvent>>>>(
                HashMap::new(),
                |mut acc, result| {
                    let event = result?;
                    acc.entry(event.button).or_insert_with(Vec::new).push(event);
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
        let date_format = if self.show_date {
            DATETIME_FORMAT
        } else {
            TIME_FORMAT
        };
        let flat_data: Vec<(String, Vec<BoxElem>)> = data
            .into_iter()
            .enumerate()
            .map(|(i, (button, events))| {
                let button_name = self.controller_type.get_button_name(*button);
                let elems: Vec<BoxElem> = events
                    .iter()
                    .map(|e| {
                        let duration = e.release_time - e.press_time;
                        let pressed_at_string = DateTime::<Utc>::from_utc(
                            NaiveDateTime::from_timestamp(e.press_time as i64, 0),
                            Utc,
                        )
                        .format(date_format);
                        let elem_name = format!(
                            "Button: {}\nPressed at: {}\nHeld for: {:.2}",
                            button_name, pressed_at_string, duration
                        );
                        BoxElem::new(
                            (i + 1) as f64,
                            BoxSpread::new(
                                e.press_time,
                                e.press_time,
                                e.press_time,
                                e.release_time,
                                e.release_time,
                            ),
                        )
                        .name(elem_name)
                        .whisker_width(0.0)
                    })
                    .collect();
                (button_name, elems)
            })
            .collect();

        let box_plot_formatter = |elem: &BoxElem, _plot: &BoxPlot| elem.name.clone();

        let box_plots: Vec<BoxPlot> = flat_data
            .iter()
            .map(|(name, elems)| {
                BoxPlot::new(elems.to_vec())
                    .name(name)
                    .horizontal()
                    .element_formatter(Box::new(box_plot_formatter))
            })
            .collect();

        let x_fmt = |x: f64, _range: &RangeInclusive<f64>| {
            // format to datetime string
            let datetime =
                DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(x as i64, 0), Utc);
            datetime.format(date_format).to_string()
        };

        ui.horizontal(|ui| {
            ui.checkbox(&mut self.show_date, "Show date");
            ui.spacing();
            ComboBox::from_label("Controller Type")
                .selected_text(format!("{:?}", self.controller_type))
                .show_ui(ui, |combo_ui| {
                    combo_ui.selectable_value(
                        &mut self.controller_type,
                        ControllerType::Default,
                        "Default",
                    );
                    combo_ui.selectable_value(
                        &mut self.controller_type,
                        ControllerType::Xbox,
                        "Xbox",
                    );
                    combo_ui.selectable_value(
                        &mut self.controller_type,
                        ControllerType::PlayStation,
                        "PlayStation",
                    );
                });
        });
        Plot::new(self.plot_id.clone())
            .legend(Legend::default())
            .x_axis_formatter(x_fmt)
            .show_axes([true, false])
            .show(ui, |plot_ui| {
                box_plots
                    .into_iter()
                    .for_each(|box_plot| plot_ui.box_plot(box_plot));
            });
    }
}
