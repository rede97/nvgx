use cache::PathCache;

use crate::{BufferId, Point, Transform, Vector2D};
use core::f32;
use std::{
    cell::RefCell,
    ops::{Add, Deref, DerefMut},
};

use crate::Rect;
pub(crate) mod cache;
mod transform;

pub const KAPPA90: f32 = 0.5522847493;
pub const PI: f32 = std::f32::consts::PI;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub enum PathDir {
    #[default]
    CCW,
    CW,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum WindingSolidity {
    Solid,
    Hole,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
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
}

pub struct Path {
    pub(crate) last_position: Point,
    pub(super) commands: Vec<Command>,
    pub(crate) fill_type: PathFillType,
    pub(crate) xform: Transform,
    pub(crate) xforms: Vec<Transform>,
}

impl Path {
    pub fn new() -> Self {
        return Self {
            last_position: Point { x: 0.0, y: 0.0 },
            commands: Vec::new(),
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

    #[inline]
    pub(crate) fn inner_arc_to(&mut self, pt0: Point, pt1: Point, pt2: Point, radius: f32) {
        let d0 = Point::new(pt0.x - pt1.x, pt0.y - pt1.y).normal();
        let d1 = Point::new(pt2.x - pt1.x, pt2.y - pt1.y).normal();
        let a = f32::acos(d0.dot(&d1));
        let sin_a_half = f32::sin(a / 2.0);
        if sin_a_half.abs() < f32::EPSILON {
            self.line_to(pt1);
            return;
        }
        let d = radius / sin_a_half;
        if d > 10000.0 {
            self.line_to(pt1);
            return;
        }

        let c = d0.add(&d1).normal().mul(d).add(&pt1);

        let (a0, a1, dir) = if Point::cross(d0, d1) > 0.0 {
            (d0.x.atan2(-d0.y), -d1.x.atan2(d1.y), PathDir::CW)
        } else {
            (-d0.x.atan2(d0.y), d1.x.atan2(-d1.y), PathDir::CCW)
        };

        self.arc(c, radius, a0, a1, dir);
    }

    pub fn arc_to<P: Into<Point>>(&mut self, pt1: P, pt2: P, radius: f32) {
        if self.commands.is_empty() {
            return;
        }
        let pt0 = self.last_position;
        let pt1 = pt1.into();
        let pt2 = pt2.into();
        self.inner_arc_to(pt0, pt1, pt2, radius);
    }

    pub fn arc<P: Into<Point>>(&mut self, cp: P, radius: f32, a0: f32, a1: f32, dir: PathDir) {
        let cp = cp.into();
        let move_ = self.commands.is_empty();

        let mut da = a1 - a0;
        if dir == PathDir::CW {
            if da.abs() >= PI * 2.0 {
                da = PI * 2.0;
            } else {
                while da < 0.0 {
                    da += PI * 2.0;
                }
            }
        } else {
            if da.abs() >= PI * 2.0 {
                da = -PI * 2.0;
            } else {
                while da > 0.0 {
                    da -= PI * 2.0;
                }
            }
        }

        let ndivs = ((da.abs() / (PI * 0.5) + 0.5) as i32).min(5).max(1);
        let hda = (da / (ndivs as f32)) / 2.0;
        let mut kappa = (4.0 / 3.0 * (1.0 - hda.cos()) / hda.sin()).abs();

        if dir == PathDir::CCW {
            kappa = -kappa;
        }

        let mut px = 0.0;
        let mut py = 0.0;
        let mut ptanx = 0.0;
        let mut ptany = 0.0;

        for i in 0..=ndivs {
            let a = a0 + da * ((i as f32) / (ndivs as f32));
            let dx = a.cos();
            let dy = a.sin();
            let x = cp.x + dx * radius;
            let y = cp.y + dy * radius;
            let tanx = -dy * radius * kappa;
            let tany = dx * radius * kappa;

            if i == 0 {
                if move_ {
                    self.append_command(Command::MoveTo(Point::new(x, y)));
                } else {
                    self.append_command(Command::LineTo(Point::new(x, y)));
                }
            } else {
                self.append_command(Command::BezierTo(
                    Point::new(px + ptanx, py + ptany),
                    Point::new(x - tanx, y - tany),
                    Point::new(x, y),
                ));
            }
            px = x;
            py = y;
            ptanx = tanx;
            ptany = tany;
        }
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

    #[inline]
    pub(crate) fn clear(&mut self) {
        self.commands.clear();
    }
}

pub struct PathWithCache {
    pub(crate) path: Path,
    pub(crate) cache: RefCell<PathCache>,
    pub(crate) vertex_buffer: BufferId,
}

impl PathWithCache {
    pub(crate) fn new(buffer: BufferId) -> Self {
        return PathWithCache {
            path: Path::new(),
            cache: RefCell::new(Default::default()),
            vertex_buffer: buffer,
        };
    }
}

impl Deref for PathWithCache {
    type Target = Path;
    fn deref(&self) -> &Self::Target {
        return &self.path;
    }
}

impl DerefMut for PathWithCache {
    fn deref_mut(&mut self) -> &mut Self::Target {
        return &mut self.path;
    }
}
