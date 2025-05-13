#version 150 core

uniform vec2 viewSize;
in vec2 vertex;
in vec2 tcoord;
in vec4 model_matrix_0;
in vec2 model_matrix_1;
out vec2 ftcoord;
out vec2 fpos;

void main(void) {
    mat2 model = mat2(
        model_matrix_0.xy, model_matrix_0.zw
    );
    vec2 v = (model * vertex) + model_matrix_1;
    ftcoord = tcoord;
    fpos = v.xy;
    gl_Position = vec4(2.0 * v.x / viewSize.x - 1.0, 1.0 - 2.0 * v.y / viewSize.y, 0, 1);
}
