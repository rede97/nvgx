use super::{PointFlags, VPoint};
use super::Vertex;
use clamped::Clamp;
use std::f32::consts::PI;

fn choose_bevel(bevel: bool, p0: &mut VPoint, p1: &mut VPoint, w: f32) -> (f32, f32, f32, f32) {
    if bevel {
        let x0 = p1.xy.x + p0.d.y * w;
        let y0 = p1.xy.y - p0.d.x * w;
        let x1 = p1.xy.x + p1.d.y * w;
        let y1 = p1.xy.y - p1.d.x * w;
        (x0, y0, x1, y1)
    } else {
        let x0 = p1.xy.x + p1.dm.x * w;
        let y0 = p1.xy.y + p1.dm.y * w;
        let x1 = p1.xy.x + p1.dm.x * w;
        let y1 = p1.xy.y + p1.dm.y * w;
        (x0, y0, x1, y1)
    }
}

pub(super) unsafe fn round_join(
    mut dst: *mut Vertex,
    p0: &mut VPoint,
    p1: &mut VPoint,
    lw: f32,
    rw: f32,
    lu: f32,
    ru: f32,
    ncap: usize,
    _fringe: f32,
) -> *mut Vertex {
    let dlx0 = p0.d.y;
    let dly0 = -p0.d.x;
    let dlx1 = p1.d.y;
    let dly1 = -p1.d.x;

    if p1.flags.contains(PointFlags::PT_LEFT) {
        let (lx0, ly0, lx1, ly1) =
            choose_bevel(p1.flags.contains(PointFlags::PR_INNERBEVEL), p0, p1, lw);
        let a0 = -dly0.atan2(-dlx0);
        let mut a1 = -dly1.atan2(-dlx1);
        if a1 > a0 {
            a1 -= PI * 2.0;
        }

        *dst = Vertex::new(lx0, ly0, lu, 1.0);
        dst = dst.add(1);

        *dst = Vertex::new(p1.xy.x - dlx0 * rw, p1.xy.y - dly0 * rw, ru, 1.0);
        dst = dst.add(1);

        let n = ((((a0 - a1) / PI) * (ncap as f32)).ceil() as i32).clamped(2, ncap as i32);
        for i in 0..n {
            let u = (i as f32) / ((n - 1) as f32);
            let a = a0 + u * (a1 - a0);
            let rx = p1.xy.x + a.cos() * rw;
            let ry = p1.xy.y + a.sin() * rw;

            *dst = Vertex::new(p1.xy.x, p1.xy.y, 0.5, 1.0);
            dst = dst.add(1);

            *dst = Vertex::new(rx, ry, ru, 1.0);
            dst = dst.add(1);
        }

        *dst = Vertex::new(lx1, ly1, lu, 1.0);
        dst = dst.add(1);

        *dst = Vertex::new(p1.xy.x - dlx1 * rw, p1.xy.y - dly1 * rw, ru, 1.0);
        dst = dst.add(1);
    } else {
        let (rx0, ry0, rx1, ry1) =
            choose_bevel(p1.flags.contains(PointFlags::PR_INNERBEVEL), p0, p1, -rw);
        let a0 = dly0.atan2(dlx0);
        let mut a1 = dly1.atan2(dlx1);
        if a1 < a0 {
            a1 += PI * 2.0;
        }

        *dst = Vertex::new(p1.xy.x + dlx0 * rw, p1.xy.y + dly0 * rw, lu, 1.0);
        dst = dst.add(1);

        *dst = Vertex::new(rx0, ry0, ru, 1.0);
        dst = dst.add(1);

        let n = ((((a0 - a1) / PI) * (ncap as f32)).ceil() as i32).clamped(2, ncap as i32);
        for i in 0..n {
            let u = (i as f32) / ((n - 1) as f32);
            let a = a0 + u * (a1 - a0);
            let lx = p1.xy.x + a.cos() * lw;
            let ly = p1.xy.y + a.cos() * lw;

            *dst = Vertex::new(lx, ly, lu, 1.0);
            dst = dst.add(1);

            *dst = Vertex::new(p1.xy.x, p1.xy.y, 0.5, 1.0);
            dst = dst.add(1);
        }

        *dst = Vertex::new(p1.xy.x + dlx1 * rw, p1.xy.y + dly1 * rw, lu, 1.0);
        dst = dst.add(1);

        *dst = Vertex::new(rx1, ry1, ru, 1.0);
        dst = dst.add(1);
    }

    dst
}

