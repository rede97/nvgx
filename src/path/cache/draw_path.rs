use super::{cap_join::*, PathSlice};
use super::{PathCache, PointFlags, VPoint};
use super::{PathInfo, Vertex};
use crate::paint::{LineCap, LineJoin};
use crate::{Command, PathDir, Point};
use rawpointer::ptrdistance;
use std::f32::consts::PI;

impl PathCache {
    /// clear path and vertices
    #[inline]
    pub fn reset(&mut self) {
        self.clear();
        self.vertices.clear();
    }

    /// only clear path
    #[inline]
    pub fn clear(&mut self) {
        self.points.clear();
        self.paths.clear();
    }

    fn add_path(&mut self) -> &mut PathInfo {
        self.paths.push(PathInfo {
            first: self.points.len(),
            count: 0,
            closed: false,
            num_bevel: 0,
            windding: PathDir::CCW,
            convex: false,
        });
        self.paths.last_mut().unwrap()
    }

    fn add_point(&mut self, pt: Point, flags: PointFlags, dist_tol: f32) {
        if let Some(path) = self.paths.last_mut() {
            if let Some(last_pt) = self.points.last_mut() {
                if path.count > 0 {
                    if last_pt.xy.equals(pt, dist_tol) {
                        last_pt.flags |= flags;
                        return;
                    }
                }
            }

            self.points.push(VPoint {
                xy: pt,
                d: Default::default(),
                len: 0.0,
                dm: Default::default(),
                flags,
            });
            path.count += 1;
        }
    }

    fn close_path(&mut self) {
        if let Some(path) = self.paths.last_mut() {
            path.closed = true;
        }
    }

    fn path_solidity(&mut self, windding: PathDir) {
        if let Some(path) = self.paths.last_mut() {
            path.windding = windding;
        }
    }

    unsafe fn alloc_temp_vertices(&mut self, count: usize) -> (*const Vertex, *mut Vertex) {
        let offset = self.vertices.len();
        self.vertices.resize(offset + count, Default::default());
        if self.vertices.is_empty() {
            return (std::ptr::null(), std::ptr::null_mut());
        }
        (
            &self.vertices[0] as *const Vertex,
            &mut self.vertices[offset] as *mut Vertex,
        )
    }

    fn tesselate_bezier(
        &mut self,
        pt1: Point,
        pt2: Point,
        pt3: Point,
        pt4: Point,
        level: usize,
        flags: PointFlags,
        tess_tol: f32,
    ) {
        if level > 10 {
            return;
        }

        let Point { x: x1, y: y1 } = pt1;
        let Point { x: x2, y: y2 } = pt2;
        let Point { x: x3, y: y3 } = pt3;
        let Point { x: x4, y: y4 } = pt4;

        let x12 = (x1 + x2) * 0.5;
        let y12 = (y1 + y2) * 0.5;
        let x23 = (x2 + x3) * 0.5;
        let y23 = (y2 + y3) * 0.5;
        let x34 = (x3 + x4) * 0.5;
        let y34 = (y3 + y4) * 0.5;
        let x123 = (x12 + x23) * 0.5;
        let y123 = (y12 + y23) * 0.5;

        let dx = x4 - x1;
        let dy = y4 - y1;
        let d2 = ((x2 - x4) * dy - (y2 - y4) * dx).abs();
        let d3 = ((x3 - x4) * dy - (y3 - y4) * dx).abs();

        if (d2 + d3) * (d2 + d3) < tess_tol * (dx * dx + dy * dy) {
            self.add_point(Point::new(x4, y4), flags, tess_tol);
            return;
        }

        let x234 = (x23 + x34) * 0.5;
        let y234 = (y23 + y34) * 0.5;
        let x1234 = (x123 + x234) * 0.5;
        let y1234 = (y123 + y234) * 0.5;

        self.tesselate_bezier(
            Point::new(x1, y1),
            Point::new(x12, y12),
            Point::new(x123, y123),
            Point::new(x1234, y1234),
            level + 1,
            PointFlags::empty(),
            tess_tol,
        );
        self.tesselate_bezier(
            Point::new(x1234, y1234),
            Point::new(x234, y234),
            Point::new(x34, y34),
            Point::new(x4, y4),
            level + 1,
            flags,
            tess_tol,
        );
    }

