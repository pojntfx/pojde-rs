use eframe::{egui, epi};
use futures::executor;
use shiplift::Docker;

use crate::instances::{Instance, Instances};

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))]
pub struct Window {
    instances: Vec<Instance>,
    refreshing: bool,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            instances: vec![],
            refreshing: false,
        }
    }
}

impl epi::App for Window {
    fn name(&self) -> &str {
        "pojdegui"
    }

    #[cfg(feature = "persistence")]
    fn load(&mut self, storage: &dyn epi::Storage) {
        *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
    }

    #[cfg(feature = "persistence")]
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
            });
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
                self.instances.iter().for_each(|i| {
                    ui.label(i.name.to_owned());
                })
            }

            egui::warn_if_debug_build(ui);
        });
    }
}

impl Window {
    async fn refresh_instances(&mut self) -> Result<(), ()> {
        let manager = Instances {
            docker: Docker::new(),
        };

        let mut s = scopeguard::guard(self, |r| {
            r.refreshing = false;
        });

        s.refreshing = true;

        println!("Refreshing ...");

        match manager.get_instances().await {
            Ok(containers) => s.instances = containers,
            Err(e) => eprintln!("Could not list instances: {}", e),
        }

        Ok(())
    }
}
