use cache::PathCache;

use crate::{Point, Transform};
use core::f32;
use std::cell::RefCell;

use crate::Rect;
pub mod cache;
mod transform;

pub const KAPPA90: f32 = 0.5522847493;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PathDir {
    CCW,
    CW,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum WindingSolidity {
    Solid,
    Hole,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PathFillType {
    Winding,
    EvenOdd,
}

impl Into<PathDir> for WindingSolidity {
    fn into(self) -> PathDir {
        match self {
            WindingSolidity::Solid => PathDir::CCW,
            WindingSolidity::Hole => PathDir::CW,
        }
    }
}

#[derive(Debug)]
pub(crate) enum Command {
    MoveTo(Point),
    LineTo(Point),
    BezierTo(Point, Point, Point),
    Close,
    Winding(PathDir),
    ArcTo(Point, Point, f32),
    Arc(Point, f32, f32, f32, PathDir),
}

pub struct Path {
    pub(crate) last_position: Point,
    pub(super) commands: Vec<Command>,
    pub(crate) cache: RefCell<PathCache>,
    pub(crate) fill_type: PathFillType,
    pub(crate) xform: Transform,
    pub(crate) xforms: Vec<Transform>,
}

impl Path {
    pub fn new() -> Self {
        return Self {
            last_position: Point { x: 0.0, y: 0.0 },
            commands: Vec::new(),
            cache: Default::default(),
            fill_type: PathFillType::Winding,
            xform: Transform::identity(),
            xforms: Vec::new(),
        };
    }

    fn append_command(&mut self, cmd: Command) {
        let xform = &self.xform;
        match cmd {
            Command::MoveTo(pt) => {
                self.commands
                    .push(Command::MoveTo(xform.transform_point(pt)));
                self.last_position = pt;
            }
            Command::LineTo(pt) => {
                self.commands
                    .push(Command::LineTo(xform.transform_point(pt)));
                self.last_position = pt;
            }
            Command::BezierTo(pt1, pt2, pt3) => {
                self.last_position = pt3;
                self.commands.push(Command::BezierTo(
                    xform.transform_point(pt1),
                    xform.transform_point(pt2),
                    xform.transform_point(pt3),
                ));
            }
            _ => {
                self.commands.push(cmd);
            }
        }
    }

    pub fn move_to<P: Into<Point>>(&mut self, pt: P) {
        self.append_command(Command::MoveTo(pt.into()));
    }

    pub fn line_to<P: Into<Point>>(&mut self, pt: P) {
        self.append_command(Command::LineTo(pt.into()));
    }

    pub fn bezier_to<P: Into<Point>>(&mut self, cp1: P, cp2: P, pt: P) {
        self.append_command(Command::BezierTo(cp1.into(), cp2.into(), pt.into()));
    }

    pub fn quad_to<P: Into<Point>>(&mut self, cp: P, pt: P) {
        let x0 = self.last_position.x;
        let y0 = self.last_position.y;
        let cp = cp.into();
        let pt = pt.into();
        self.append_command(Command::BezierTo(
            Point::new(x0 + 2.0 / 3.0 * (cp.x - x0), y0 + 2.0 / 3.0 * (cp.y - y0)),
            Point::new(
                pt.x + 2.0 / 3.0 * (cp.x - pt.x),
                pt.y + 2.0 / 3.0 * (cp.y - pt.y),
            ),
            pt,
        ));
    }

    pub fn arc_to<P: Into<Point>>(&mut self, pt1: P, pt2: P, radius: f32) {
        self.append_command(Command::ArcTo(pt1.into(), pt2.into(), radius));
    }

    pub fn arc<P: Into<Point>>(&mut self, cp: P, radius: f32, a0: f32, a1: f32, dir: PathDir) {
        self.append_command(Command::Arc(cp.into(), radius, a0, a1, dir));
    }

    pub fn rect<T: Into<Rect>>(&mut self, rect: T) {
        let rect = rect.into();
        self.append_command(Command::MoveTo(Point::new(rect.xy.x, rect.xy.y)));
        self.append_command(Command::LineTo(Point::new(
            rect.xy.x,
            rect.xy.y + rect.size.height,
        )));
        self.append_command(Command::LineTo(Point::new(
            rect.xy.x + rect.size.width,
            rect.xy.y + rect.size.height,
        )));
        self.append_command(Command::LineTo(Point::new(
            rect.xy.x + rect.size.width,
            rect.xy.y,
        )));
        self.append_command(Command::Close);
    }

    pub fn rounded_rect<T: Into<Rect>>(&mut self, rect: T, radius: f32) {
        let rect = rect.into();
        self.rounded_rect_varying(rect, radius, radius, radius, radius);
    }

    pub fn rounded_rect_varying<T: Into<Rect>>(
        &mut self,
        rect: T,
        lt: f32,
        rt: f32,
        rb: f32,
        lb: f32,
    ) {
        let rect = rect.into();
        if lt < 0.1 && rt < 0.1 && lb < 0.1 && rb < 0.1 {
            self.rect(rect);
        } else {
            let halfw = rect.size.width.abs() * 0.5;
            let halfh = rect.size.height.abs() * 0.5;
            let rxlb = lb.min(halfw) * rect.size.width.signum();
            let rylb = lb.min(halfh) * rect.size.height.signum();
            let rxrb = rb.min(halfw) * rect.size.width.signum();
            let ryrb = rb.min(halfh) * rect.size.height.signum();
            let rxrt = rt.min(halfw) * rect.size.width.signum();
            let ryrt = rt.min(halfh) * rect.size.height.signum();
            let rxlt = lt.min(halfw) * rect.size.width.signum();
            let rylt = lt.min(halfh) * rect.size.height.signum();

            self.append_command(Command::MoveTo(Point::new(rect.xy.x, rect.xy.y + rylt)));
            self.append_command(Command::LineTo(Point::new(
                rect.xy.x,
                rect.xy.y + rect.size.height - rylb,
            )));
            self.append_command(Command::BezierTo(
                Point::new(
                    rect.xy.x,
                    rect.xy.y + rect.size.height - rylb * (1.0 - KAPPA90),
                ),
                Point::new(
                    rect.xy.x + rxlb * (1.0 - KAPPA90),
                    rect.xy.y + rect.size.height,
                ),
                Point::new(rect.xy.x + rxlb, rect.xy.y + rect.size.height),
            ));
            self.append_command(Command::LineTo(Point::new(
                rect.xy.x + rect.size.width - rxrb,
                rect.xy.y + rect.size.height,
            )));
            self.append_command(Command::BezierTo(
                Point::new(
                    rect.xy.x + rect.size.width - rxrb * (1.0 - KAPPA90),
                    rect.xy.y + rect.size.height,
                ),
                Point::new(
                    rect.xy.x + rect.size.width,
                    rect.xy.y + rect.size.height - ryrb * (1.0 - KAPPA90),
                ),
                Point::new(
                    rect.xy.x + rect.size.width,
                    rect.xy.y + rect.size.height - ryrb,
                ),
            ));
            self.append_command(Command::LineTo(Point::new(
                rect.xy.x + rect.size.width,
                rect.xy.y + ryrt,
            )));
            self.append_command(Command::BezierTo(
                Point::new(
                    rect.xy.x + rect.size.width,
                    rect.xy.y + ryrt * (1.0 - KAPPA90),
                ),
                Point::new(
                    rect.xy.x + rect.size.width - rxrt * (1.0 - KAPPA90),
                    rect.xy.y,
                ),
                Point::new(rect.xy.x + rect.size.width - rxrt, rect.xy.y),
            ));
            self.append_command(Command::LineTo(Point::new(rect.xy.x + rxlt, rect.xy.y)));
            self.append_command(Command::BezierTo(
                Point::new(rect.xy.x + rxlt * (1.0 - KAPPA90), rect.xy.y),
                Point::new(rect.xy.x, rect.xy.y + rylt * (1.0 - KAPPA90)),
                Point::new(rect.xy.x, rect.xy.y + rylt),
            ));
            self.append_command(Command::Close);
        }
    }

    pub fn ellipse<P: Into<Point>>(&mut self, center: P, radius_x: f32, radius_y: f32) {
        let center = center.into();
        self.append_command(Command::MoveTo(Point::new(center.x - radius_x, center.y)));
        self.append_command(Command::BezierTo(
            Point::new(center.x - radius_x, center.y + radius_y * KAPPA90),
            Point::new(center.x - radius_x * KAPPA90, center.y + radius_y),
            Point::new(center.x, center.y + radius_y),
        ));
        self.append_command(Command::BezierTo(
            Point::new(center.x + radius_x * KAPPA90, center.y + radius_y),
            Point::new(center.x + radius_x, center.y + radius_y * KAPPA90),
            Point::new(center.x + radius_x, center.y),
        ));
        self.append_command(Command::BezierTo(
            Point::new(center.x + radius_x, center.y - radius_y * KAPPA90),
            Point::new(center.x + radius_x * KAPPA90, center.y - radius_y),
            Point::new(center.x, center.y - radius_y),
        ));
        self.append_command(Command::BezierTo(
            Point::new(center.x - radius_x * KAPPA90, center.y - radius_y),
            Point::new(center.x - radius_x, center.y - radius_y * KAPPA90),
            Point::new(center.x - radius_x, center.y),
        ));
        self.append_command(Command::Close);
    }

    pub fn circle<P: Into<Point>>(&mut self, center: P, radius: f32) {
        self.ellipse(center.into(), radius, radius);
    }

    pub fn path_winding<D: Into<PathDir>>(&mut self, dir: D) {
        self.append_command(Command::Winding(dir.into()));
    }

    pub fn close_path(&mut self) {
        self.commands.push(Command::Close);
    }

    pub fn fill_type(&mut self, fill_type: PathFillType) {
        self.fill_type = fill_type;
    }

    pub(crate) fn clear(&mut self) {
        self.commands.clear();
        self.cache.borrow_mut().clear();
    }
}