pub(super) unsafe fn bevel_join(
    mut dst: *mut Vertex,
    p0: &mut VPoint,
    p1: &mut VPoint,
    lw: f32,
    rw: f32,
    lu: f32,
    ru: f32,
    _fringe: f32,
) -> *mut Vertex {
    let dlx0 = p0.d.y;
    let dly0 = -p0.d.x;
    let dlx1 = p1.d.y;
    let dly1 = -p1.d.x;

    if p1.flags.contains(PointFlags::PT_LEFT) {
        let (lx0, ly0, lx1, ly1) =
            choose_bevel(p1.flags.contains(PointFlags::PR_INNERBEVEL), p0, p1, lw);

        *dst = Vertex::new(lx0, ly0, lu, 1.0);
        dst = dst.add(1);

        *dst = Vertex::new(p1.xy.x - dlx0 * rw, p1.xy.y - dly0 * rw, ru, 1.0);
        dst = dst.add(1);

        if p1.flags.contains(PointFlags::PT_BEVEL) {
            *dst = Vertex::new(lx0, ly0, lu, 1.0);
            dst = dst.add(1);

            *dst = Vertex::new(p1.xy.x - dlx0 * rw, p1.xy.y - dly0 * rw, ru, 1.0);
            dst = dst.add(1);

            *dst = Vertex::new(lx1, ly1, lu, 1.0);
            dst = dst.add(1);

            *dst = Vertex::new(p1.xy.x - dlx1 * rw, p1.xy.y - dly1 * rw, ru, 1.0);
            dst = dst.add(1);
        } else {
            let rx0 = p1.xy.x - p1.dm.x * rw;
            let ry0 = p1.xy.y - p1.dm.y * rw;

            *dst = Vertex::new(p1.xy.x, p1.xy.y, 0.5, 1.0);
            dst = dst.add(1);

            *dst = Vertex::new(p1.xy.x - dlx0 * rw, p1.xy.y - dly0 * rw, ru, 1.0);
            dst = dst.add(1);

            *dst = Vertex::new(rx0, ry0, ru, 1.0);
            dst = dst.add(1);

            *dst = Vertex::new(rx0, ry0, ru, 1.0);
            dst = dst.add(1);

            *dst = Vertex::new(p1.xy.x, p1.xy.y, 0.5, 1.0);
            dst = dst.add(1);

            *dst = Vertex::new(p1.xy.x - dlx1 * rw, p1.xy.y - dly1 * rw, ru, 1.0);
            dst = dst.add(1);
        }

        *dst = Vertex::new(lx1, ly1, lu, 1.0);
        dst = dst.add(1);

        *dst = Vertex::new(p1.xy.x - dlx1 * rw, p1.xy.y - dly1 * rw, ru, 1.0);
        dst = dst.add(1);
    } else {
        let (rx0, ry0, rx1, ry1) =
            choose_bevel(p1.flags.contains(PointFlags::PR_INNERBEVEL), p0, p1, -rw);

        *dst = Vertex::new(p1.xy.x + dlx0 * lw, p1.xy.y + dly0 * lw, lu, 1.0);
        dst = dst.add(1);

        *dst = Vertex::new(rx0, ry0, ru, 1.0);
        dst = dst.add(1);

        if p1.flags.contains(PointFlags::PT_BEVEL) {
            *dst = Vertex::new(p1.xy.x + dlx0 * lw, p1.xy.y + dly0 * lw, lu, 1.0);
            dst = dst.add(1);

            *dst = Vertex::new(rx0, ry0, ru, 1.0);
            dst = dst.add(1);

            *dst = Vertex::new(p1.xy.x + dlx1 * lw, p1.xy.y + dly1 * lw, lu, 1.0);
            dst = dst.add(1);

            *dst = Vertex::new(rx1, ry1, ru, 1.0);
            dst = dst.add(1);
        } else {
            let lx0 = p1.xy.x + p1.dm.x * lw;
            let ly0 = p1.xy.y + p1.dm.y * lw;

            *dst = Vertex::new(p1.xy.x + dlx0 * lw, p1.xy.y + dly0 * lw, lu, 1.0);
            dst = dst.add(1);

            *dst = Vertex::new(p1.xy.x, p1.xy.y, 0.5, 1.0);
            dst = dst.add(1);

            *dst = Vertex::new(lx0, ly0, lu, 1.0);
            dst = dst.add(1);

            *dst = Vertex::new(lx0, ly0, lu, 1.0);
            dst = dst.add(1);

            *dst = Vertex::new(p1.xy.x + dlx1 * lw, p1.xy.y + dly1 * lw, lu, 1.0);
            dst = dst.add(1);

            *dst = Vertex::new(p1.xy.x, p1.xy.y, 0.5, 1.0);
            dst = dst.add(1);
        }

        *dst = Vertex::new(p1.xy.x + dlx1 * lw, p1.xy.y + dly1 * lw, lu, 1.0);
        dst = dst.add(1);

        *dst = Vertex::new(rx1, ry1, ru, 1.0);
        dst = dst.add(1);
    }

    dst
}

