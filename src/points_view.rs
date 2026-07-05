use std::cell::RefCell;
use std::rc::Rc;

use gtk::cairo::Context;
use gtk::prelude::{DrawingAreaExtManual, FrameExt, WidgetExt};
use gtk::{DrawingArea, Frame};

use zibi_core::point::Point;

#[derive(Default)]
struct Snapshot {
    points: Vec<Point>,
    camera_width: i16,
    camera_height: i16,
}

#[derive(Clone)]
pub struct PointsView {
    root: Frame,
    area: DrawingArea,
    snapshot: Rc<RefCell<Snapshot>>,
}

const HAND_CONNECTIONS: &[(usize, usize)] = &[
    // Thumb
    (0, 1),
    (1, 2),
    (2, 3),
    (3, 4),
    // Index
    (0, 5),
    (5, 6),
    (6, 7),
    (7, 8),
    // Middle
    (5, 9),
    (9, 10),
    (10, 11),
    (11, 12),
    // Ring
    (9, 13),
    (13, 14),
    (14, 15),
    (15, 16),
    // Pinky
    (13, 17),
    (17, 18),
    (18, 19),
    (19, 20),
    // Palm
    (0, 17),
];

impl PointsView {
    pub fn new() -> Self {
        let area = DrawingArea::new();
        area.set_hexpand(true);
        area.set_vexpand(true);

        let snapshot: Rc<RefCell<Snapshot>> = Rc::new(RefCell::new(Snapshot::default()));

        area.set_draw_func({
            let snapshot = snapshot.clone();
            move |_area, ctx, width, height| {
                let snapshot = snapshot.borrow();
                Self::draw(ctx, width, height, &snapshot);
            }
        });

        let root = Frame::new(None);
        root.set_hexpand(true);
        root.set_vexpand(true);
        root.set_child(Some(&area));

        Self {
            root,
            area,
            snapshot,
        }
    }

    pub fn widget(&self) -> &Frame {
        &self.root
    }

    pub fn set_points(&self, points: Vec<Point>, camera_width: i16, camera_height: i16) {
        *self.snapshot.borrow_mut() = Snapshot {
            points,
            camera_width,
            camera_height,
        };
        self.area.queue_draw();
    }

    fn draw(ctx: &Context, width: i32, height: i32, snapshot: &Snapshot) {
        if snapshot.camera_width <= 0 || snapshot.camera_height <= 0 {
            return;
        }

        let camera_width = snapshot.camera_width as f64;
        let camera_height = snapshot.camera_height as f64;
        let width = width as f64;
        let height = height as f64;

        let mapped: Vec<(f64, f64)> = snapshot
            .points
            .iter()
            .map(|point| {
                let x = (point.x as f64 / camera_width) * width;
                let y = (point.y as f64 / camera_height) * height;
                (x, y)
            })
            .collect();

        ctx.set_source_rgb(1.0, 1.0, 1.0);
        ctx.set_line_width(2.0);

        for &(start, end) in HAND_CONNECTIONS {
            let (x1, y1) = mapped[start];
            let (x2, y2) = mapped[end];

            ctx.move_to(x1, y1);
            ctx.line_to(x2, y2);
            let _ = ctx.stroke();
        }

        ctx.set_source_rgb(0.2, 0.7, 1.0);
        for &(x, y) in &mapped {
            ctx.arc(x, y, 4.0, 0.0, std::f64::consts::TAU);
            let _ = ctx.fill();
        }
    }
}

impl Default for PointsView {
    fn default() -> Self {
        Self::new()
    }
}