    pub(crate) fn flatten_paths(&mut self, commands: &[Command], dist_tol: f32, tess_tol: f32) {
        if self.paths.len() != 0 {
            return;
        }
        for cmd in commands {
            match cmd {
                Command::MoveTo(pt) => {
                    self.add_path();
                    self.add_point(*pt, PointFlags::PT_CORNER, dist_tol);
                }
                Command::LineTo(pt) => {
                    self.add_point(*pt, PointFlags::PT_CORNER, dist_tol);
                }
                Command::BezierTo(cp1, cp2, pt) => {
                    if let Some(last) = self.points.last().map(|pt| *pt) {
                        self.tesselate_bezier(
                            last.xy,
                            *cp1,
                            *cp2,
                            *pt,
                            0,
                            PointFlags::PT_CORNER,
                            tess_tol,
                        );
                    }
                }
                Command::Close => self.close_path(),
                Command::Winding(solidity) => self.path_solidity(*solidity),
            }
        }

        self.bounds.min = Point::new(std::f32::MAX, std::f32::MAX);
        self.bounds.max = Point::new(std::f32::MIN, std::f32::MIN);

        unsafe {
            for j in 0..self.paths.len() {
                let path = &mut self.paths[j];
                let pts = &mut self.points[path.first] as *mut VPoint;
                let mut p0 = pts.offset(path.count as isize - 1);
                let mut p1 = pts;

                if (*p0).xy.equals((*p1).xy, dist_tol) {
                    if path.count > 0 {
                        path.count -= 1;
                    }
                    p0 = pts.offset(path.count as isize - 1);
                    path.closed = true;
                }

                if path.count > 2 {
                    let area = poly_area(std::slice::from_raw_parts(pts, path.count));
                    if path.windding == PathDir::CCW && area < 0.0 {
                        poly_reverse(std::slice::from_raw_parts_mut(pts, path.count));
                    }
                    if path.windding == PathDir::CW && area > 0.0 {
                        poly_reverse(std::slice::from_raw_parts_mut(pts, path.count));
                    }
                }

                for _ in 0..path.count {
                    (*p0).d.x = (*p1).xy.x - (*p0).xy.x;
                    (*p0).d.y = (*p1).xy.y - (*p0).xy.y;
                    (*p0).len = (*p0).d.normalize();

                    self.bounds.min.x = self.bounds.min.x.min((*p0).xy.x);
                    self.bounds.min.y = self.bounds.min.y.min((*p0).xy.y);
                    self.bounds.max.x = self.bounds.max.x.max((*p0).xy.x);
                    self.bounds.max.y = self.bounds.max.y.max((*p0).xy.y);

                    p0 = p1;
                    p1 = p1.add(1);
                }
            }
        }
    }

    fn calculate_joins(&mut self, w: f32, line_join: LineJoin, miter_limit: f32) {
        let mut iw = 0.0;
        if w > 0.0 {
            iw = 1.0 / w;
        }

        unsafe {
            for i in 0..self.paths.len() {
                let path = &mut self.paths[i];
                let pts = &mut self.points[path.first] as *mut VPoint;
                let mut p0 = pts.offset(path.count as isize - 1);
                let mut p1 = pts;
                let mut nleft = 0;

                path.num_bevel = 0;

                for _ in 0..path.count {
                    let dlx0 = (*p0).d.y;
                    let dly0 = -(*p0).d.x;
                    let dlx1 = (*p1).d.y;
                    let dly1 = -(*p1).d.x;

                    (*p1).dm.x = (dlx0 + dlx1) * 0.5;
                    (*p1).dm.y = (dly0 + dly1) * 0.5;
                    let dmr2 = (*p1).dm.x * (*p1).dm.x + (*p1).dm.y * (*p1).dm.y;

                    if dmr2 > 0.000001 {
                        let mut scale = 1.0 / dmr2;
                        if scale > 600.0 {
                            scale = 600.0;
                        }
                        (*p1).dm.x *= scale;
                        (*p1).dm.y *= scale;
                    }

                    (*p1).flags &= PointFlags::PT_CORNER;

                    let cross = (*p1).d.x * (*p0).d.y - (*p0).d.x * (*p1).d.y;
                    if cross > 0.0 {
                        nleft += 1;
                        (*p1).flags |= PointFlags::PT_LEFT;
                    }

                    let limit = (((*p0).len.min((*p1).len) as f32) * iw).max(1.01);
                    if (dmr2 * limit * limit) < 1.0 {
                        (*p1).flags |= PointFlags::PR_INNERBEVEL;
                    }

                    if (*p1).flags.contains(PointFlags::PT_CORNER) {
                        if (dmr2 * miter_limit * miter_limit) < 1.0
                            || line_join == LineJoin::Bevel
                            || line_join == LineJoin::Round
                        {
                            (*p1).flags |= PointFlags::PT_BEVEL;
                        }
                    }

                    if (*p1).flags.contains(PointFlags::PT_BEVEL)
                        || (*p1).flags.contains(PointFlags::PR_INNERBEVEL)
                    {
                        path.num_bevel += 1;
                    }

                    p0 = p1;
                    p1 = p1.add(1);
                }

                path.convex = nleft == path.count;
            }
        }
    }

