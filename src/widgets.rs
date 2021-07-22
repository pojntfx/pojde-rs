use eframe::{
    egui::{self, Label},
    epi,
};
use futures::executor;
use shiplift::Docker;
use tokio::task::spawn_blocking;

use crate::{
    instances::{Instance, Instances},
    update::update,
};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct SerializableInstance {
    pub name: String,
    pub start_port: Option<u64>,
    pub end_port: Option<u64>,
    pub status: String,
}

impl Default for SerializableInstance {
    fn default() -> Self {
        Self {
            name: "".to_owned(),
            start_port: Some(0),
            end_port: Some(0),
            status: "".to_owned(),
        }
    }
}

impl From<&Instance> for SerializableInstance {
    fn from(i: &Instance) -> Self {
        Self {
            name: i.name.to_owned(),
            start_port: i.start_port,
            end_port: i.end_port,
            status: i.status.to_owned(),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Window {
    #[serde(skip)]
    instances: Vec<SerializableInstance>,
    #[serde(skip)]
    refreshing: bool,
    #[serde(skip)]
    manager: Option<Instances>,

    dark: bool,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            instances: vec![],
            refreshing: false,
            manager: None,

            dark: true,
        }
    }
}

impl epi::App for Window {
    fn name(&self) -> &str {
        "pojdegui"
    }

    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &mut epi::Frame<'_>,
        storage: Option<&dyn epi::Storage>,
    ) {
        *self = epi::get_value(storage.unwrap(), epi::APP_KEY).unwrap_or_default()
    }

    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Refresh").clicked() {
                        // TODO: Handle errors and run in background
                        executor::block_on(self.refresh_instances()).unwrap();
                    }

                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });

                egui::menu::menu(ui, "Edit", |ui| {
                    ui.checkbox(&mut self.dark, "Dark mode")
                        .on_hover_text("Enable dark mode");
                });

                egui::menu::menu(ui, "Help", |ui| {
                    if ui.button("Check for updates").clicked() {
                        // TODO: Handle errors and run in background
                        executor::block_on(spawn_blocking(|| update()))
                            .unwrap()
                            .unwrap();
                    }
                });
            });

            self.update_dark_mode(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.refreshing {
                ui.heading("Refreshing ..");
            } else if self.instances.len() <= 0 {
                ui.heading("No instances yet");
                if ui.button("Connect to Docker").clicked() {
                    // TODO: Handle errors and run in background
                    executor::block_on(self.refresh_instances()).unwrap();
                };
            }

            if self.instances.len() > 0 {
                egui::Grid::new("instances").striped(true).show(ui, |ui| {
                    ui.add(Label::new("Name").strong());
                    ui.add(Label::new("Status").strong());
                    ui.add(Label::new("Ports").strong());
                    ui.add(Label::new("Actions").strong());

                    ui.end_row();

                    self.instances.iter().for_each(|i| {
                        if let (Some(start_port), Some(end_port)) = (i.start_port, i.end_port) {
                            ui.label(i.name.to_owned());
                            ui.label(i.status.to_owned());
                            ui.monospace(start_port.to_string() + "-" + &end_port.to_string());

                            ui.horizontal(|ui| {
                                if ui.button("Stop").clicked() {
                                    executor::block_on(self.stop_instance(&i.name.to_owned()))
                                        .unwrap();
                                }
                            });
                        } else {
                            ui.label(i.name.to_owned());
                            ui.label(i.status.to_owned());
                            ui.monospace("");
                        }

                        ui.end_row();
                    });
                });
            }

            egui::warn_if_debug_build(ui);
        });
    }
}

impl Window {
    async fn refresh_instances(&mut self) -> Result<(), ()> {
        let mut s = scopeguard::guard(self, |r| {
            r.refreshing = false;
        });

        s.refreshing = true;

        let manager = match &mut s.manager {
            Some(m) => m,
            None => {
                s.manager = Some(Instances {
                    docker: Docker::new(),
                });

                s.manager.as_ref().unwrap()
            }
        };

        println!("Refreshing ...");

        match manager.get_instances().await {
            Ok(containers) => {
                s.instances = containers
                    .iter()
                    .map(|i| SerializableInstance::from(i))
                    .collect::<Vec<_>>()
            }
            Err(e) => eprintln!("Could not list instances: {}", e),
        }

        Ok(())
    }

    async fn stop_instance(&self, name: &str) -> Result<(), ()> {
        // TODO: Upsert manager by calling `refresh_instances`
        match self.manager.as_ref().unwrap().stop(name).await {
            Ok(_) => {}
            Err(e) => eprintln!("Could not stop instance: {}", e),
        }

        Ok(())
    }

    fn update_dark_mode(&mut self, ui: &mut egui::Ui) {
        if self.dark {
            ui.ctx().set_visuals(egui::Visuals::dark());
        } else {
            ui.ctx().set_visuals(egui::Visuals::light());
        }
    }
}