pub(super) unsafe fn butt_cap_start(
    mut dst: *mut Vertex,
    p: &mut VPoint,
    dx: f32,
    dy: f32,
    w: f32,
    d: f32,
    aa: f32,
    u0: f32,
    u1: f32,
) -> *mut Vertex {
    let px = p.xy.x - dx * d;
    let py = p.xy.y - dy * d;
    let dlx = dy;
    let dly = -dx;

    *dst = Vertex::new(px + dlx * w - dx * aa, py + dly * w - dy * aa, u0, 0.0);
    dst = dst.add(1);

    *dst = Vertex::new(px - dlx * w - dx * aa, py - dly * w - dy * aa, u1, 0.0);
    dst = dst.add(1);

    *dst = Vertex::new(px + dlx * w, py + dly * w, u0, 1.0);
    dst = dst.add(1);

    *dst = Vertex::new(px - dlx * w, py - dly * w, u1, 1.0);
    dst = dst.add(1);

    dst
}

pub(super) unsafe fn butt_cap_end(
    mut dst: *mut Vertex,
    p: &mut VPoint,
    dx: f32,
    dy: f32,
    w: f32,
    d: f32,
    aa: f32,
    u0: f32,
    u1: f32,
) -> *mut Vertex {
    let px = p.xy.x - dx * d;
    let py = p.xy.y - dy * d;
    let dlx = dy;
    let dly = -dx;

    *dst = Vertex::new(px + dlx * w, py + dly * w, u0, 1.0);
    dst = dst.add(1);

    *dst = Vertex::new(px - dlx * w, py - dly * w, u1, 1.0);
    dst = dst.add(1);

    *dst = Vertex::new(px + dlx * w + dx * aa, py + dly * w + dy * aa, u0, 0.0);
    dst = dst.add(1);

    *dst = Vertex::new(px - dlx * w + dx * aa, py - dly * w + dy * aa, u1, 0.0);
    dst = dst.add(1);

    dst
}

pub(super) unsafe fn round_cap_start(
    mut dst: *mut Vertex,
    p: &mut VPoint,
    dx: f32,
    dy: f32,
    w: f32,
    ncap: usize,
    _aa: f32,
    u0: f32,
    u1: f32,
) -> *mut Vertex {
    let px = p.xy.x;
    let py = p.xy.y;
    let dlx = dy;
    let dly = -dx;

    for i in 0..ncap {
        let a = (i as f32) / ((ncap - 1) as f32) * PI;
        let ax = a.cos() * w;
        let ay = a.sin() * w;

        *dst = Vertex::new(px - dlx * ax - dx * ay, py - dly * ax - dy * ay, u0, 1.0);
        dst = dst.add(1);

        *dst = Vertex::new(px, py, 0.5, 1.0);
        dst = dst.add(1);
    }

    *dst = Vertex::new(px + dlx * w, py + dly * w, u0, 1.0);
    dst = dst.add(1);

    *dst = Vertex::new(px - dlx * w, py - dly * w, u1, 1.0);
    dst = dst.add(1);

    dst
}

pub(super) unsafe fn round_cap_end(
    mut dst: *mut Vertex,
    p: &mut VPoint,
    dx: f32,
    dy: f32,
    w: f32,
    ncap: usize,
    _aa: f32,
    u0: f32,
    u1: f32,
) -> *mut Vertex {
    let px = p.xy.x;
    let py = p.xy.y;
    let dlx = dy;
    let dly = -dx;

    *dst = Vertex::new(px + dlx * w, py + dly * w, u0, 1.0);
    dst = dst.add(1);

    *dst = Vertex::new(px - dlx * w, py - dly * w, u1, 1.0);
    dst = dst.add(1);

    for i in 0..ncap {
        let a = (i as f32) / ((ncap - 1) as f32) * PI;
        let ax = a.cos() * w;
        let ay = a.sin() * w;

        *dst = Vertex::new(px, py, 0.5, 1.0);
        dst = dst.add(1);

        *dst = Vertex::new(px - dlx * ax + dx * ay, py - dly * ax + dy * ay, u0, 1.0);
        dst = dst.add(1);
    }

    dst
}
