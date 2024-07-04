fn main() {
    yarn_build();

    let outdir = std::env::var("OUT_DIR").expect("Missing OUT_DIR");
    let outdir = std::path::PathBuf::from(outdir);

    compile_js("js/dist/browser.js", &outdir.join("bootcode-browser.jsc"));
    compile_js("js/dist/nodejs.js", &outdir.join("bootcode-nodejs.jsc"));
}

fn compile_js(src_file: &str, out_file: &std::path::Path) {
    let src = std::fs::read_to_string(src_file).expect("failed to read bootcode.js");
    let bytecode = qjsbind::compile(&src, "<bootcode>").expect("failed to compile the bootcode");
    std::fs::write(out_file, bytecode).expect("failed to write bytecode to file");
}

fn yarn_build() {
    println!("cargo:rerun-if-changed=js/src");
    println!("cargo:rerun-if-changed=js/package.json");
    println!("cargo:rerun-if-env-changed=PROFILE");
    let mut cmd = std::process::Command::new("bash");
    if std::env::var("PROFILE").unwrap() == "release" {
        cmd.arg("-c").arg("cd js && yarn && yarn build");
    } else {
        cmd.arg("-c").arg("cd js && yarn && yarn build:debug");
    }
    cmd.status().expect("failed to run yarn");
}
