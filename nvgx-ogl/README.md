# OpenGL Impl: NVGX Pure-rust NanoVG

* Homepage: [nvgx-ogl](https://github.com/rede97/nvgx/tree/master/nvgx-ogl) 
* crates.io: [nvgx](https://crates.io/crates/nvgx)

## Note
The current OpenGL backend API is based on OpenGL 3.1, while WebGL 2.0 (GLES 3.0) compatibility has been considered but not yet tested. The fragmentation and problematic nature of GPU driver implementations across different vendors remain significant issues, as discussed in the [Glium post-mortem](https://users.rust-lang.org/t/glium-post-mortem/7063 ). With OpenGL 4.0+ APIs being gradually replaced by the more standardized Vulkan, the OpenGL backend should prioritize the relatively stable and unified OpenGL 3.1 standard. Although OpenGL 4.0 has been in existence for 15 years and is supported by the vast majority of modern GPUs, backward compatibility concerns for OpenGL 3.1 are largely obsolete for contemporary hardware. Earlier versions like OpenGL 2.0+ are no longer supported due to their lack of instanced rendering APIs and the excessive complexity of cross-version API and shader compatibility, which introduces unnecessary technical debt.
