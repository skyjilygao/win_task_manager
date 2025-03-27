use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    if env::var("CARGO_CFG_TARGET_FAMILY").unwrap() == "windows" {
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let rc_path = PathBuf::from("resources/task_manager.rc");
        let res_path = out_dir.join("task_manager.res");

        // Compile the .rc file
        let status = Command::new("rc.exe")
            .arg("/fo")
            .arg(&res_path)
            .arg(&rc_path)
            .status()
            .expect("Failed to compile resources");

        if !status.success() {
            panic!("Resource compilation failed");
        }

        // Link the compiled resources
        println!("cargo:rustc-link-arg-bins={}", res_path.display());
    }
}