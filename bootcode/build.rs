use qjs_sys as js;

fn main() {
    println!("cargo:rerun-if-changed=bootcode.js");
    let outdir = std::env::var("OUT_DIR").expect("Missing CARGO_MANIFEST_DIR");
    let outdir = std::path::PathBuf::from(outdir);
    let bytecode = js::compile(include_str!("bootcode.js"), "<bootcode>")
        .expect("Failed to compile the bootcode");
    std::fs::write(outdir.join("bootcode.jsc"), bytecode)
        .expect("Failed to write bytecode to file");
}
