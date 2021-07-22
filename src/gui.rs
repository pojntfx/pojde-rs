use pojde_rs::widgets;

#[tokio::main]
pub async fn main() {
    let app = widgets::Window::default();
    let native_options = eframe::NativeOptions::default();

    eframe::run_native(Box::new(app), native_options);
}