    pub(crate) fn expand_stroke(
        &mut self,
        mut w: f32,
        fringe: f32,
        line_cap: LineCap,
        line_join: LineJoin,
        miter_limit: f32,
        tess_tol: f32,
        paths_slice: &mut Vec<PathSlice>,
    ) {
        let aa = fringe;
        let mut u0 = 0.0;
        let mut u1 = 1.0;
        let ncap = curve_divs(w, PI, tess_tol);

        w += aa * 0.5;

        if aa == 0.0 {
            u0 = 0.5;
            u1 = 0.5;
        }

        self.calculate_joins(w, line_join, miter_limit);

        let mut cverts = 0;
        for path in &self.paths {
            let loop_ = path.closed;
            if line_join == LineJoin::Round {
                cverts += (path.count + path.num_bevel * (ncap + 2) + 1) * 2;
            } else {
                cverts += (path.count + path.num_bevel * 5 + 1) * 2;
                if !loop_ {
                    if line_cap == LineCap::Round {
                        cverts += (ncap * 2 + 2) * 2;
                    } else {
                        cverts += (3 + 3) * 2;
                    }
                }
            }
        }

        unsafe {
            let (origin, mut vertices) = self.alloc_temp_vertices(cverts);
            if vertices.is_null() {
                return;
            }

            paths_slice.resize(self.paths.len(), Default::default());
            for (path, path_slice) in self.paths.iter_mut().zip(paths_slice.iter_mut()) {
                let pts = &mut self.points[path.first] as *mut VPoint;
                let loop_ = path.closed;
                let mut dst = vertices;
                path_slice.offset = ptrdistance(origin, dst);
                path_slice.num_fill = 0;

                let (mut p0, mut p1, s, e) = if loop_ {
                    (pts.offset(path.count as isize - 1), pts, 0, path.count)
                } else {
                    (pts, pts.add(1), 1, path.count - 1)
                };

                if !loop_ {
                    let mut d = Point::new((*p1).xy.x - (*p0).xy.x, (*p1).xy.y - (*p0).xy.y);
                    d.normalize();
                    match line_cap {
                        LineCap::Butt => {
                            dst = butt_cap_start(
                                dst,
                                p0.as_mut().unwrap(),
                                d.x,
                                d.y,
                                w,
                                -aa * 0.5,
                                aa,
                                u0,
                                u1,
                            )
                        }
                        LineCap::Square => {
                            dst = butt_cap_start(
                                dst,
                                p0.as_mut().unwrap(),
                                d.x,
                                d.y,
                                w,
                                w - aa,
                                aa,
                                u0,
                                u1,
                            )
                        }
                        LineCap::Round => {
                            dst = round_cap_start(
                                dst,
                                p0.as_mut().unwrap(),
                                d.x,
                                d.y,
                                w,
                                ncap,
                                aa,
                                u0,
                                u1,
                            )
                        }
                    }
                }

                for _ in s..e {
                    if (*p1).flags.contains(PointFlags::PT_BEVEL)
                        || (*p1).flags.contains(PointFlags::PR_INNERBEVEL)
                    {
                        if line_join == LineJoin::Round {
                            dst = round_join(
                                dst,
                                p0.as_mut().unwrap(),
                                p1.as_mut().unwrap(),
                                w,
                                w,
                                u0,
                                u1,
                                ncap,
                                aa,
                            );
                        } else {
                            dst = bevel_join(
                                dst,
                                p0.as_mut().unwrap(),
                                p1.as_mut().unwrap(),
                                w,
                                w,
                                u0,
                                u1,
                                aa,
                            );
                        }
                    } else {
                        *dst = Vertex::new(
                            (*p1).xy.x + ((*p1).dm.x * w),
                            (*p1).xy.y + ((*p1).dm.y * w),
                            u0,
                            1.0,
                        );
                        dst = dst.add(1);

                        *dst = Vertex::new(
                            (*p1).xy.x - ((*p1).dm.x * w),
                            (*p1).xy.y - ((*p1).dm.y * w),
                            u1,
                            1.0,
                        );
                        dst = dst.add(1);
                    }
                    p0 = p1;
                    p1 = p1.add(1);
                }

                if loop_ {
                    let v0 = vertices;
                    let v1 = vertices.add(1);

                    *dst = Vertex::new((*v0).x, (*v0).y, u0, 1.0);
                    dst = dst.add(1);

                    *dst = Vertex::new((*v1).x, (*v1).y, u1, 1.0);
                    dst = dst.add(1);
                } else {
                    let mut d = Point::new((*p1).xy.x - (*p0).xy.x, (*p1).xy.y - (*p0).xy.y);
                    d.normalize();
                    match line_cap {
                        LineCap::Butt => {
                            dst = butt_cap_end(
                                dst,
                                p1.as_mut().unwrap(),
                                d.x,
                                d.y,
                                w,
                                -aa * 0.5,
                                aa,
                                u0,
                                u1,
                            );
                        }
                        LineCap::Square => {
                            dst = butt_cap_end(
                                dst,
                                p1.as_mut().unwrap(),
                                d.x,
                                d.y,
                                w,
                                w - aa,
                                aa,
                                u0,
                                u1,
                            );
                        }
                        LineCap::Round => {
                            dst = round_cap_end(
                                dst,
                                p1.as_mut().unwrap(),
                                d.x,
                                d.y,
                                w,
                                ncap,
                                aa,
                                u0,
                                u1,
                            );
                        }
                    }
                }

                path_slice.num_stroke = ptrdistance(vertices, dst);
                vertices = dst;
            }
        }
    }

