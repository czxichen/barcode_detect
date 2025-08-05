fn main() {
    println!("cargo:rerun-if-env-changed=detect.cpp");

    let ncnn_path = std::env::var("NCNN_PATH").unwrap_or_default();
    if ncnn_path.is_empty() {
        panic!("ncnn path is empty")
    }

    build_ncnn(&ncnn_path);

    println!("cargo:rustc-link-lib=static=ncnn");
    println!("cargo:rustc-link-lib=static=detect");
    println!("cargo:rustc-link-lib=c++");
}

#[cfg(target_os = "macos")]
fn build_ncnn(path: &str) {
    let path = cmake::Config::new(".")
        .define("NCNN_SOURCE_DIR", path)
        .define("CMAKE_OSX_DEPLOYMENT_TARGET", "11.0")
        .profile("Release")
        .build();

    println!("cargo:rustc-link-search={}/lib", path.to_string_lossy());
}

#[cfg(target_os = "android")]
fn build_ncnn(path: &str) {
    let arch = match target.as_str() {
        "i686-linux-android" => "x86",
        "x86_64-linux-android" => "x86_64",
        "aarch64-linux-android" => "arm64-v8a",
        "armv7-linux-androideabi" => "armeabi-v7a",
        _ => "",
    };

    let path = cmake::Config::new(".")
        .define("NCNN_SOURCE_DIR", path)
        .define(
            "CMAKE_TOOLCHAIN_FILE",
            std::env::var("ANDROID_NDK_ROOT").unwrap_or_default()
                + "/build/cmake/android.toolchain.cmake",
        )
        .define("ANDROID_ABI", arch)
        .define("ANDROID_PLATFORM", "android-21")
        .profile("Release")
        .build();

    println!("cargo:rustc-link-search={}/lib", path.to_string_lossy());
}

// export ANDROID_NDK_ROOT=$ANDROID_HOME/ndk/21.4.7075529
// cmake -DNCNN_SOURCE_DIR=/tmp/ncnn -DCMAKE_INSTALL_PREFIX=install ..
// cmake -DNCNN_SOURCE_DIR=/tmp/ncnn -DCMAKE_INSTALL_PREFIX=install -DANDROID_ABI=arm64-v8a -DANDROID_PLATFORM=android-21 -DCMAKE_TOOLCHAIN_FILE="$ANDROID_HOME/ndk/21.4.7075529/build/cmake/android.toolchain.cmake" ..
