use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::Sender;

use gtk::prelude::{
    BoxExt, ButtonExt, Cast, EditableExt, EntryExt, FrameExt, GtkWindowExt, WidgetExt,
};
use gtk::{
    Align, Application, ApplicationWindow, Box as GtkBox, Button, DropDown, Entry, Frame,
    InputPurpose, Label, Orientation, StringObject,
};
use gtk::glib;
use gtk::glib::object::IsA;

use tracing::{error, info};
use zibi_core::hand::Hand;

use crate::camera::list_cameras;
use crate::config::{Config, CoreConfig};
use crate::Command;

pub fn on_activate(application: &Application, cfg: &Arc<Config>, tx: &Sender<Command>) {
    let window = ApplicationWindow::new(application);
    window.set_title(Some("zibi"));
    window.set_default_size(1000, 640);

    let root = GtkBox::new(Orientation::Horizontal, 0);

    root.append(&build_sidebar(cfg, tx));
    root.append(&build_content());

    window.set_child(Some(&root));
    window.present();
}

fn build_sidebar(cfg: &Arc<Config>, tx: &Sender<Command>) -> GtkBox {
    let sidebar = GtkBox::new(Orientation::Vertical, 12);
    sidebar.set_width_request(260);
    sidebar.set_margin_top(12);
    sidebar.set_margin_bottom(12);
    sidebar.set_margin_start(12);
    sidebar.set_margin_end(12);
    sidebar.add_css_class("sidebar");

    let heading = Label::new(Some("Properties"));
    heading.set_halign(Align::Start);
    heading.add_css_class("title-4");
    sidebar.append(&heading);

    // 1) Move threshold input.
    let move_threshold = Entry::new();
    move_threshold.set_input_purpose(InputPurpose::Number);
    move_threshold.set_placeholder_text(Some(&format!("{}", cfg.core.move_threshold)));
    sidebar.append(&field("Move threshold", &move_threshold));

    // 2) Hand dropdown.
    let side = DropDown::from(cfg.core.dominant_hand);
    sidebar.append(&field("Side", &side));

    // 3) Camera selection dropdown.
    let cameras = list_cameras();
    let camera_refs: Vec<&str> = cameras.iter().map(String::as_str).collect();
    let camera = DropDown::from_strings(&camera_refs);
    sidebar.append(&field("Camera", &camera));

    // Push the action buttons to the bottom.
    let spacer = GtkBox::new(Orientation::Vertical, 0);
    spacer.set_vexpand(true);
    sidebar.append(&spacer);

    sidebar.append(&build_actions(&move_threshold, &side, &camera, cfg, tx));

    sidebar
}

fn field(title: &str, control: &impl IsA<gtk::Widget>) -> GtkBox {
    let group = GtkBox::new(Orientation::Vertical, 4);

    let label = Label::new(Some(title));
    label.set_halign(Align::Start);
    label.add_css_class("heading");
    group.append(&label);
    group.append(control);

    group
}

fn build_actions(
    move_threshold: &Entry,
    side: &DropDown,
    camera: &DropDown,
    cfg: &Arc<Config>,
    tx: &Sender<Command>,
) -> GtkBox {
    let actions = GtkBox::new(Orientation::Vertical, 8);

    let save = Button::with_label("Save");
    save.connect_clicked(glib::clone!(
        #[weak]
        move_threshold,
        #[weak]
        side,
        #[weak]
        camera,
        #[strong]
        cfg,
        move |_| {
            let new_cfg = read_config(&move_threshold, &side, &camera, &cfg);
            match new_cfg.save() {
                Ok(()) => info!("Config saved"),
                Err(e) => error!("Failed to save config: {e}"),
            }
        }
    ));
    actions.append(&save);

    let start = Button::with_label("Start");
    start.add_css_class("suggested-action");

    let stop = Button::with_label("Stop");
    stop.add_css_class("destructive-action");
    stop.set_visible(false);

    start.connect_clicked(glib::clone!(
        #[weak]
        stop,
        #[weak]
        save,
        #[weak]
        move_threshold,
        #[weak]
        side,
        #[weak]
        camera,
        #[strong]
        cfg,
        #[strong]
        tx,
        move |start| {
            start.set_visible(false);
            stop.set_visible(true);
            // Lock the inputs while tracking is running.
            save.set_sensitive(false);
            move_threshold.set_sensitive(false);
            side.set_sensitive(false);
            camera.set_sensitive(false);

            // Hand the current form off to the main thread to start tracking.
            let cfg = read_config(&move_threshold, &side, &camera, &cfg);
            if let Err(e) = tx.send(Command::Start(cfg)) {
                error!("Failed to send Start command: {e}");
            }
        }
    ));
    stop.connect_clicked(glib::clone!(
        #[weak]
        start,
        #[weak]
        save,
        #[weak]
        move_threshold,
        #[weak]
        side,
        #[weak]
        camera,
        #[strong]
        tx,
        move |stop| {
            stop.set_visible(false);
            start.set_visible(true);
            // Unlock the inputs once tracking has stopped.
            save.set_sensitive(true);
            move_threshold.set_sensitive(true);
            side.set_sensitive(true);
            camera.set_sensitive(true);

            if let Err(e) = tx.send(Command::Stop) {
                error!("Failed to send Stop command: {e}");
            }
        }
    ));

    actions.append(&start);
    actions.append(&stop);

    actions
}

fn build_content() -> GtkBox {
    let content = GtkBox::new(Orientation::Vertical, 0);
    content.set_hexpand(true);
    content.set_vexpand(true);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);

    let inner = Frame::new(None);
    inner.set_hexpand(true);
    inner.set_vexpand(true);

    let placeholder = Label::new(Some("Press Start to begin hand tracking"));
    placeholder.set_halign(Align::Center);
    placeholder.set_valign(Align::Center);
    placeholder.add_css_class("dim-label");
    placeholder.add_css_class("title-2");
    inner.set_child(Some(&placeholder));

    content.append(&inner);

    content
}

fn read_config(move_threshold: &Entry, side: &DropDown, camera: &DropDown, cfg: &Config) -> Config {
    let move_threshold = move_threshold
        .text()
        .trim()
        .parse::<i16>()
        .unwrap_or(cfg.core.move_threshold);

    let dominant_hand = match side.selected() {
        0 => Hand::Left,
        _ => Hand::Right,
    };

    let camera = camera
        .selected_item()
        .and_then(|item| item.downcast::<StringObject>().ok())
        .map(|s| PathBuf::from(s.string().as_str()))
        .unwrap_or_else(|| cfg.core.camera.clone());

    Config {
        core: CoreConfig {
            move_threshold,
            dominant_hand,
            camera,
        },
    }
}
