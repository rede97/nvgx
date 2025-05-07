use super::*;

impl Path {
    pub fn identity(&mut self) {
        self.xform = Transform::identity();
    }

    pub fn translate(&mut self, tx: f32, ty: f32) {
        self.xform = self.xform.pre_multiply(Transform::translate(tx, ty));
    }

    pub fn scale(&mut self, sx: f32, sy: f32) {
        self.xform = self.xform.pre_multiply(Transform::scale(sx, sy));
    }

    pub fn rotate(&mut self, a: f32) {
        self.xform = self.xform.pre_multiply(Transform::rotate(a));
    }

    pub fn skew_x(&mut self, a: f32) {
        self.xform = self.xform.pre_multiply(Transform::skew_x(a));
    }

    pub fn skew_y(&mut self, a: f32) {
        self.xform = self.xform.pre_multiply(Transform::skew_y(a));
    }

    pub fn save(&mut self) {
        self.xforms.push(self.xform);
    }

    pub fn restore(&mut self) {
        self.xform = self.xforms.pop().unwrap();
    }
}
