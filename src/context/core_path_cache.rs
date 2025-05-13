use std::cell::RefMut;

use crate::{LineJoin, Paint};
use crate::{PathMutInner, RendererDevice};

use super::*;

impl<R: RendererDevice> Context<R> {
    #[inline]
    pub(crate) fn expand_fill_path<FE: FlattenExpandPath>(
        path_cache: &mut FE,
        paint: &Paint,
        dist_tol: f32,
        tess_tol: f32,
        fringe_width: f32,
    ) -> Option<usize> {
        path_cache.flatten_paths(dist_tol, tess_tol);
        return if paint.antialias && paint.antialias {
            path_cache.expand_fill(fringe_width, LineJoin::Miter, 2.4, fringe_width)
        } else {
            path_cache.expand_fill(0.0, LineJoin::Miter, 2.4, fringe_width)
        };
    }

    #[inline]
    pub(crate) fn expand_stroke_path<FE: FlattenExpandPath>(
        path_cache: &mut FE,
        antialias: bool,
        stroke_width: f32,
        paint: &Paint,
        dist_tol: f32,
        tess_tol: f32,
        fringe_width: f32,
    ) {
        path_cache.flatten_paths(dist_tol, tess_tol);
        if antialias {
            path_cache.expand_stroke(
                stroke_width * 0.5,
                fringe_width,
                paint.line_cap,
                paint.line_join,
                paint.miter_limit,
                tess_tol,
            );
        } else {
            path_cache.expand_stroke(
                stroke_width * 0.5,
                0.0,
                paint.line_cap,
                paint.line_join,
                paint.miter_limit,
                tess_tol,
            );
        }
    }

    #[cfg(feature = "wirelines")]
    #[inline]
    pub(crate) fn expand_wirelines_path<FE: FlattenExpandPath>(
        path_cache: &mut FE,
        dist_tol: f32,
        tess_tol: f32,
    ) {
        path_cache.flatten_paths(dist_tol, tess_tol);
        path_cache.expand_lines();
    }
}

#[derive(Default)]
pub(crate) struct PathCommandsWithCache {
    pub path: PathCommands,
    pub cache: PathCache,
    pub draw_paths_slice: Vec<PathSlice>,
}

impl FlattenExpandPath for PathCommandsWithCache {
    #[inline]
    fn path_commands(&self) -> &PathCommands {
        return &self.path;
    }

    #[inline]
    fn flatten_paths(&mut self, dist_tol: f32, tess_tol: f32) {
        self.cache
            .flatten_paths(&self.path.commands, dist_tol, tess_tol);
    }

    #[inline]
    fn expand_fill(
        &mut self,
        w: f32,
        line_join: LineJoin,
        miter_limit: f32,
        fringe_width: f32,
    ) -> Option<usize> {
        return self.cache.expand_fill(
            w,
            line_join,
            miter_limit,
            fringe_width,
            &mut self.draw_paths_slice,
        );
    }

    #[inline]
    fn expand_stroke(
        &mut self,
        w: f32,
        fringe: f32,
        line_cap: LineCap,
        line_join: LineJoin,
        miter_limit: f32,
        tess_tol: f32,
    ) {
        self.cache.expand_stroke(
            w,
            fringe,
            line_cap,
            line_join,
            miter_limit,
            tess_tol,
            &mut self.draw_paths_slice,
        );
    }

    #[inline]
    #[cfg(feature = "wirelines")]
    fn expand_lines(&mut self) {
        self.cache.expand_lines(&mut self.draw_paths_slice);
    }

    #[inline]
    fn get_fill_slice(&self) -> &[PathSlice] {
        return &self.draw_paths_slice;
    }

    #[inline]
    fn get_stroke_slice(&self) -> &[PathSlice] {
        return &self.draw_paths_slice;
    }

    #[inline]
    #[cfg(feature = "wirelines")]
    fn get_lines_slice(&self) -> &[PathSlice] {
        return &self.draw_paths_slice;
    }
}

impl Deref for PathCommandsWithCache {
    type Target = PathCommands;
    fn deref(&self) -> &Self::Target {
        return &self.path;
    }
}

impl DerefMut for PathCommandsWithCache {
    fn deref_mut(&mut self) -> &mut Self::Target {
        return &mut self.path;
    }
}

pub(crate) struct PathRefWithCache<'a, R: RendererDevice> {
    pub path: &'a PathCommands,
    pub path_mut_inner: RefMut<'a, PathMutInner<R>>,
    pub cache: PathCache,
}

impl<'a, R: RendererDevice> PathRefWithCache<'a, R> {
    pub(crate) fn new(path: &'a Path<R>) -> Self {
        Self {
            path: &path.path_comands,
            path_mut_inner: path.inner.borrow_mut(),
            cache: PathCache::default(),
        }
    }
}

impl<'a, R: RendererDevice> FlattenExpandPath for PathRefWithCache<'a, R> {
    #[inline]
    fn path_commands(&self) -> &PathCommands {
        &self.path
    }

    #[inline]
    fn flatten_paths(&mut self, dist_tol: f32, tess_tol: f32) {
        self.cache
            .flatten_paths(&self.path.commands, dist_tol, tess_tol);
    }

    #[inline]
    fn expand_fill(
        &mut self,
        w: f32,
        line_join: LineJoin,
        miter_limit: f32,
        fringe_width: f32,
    ) -> Option<usize> {
        return self.cache.expand_fill(
            w,
            line_join,
            miter_limit,
            fringe_width,
            &mut self.path_mut_inner.draw_slice.fill,
        );
    }

    #[inline]
    fn expand_stroke(
        &mut self,
        w: f32,
        fringe: f32,
        line_cap: LineCap,
        line_join: LineJoin,
        miter_limit: f32,
        tess_tol: f32,
    ) {
        self.cache.expand_stroke(
            w,
            fringe,
            line_cap,
            line_join,
            miter_limit,
            tess_tol,
            &mut self.path_mut_inner.draw_slice.stroke,
        );
    }

    #[cfg(feature = "wirelines")]
    #[inline]
    fn expand_lines(&mut self) {
        self.cache
            .expand_lines(&mut self.path_mut_inner.draw_slice.lines);
    }

    #[inline]
    fn get_fill_slice(&self) -> &[PathSlice] {
        return &self.path_mut_inner.draw_slice.fill;
    }

    #[inline]
    fn get_stroke_slice(&self) -> &[PathSlice] {
        return &self.path_mut_inner.draw_slice.stroke;
    }

    #[cfg(feature = "wirelines")]
    #[inline]
    fn get_lines_slice(&self) -> &[PathSlice] {
        return &self.path_mut_inner.draw_slice.lines;
    }
}
