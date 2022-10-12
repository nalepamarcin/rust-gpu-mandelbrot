
fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();

    let mut file = std::fs::File::create(&std::path::Path::new(&out_dir).join("bindings.rs")).unwrap();

    gl_generator::Registry::new(
        gl_generator::Api::Gl,
        (4, 6),
        gl_generator::Profile::Core,
        gl_generator::Fallbacks::All,
        [
            "GL_ARB_gl_spirv"
        ],
    )
    .write_bindings(gl_generator::GlobalGenerator, &mut file)
    .unwrap();
}