    pub(crate) fn expand_fill(
        &mut self,
        w: f32,
        line_join: LineJoin,
        miter_limit: f32,
        fringe_width: f32,
        paths_slice: &mut Vec<PathSlice>,
    ) -> Option<usize> {
        let aa = fringe_width;
        let fringe = w > 0.0;

        self.calculate_joins(w, line_join, miter_limit);

        let convex = self.paths.len() == 1 && self.paths[0].convex;
        let mut cverts = if convex { 0 } else { 4 };
        for path in &self.paths {
            cverts += path.count + path.num_bevel + 1;
            if fringe {
                cverts += (path.count + path.num_bevel * 5 + 1) * 2;
            }
        }

        unsafe {
            let (origin, mut vertices) = self.alloc_temp_vertices(cverts);
            if vertices.is_null() {
                return None;
            }

            paths_slice.resize(self.paths.len(), Default::default());
            for (path, path_slice) in self.paths.iter_mut().zip(paths_slice.iter_mut()) {
                let pts = &mut self.points[path.first] as *mut VPoint;
                let woff = 0.5 * aa;
                let mut dst = vertices;

                path_slice.offset = ptrdistance(origin, dst);

                if fringe {
                    let mut p0 = pts.offset(path.count as isize - 1);
                    let mut p1 = pts;
                    for _ in 0..path.count {
                        if (*p1).flags.contains(PointFlags::PT_BEVEL) {
                            let dlx0 = (*p0).d.y;
                            let dly0 = -(*p0).d.x;
                            let dlx1 = (*p1).d.y;
                            let dly1 = -(*p1).d.x;
                            if (*p1).flags.contains(PointFlags::PT_LEFT) {
                                let lx = (*p1).xy.x + (*p1).dm.x * woff;
                                let ly = (*p1).xy.y + (*p1).dm.y * woff;
                                *dst = Vertex::new(lx, ly, 0.5, 1.0);
                                dst = dst.add(1);
                            } else {
                                let lx0 = (*p1).xy.x + dlx0 * woff;
                                let ly0 = (*p1).xy.y + dly0 * woff;
                                let lx1 = (*p1).xy.x + dlx1 * woff;
                                let ly1 = (*p1).xy.y + dly1 * woff;

                                *dst = Vertex::new(lx0, ly0, 0.5, 1.0);
                                dst = dst.add(1);

                                *dst = Vertex::new(lx1, ly1, 0.5, 1.0);
                                dst = dst.add(1);
                            }
                        } else {
                            *dst = Vertex::new(
                                (*p1).xy.x + ((*p1).dm.x * woff),
                                (*p1).xy.y + ((*p1).dm.y * woff),
                                0.5,
                                1.0,
                            );
                            dst = dst.add(1);
                        }

                        p0 = p1;
                        p1 = p1.add(1);
                    }
                } else {
                    for j in 0..path.count {
                        let pt = pts.add(j);
                        *dst = Vertex::new((*pt).xy.x, (*pt).xy.y, 0.5, 1.0);
                        dst = dst.add(1);
                    }
                }

                path_slice.num_fill = ptrdistance(vertices, dst);
                vertices = dst;

                if fringe {
                    let mut lw = w + woff;
                    let rw = w - woff;
                    let mut lu = 0.0;
                    let ru = 1.0;
                    let mut dst = vertices;
                    if convex {
                        lw = woff;
                        lu = 0.5;
                    }

                    let mut p0 = pts.offset(path.count as isize - 1);
                    let mut p1 = pts;

                    for _ in 0..path.count {
                        if (*p1).flags.contains(PointFlags::PT_BEVEL)
                            || (*p1).flags.contains(PointFlags::PR_INNERBEVEL)
                        {
                            dst = bevel_join(
                                dst,
                                p0.as_mut().unwrap(),
                                p1.as_mut().unwrap(),
                                lw,
                                rw,
                                lu,
                                ru,
                                fringe_width,
                            );
                        } else {
                            *dst = Vertex::new(
                                (*p1).xy.x + ((*p1).dm.x * lw),
                                (*p1).xy.y + ((*p1).dm.y * lw),
                                lu,
                                1.0,
                            );
                            dst = dst.add(1);

                            *dst = Vertex::new(
                                (*p1).xy.x - ((*p1).dm.x * rw),
                                (*p1).xy.y - ((*p1).dm.y * rw),
                                ru,
                                1.0,
                            );
                            dst = dst.add(1);
                        }
                        p0 = p1;
                        p1 = p1.add(1);
                    }

                    let v0 = vertices;
                    let v1 = vertices.add(1);

                    *dst = Vertex::new((*v0).x, (*v0).y, lu, 1.0);
                    dst = dst.add(1);

                    *dst = Vertex::new((*v1).x, (*v1).y, ru, 1.0);
                    dst = dst.add(1);

                    path_slice.num_stroke = ptrdistance(vertices, dst);
                    vertices = dst;
                } else {
                    path_slice.num_stroke = 0;
                }
            }
            if !convex {
                // add bounds to vertices
                *vertices = Vertex::new(self.bounds.max.x, self.bounds.max.y, 0.5, 1.0);
                *vertices.add(1) = Vertex::new(self.bounds.max.x, self.bounds.min.y, 0.5, 1.0);
                *vertices.add(2) = Vertex::new(self.bounds.min.x, self.bounds.max.y, 0.5, 1.0);
                *vertices.add(3) = Vertex::new(self.bounds.min.x, self.bounds.min.y, 0.5, 1.0);
                return Some(ptrdistance(origin, vertices));
            }
            return None;
        }
    }

