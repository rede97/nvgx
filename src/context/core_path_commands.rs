use crate::{PathDir, Point, Rect, RendererDevice};

use super::*;

impl<R: RendererDevice> Context<R> {
    #[inline]
    pub fn move_to<P: Into<Point>>(&mut self, pt: P) {
        self.path_cache.xform = self.state_mut().xform;
        self.path_cache.move_to(pt);
    }

    #[inline]
    pub fn line_to<P: Into<Point>>(&mut self, pt: P) {
        self.path_cache.xform = self.state_mut().xform;
        self.path_cache.line_to(pt);
    }

    #[inline]
    pub fn bezier_to<P: Into<Point>>(&mut self, cp1: P, cp2: P, pt: P) {
        self.path_cache.xform = self.state_mut().xform;
        self.path_cache.bezier_to(cp1, cp2, pt);
    }

    #[inline]
    pub fn quad_to<P: Into<Point>>(&mut self, cp: P, pt: P) {
        self.path_cache.xform = self.state_mut().xform;
        self.path_cache.quad_to(cp, pt);
    }

    #[inline]
    pub fn arc_to<P: Into<Point>>(&mut self, pt1: P, pt2: P, radius: f32) {
        self.path_cache.xform = self.state_mut().xform;
        if self.path_cache.commands.is_empty() {
            return;
        }
        let pt0 = self.path_cache.last_position;
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
        self.path_cache.inner_arc_to(pt0, pt1, pt2, radius);
    }

    #[inline]
    pub fn arc<P: Into<Point>>(&mut self, cp: P, radius: f32, a0: f32, a1: f32, dir: PathDir) {
        self.path_cache.xform = self.state_mut().xform;
        self.path_cache.arc(cp, radius, a0, a1, dir);
    }

    #[inline]
    pub fn rect<T: Into<Rect>>(&mut self, rect: T) {
        self.path_cache.xform = self.state_mut().xform;
        self.path_cache.rect(rect);
    }

    #[inline]
    pub fn rounded_rect<T: Into<Rect>>(&mut self, rect: T, radius: f32) {
        self.path_cache.xform = self.state_mut().xform;
        self.path_cache.rounded_rect(rect, radius);
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
        self.path_cache.xform = self.state_mut().xform;
        self.path_cache.rounded_rect_varying(rect, lt, rt, rb, lb);
    }

    #[inline]
    pub fn ellipse<P: Into<Point>>(&mut self, center: P, radius_x: f32, radius_y: f32) {
        self.path_cache.xform = self.state_mut().xform;
        self.path_cache.ellipse(center, radius_x, radius_y);
    }

    #[inline]
    pub fn circle<P: Into<Point>>(&mut self, center: P, radius: f32) {
        self.path_cache.xform = self.state_mut().xform;
        self.path_cache.circle(center, radius);
    }

    #[inline]
    pub fn path_winding<D: Into<PathDir>>(&mut self, dir: D) {
        self.path_cache.path_winding(dir);
    }

    #[inline]
    pub fn begin_path(&mut self) {
        self.path_cache.clear();
        self.path_cache.cache.clear();
    }

    #[inline]
    pub fn close_path(&mut self) {
        self.path_cache.close_path();
    }
}
