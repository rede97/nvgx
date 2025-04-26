use super::Context;
use crate::{PathDir, Point, Rect, Renderer};

impl<R: Renderer> Context<R> {
    #[inline]
    pub fn move_to<P: Into<Point>>(&mut self, pt: P) {
        self.path.xform = self.state_mut().xform;
        self.path.move_to(pt);
    }

    #[inline]
    pub fn line_to<P: Into<Point>>(&mut self, pt: P) {
        self.path.xform = self.state_mut().xform;
        self.path.line_to(pt);
    }

    #[inline]
    pub fn bezier_to<P: Into<Point>>(&mut self, cp1: P, cp2: P, pt: P) {
        self.path.xform = self.state_mut().xform;
        self.path.bezier_to(cp1, cp2, pt);
    }

    #[inline]
    pub fn quad_to<P: Into<Point>>(&mut self, cp: P, pt: P) {
        self.path.xform = self.state_mut().xform;
        self.path.quad_to(cp, pt);
    }

    #[inline]
    pub fn arc_to<P: Into<Point>>(&mut self, pt1: P, pt2: P, radius: f32) {
        self.path.xform = self.state_mut().xform;
        if self.path.commands.is_empty() {
            return;
        }
        let pt0 = self.path.last_position;
        let pt1 = pt1.into();
        let pt2 = pt2.into();
        if pt0.equals(pt1, self.dist_tol)
            || pt1.equals(pt2, self.dist_tol)
            || pt1.dist_pt_seg(pt0, pt2) < self.dist_tol * self.dist_tol
            || radius < self.dist_tol
        {
            self.line_to(pt1);
            return;
        }
        self.path.arc_to(pt1, pt2, radius);
    }

    #[inline]
    pub fn arc<P: Into<Point>>(&mut self, cp: P, radius: f32, a0: f32, a1: f32, dir: PathDir) {
        self.path.xform = self.state_mut().xform;
        self.path.arc(cp, radius, a0, a1, dir);
    }

    #[inline]
    pub fn rect<T: Into<Rect>>(&mut self, rect: T) {
        self.path.xform = self.state_mut().xform;
        self.path.rect(rect);
    }

    #[inline]
    pub fn rounded_rect<T: Into<Rect>>(&mut self, rect: T, radius: f32) {
        self.path.xform = self.state_mut().xform;
        self.path.rounded_rect(rect, radius);
    }

    #[inline]
    pub fn rounded_rect_varying<T: Into<Rect>>(
        &mut self,
        rect: T,
        lt: f32,
        rt: f32,
        rb: f32,
        lb: f32,
    ) {
        self.path.xform = self.state_mut().xform;
        self.path.rounded_rect_varying(rect, lt, rt, rb, lb);
    }

    #[inline]
    pub fn ellipse<P: Into<Point>>(&mut self, center: P, radius_x: f32, radius_y: f32) {
        self.path.xform = self.state_mut().xform;
        self.path.ellipse(center, radius_x, radius_y);
    }

    #[inline]
    pub fn circle<P: Into<Point>>(&mut self, center: P, radius: f32) {
        self.path.xform = self.state_mut().xform;
        self.path.circle(center, radius);
    }

    #[inline]
    pub fn path_winding<D: Into<PathDir>>(&mut self, dir: D) {
        self.path.path_winding(dir);
    }

    #[inline]
    pub fn begin_path(&mut self) {
        self.path.clear();
    }

    #[inline]
    pub fn close_path(&mut self) {
        self.path.close_path();
    }
}