    #[cfg(feature = "wirelines")]
    pub(crate) fn expand_lines(&mut self, paths_slice: &mut Vec<PathSlice>) {
        unsafe {
            let cverts = self
                .paths
                .iter()
                .fold(0, |acc, e| acc + e.count + (e.closed as usize));
            let (origin, mut vertices) = self.alloc_temp_vertices(cverts);
            if vertices.is_null() {
                return;
            }
            paths_slice.resize(self.paths.len(), Default::default());
            for (path, path_slice) in self.paths.iter_mut().zip(paths_slice.iter_mut()) {
                let pts = &self.points[path.first..path.first + path.count];
                let mut dst = vertices;
                path_slice.offset = ptrdistance(origin, dst);
                path_slice.num_fill = 0;
                for pt in pts {
                    *dst = Vertex::new(pt.xy.x, pt.xy.y, 0.5, 1.0);
                    dst = dst.add(1);
                }
                if path.closed {
                    let v0 = &*vertices;
                    *dst = Vertex::new(v0.x, v0.y, 0.5, 1.0);
                    dst = dst.add(1);
                }
                path_slice.num_stroke = ptrdistance(vertices, dst);
                vertices = dst;
            }
        }
    }
}

fn triangle_area(a: &VPoint, b: &VPoint, c: &VPoint) -> f32 {
    let a = &a.xy;
    let b = &b.xy;
    let c = &c.xy;
    let abx = b.x - a.x;
    let aby = b.y - a.y;
    let acx = c.x - a.x;
    let acy = c.y - a.y;
    acx * aby - abx * acy
}

fn poly_area(pts: &[VPoint]) -> f32 {
    let mut area = 0.0;
    for i in 2..pts.len() {
        let a = &pts[0];
        let b = &pts[i - 1];
        let c = &pts[i];
        area += triangle_area(a, b, c);
    }
    area * 0.5
}

fn poly_reverse(pts: &mut [VPoint]) {
    let mut i = 0;
    let mut j = pts.len() as i32 - 1;
    while i < j {
        pts.swap(i as usize, j as usize);
        i += 1;
        j -= 1;
    }
}

fn curve_divs(r: f32, arc: f32, tess_tol: f32) -> usize {
    let da = (r / (r + tess_tol)).acos() * 2.0;
    ((arc / da).ceil() as i32).max(2) as usize
}
