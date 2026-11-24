use std::{env, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=../runtime/engineer");
    println!("cargo:rerun-if-changed=../scripts/build_engineer_bridge.py");
    for name in
        ["JAVA_HOME", "ANDROID_HOME", "ANDROID_SDK_ROOT", "ANDROID_NDK_ROOT", "LOCALAPPDATA"]
    {
        println!("cargo:rerun-if-env-changed={name}");
    }
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("android") {
        return;
    }
    let sdk = env::var_os("ANDROID_HOME")
        .or_else(|| env::var_os("ANDROID_SDK_ROOT"))
        .map(PathBuf::from)
        .or_else(|| env::var_os("LOCALAPPDATA").map(|path| PathBuf::from(path).join("Android/Sdk")))
        .expect("Engineer bridge requires Android SDK");
    let ndk = env::var_os("ANDROID_NDK_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| sdk.join("ndk/30.0.16248370"));
    let target = env::var("TARGET").unwrap();
    let root =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).parent().unwrap().to_owned();
    let status = Command::new("python")
        .arg(root.join("scripts/build_engineer_bridge.py"))
        .args(["--target", &target, "--sdk"])
        .arg(&sdk)
        .arg("--ndk")
        .arg(ndk)
        .status()
        .expect("Cannot run engineer bridge builder");
    assert!(status.success(), "Engineer bridge build failed");
    println!(
        "cargo:rustc-env=OMK_ENGINEER_DEX={}",
        root.join("target/engineer-bridge").join(target).join("classes.dex").display()
    );
    // On x86_64 the inline property hook uses Dobby's trampoline (the AArch64
    // path patches the export directly). Link the same prefab already fetched
    // for the engineer bridge so both ABIs keep the same hook coverage.
    if target == "x86_64-linux-android" {
        let deps = root.join("target/engineer-deps");
        let prefab = deps.join("dobby-prefab-251f48ae2168");
        let library_dir = prefab.join("prefab/modules/dobby/libs/android.x86_64");
        assert!(library_dir.join("libdobby.a").is_file(), "Dobby x86_64 library missing");
        println!("cargo:rustc-link-search=native={}", library_dir.display());
        println!("cargo:rustc-link-lib=static=dobby");
        // Dobby is C++; the Rust cdylib does not pull in the NDK C++ runtime.
        println!("cargo:rustc-link-lib=static=c++_static");
        println!("cargo:rustc-link-lib=c++abi");
        println!("cargo:rustc-link-lib=dl");
    }
}
