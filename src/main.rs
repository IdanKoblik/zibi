use std::process::Child;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

use tracing::{error, info};

use gtk::Application;
use gtk::prelude::{ApplicationExt, ApplicationExtManual};

use zibi_core::landmark::Landmark;

use crate::config::Config;
use crate::gui::on_activate;

pub mod camera;
pub mod config;
pub mod gui;
pub mod logging;
pub mod niri;
pub mod pipeline;
pub mod points_view;
pub mod track;

pub enum Command {
    Start(Config),
    Stop,
}

fn main() {
    logging::init();

    info!("Loading zibi config");
    let cfg = Arc::new(config::Config::load());

    let (tx, rx) = mpsc::channel::<Command>();
    let (points_tx, points_rx) = mpsc::channel::<Landmark>();

    let gui_thread = thread::spawn({
        let cfg = cfg.clone();
        move || run_gui(cfg, tx, points_rx)
    });

    let mut tracker: Option<Tracker> = None;
    for cmd in rx {
        match cmd {
            Command::Start(cfg) => {
                if tracker.is_some() {
                    error!("Start received while tracker already running, ignoring");
                    continue;
                }
                info!("Starting tracker");
                tracker = Some(Tracker::start(cfg, points_tx.clone()));
            }
            Command::Stop => match tracker.take() {
                Some(tracker) => {
                    info!("Stopping tracker");
                    tracker.stop();
                }
                None => error!("Stop received while no tracker running, ignoring"),
            },
        }
    }

    if let Some(tracker) = tracker.take() {
        tracker.stop();
    }
    let _ = gui_thread.join();
}

fn run_gui(cfg: Arc<Config>, tx: Sender<Command>, points_rx: Receiver<Landmark>) {
    let app = Application::builder()
        .application_id("dev.idank.zibi")
        .build();

    let points_rx = std::cell::RefCell::new(Some(points_rx));

    app.connect_activate(move |application| {
        on_activate(application, &cfg, &tx, points_rx.borrow_mut().take());
    });
    app.run();
}

struct Tracker {
    child: Child,
    worker: JoinHandle<()>,
}

impl Tracker {
    fn start(cfg: Config, points_tx: Sender<Landmark>) -> Self {
        let mut child = track::spawn(&cfg);

        let worker = match child.stdout.take() {
            Some(stdout) => thread::spawn(move || {
                let mut socket = niri::connect();
                pipeline::run(
                    std::io::BufReader::new(stdout),
                    &mut socket,
                    &cfg,
                    &points_tx,
                );
                info!("track stream ended");
            }),
            None => {
                error!("track child has no stdout handle");
                thread::spawn(|| {})
            }
        };

        Tracker { child, worker }
    }

    fn stop(mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = self.worker.join();
    }
}
