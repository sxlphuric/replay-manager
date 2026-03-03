use eframe::egui;
mod app;
mod thumbnails;
mod videoutils;
pub use app::ReplayManager;

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0]),
        /*.with_icon(
            // NOTE: Adding an icon is optional
            eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                .expect("Failed to load icon"),
        ),*/
        ..Default::default()
    };
    eframe::run_native(
        "Replay Manager",
        native_options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(ReplayManager::new(cc)))
        }),
    )
}

#[cfg(test)]
mod tests {
    use crate::{thumbnails, videoutils};
    use std::path::PathBuf;
    #[test]
    fn can_create_thumbnail() {
        let thumbnail =
            thumbnails::create(&PathBuf::from("../test/bounce.webm"), "test", false, 0.1);
        assert!(thumbnail.expect("Thumbnail not loaded").exists());
    }
    #[test]
    fn can_get_name() {
        let file = PathBuf::from("app.rs");
        assert_eq!("app".to_string(), videoutils::get_name(&file));
    }
    #[test]
    fn can_get_mod_date() {
        let file = PathBuf::from("app.rs");
        //assert_eq!("app".to_string(), videoutils::get_name(&file));
        unimplemented!()
    }
    #[test]
    fn can_get_creation_date() {
        let file = PathBuf::from("app.rs");
        //assert_eq!("app".to_string(), videoutils::get_name(&file));
        unimplemented!()
    }
    #[test]
    fn can_get_size() {
        let file = PathBuf::from("../LICENSE");
        // assert_eq!("app".to_string(), videoutils::get_size(&file));
        unimplemented!();
    }
}
