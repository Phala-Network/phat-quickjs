fn main() {
    yarn_build();

    let src_file = std::path::PathBuf::from("js/dist/index.js");
    let src = std::fs::read_to_string(&src_file).expect("failed to read bootcode.js");
    let outdir = std::env::var("OUT_DIR").expect("Missing OUT_DIR");
    let outdir = std::path::PathBuf::from(outdir);
    let bytecode = qjsbind::compile(&src, "<bootcode>").expect("failed to compile the bootcode");
    std::fs::write(outdir.join("bootcode.jsc"), bytecode)
        .expect("failed to write bytecode to file");
}

fn yarn_build() {
    println!("cargo:rerun-if-changed=js/src");
    let mut cmd = std::process::Command::new("bash");
    cmd.arg("-c").arg("cd js && yarn && yarn build");
    cmd.status().expect("failed to run yarn");
}
