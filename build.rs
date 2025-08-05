fn main() {
    println!("cargo:rerun-if-env-changed=detect.cpp");

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");

    let manifest_path = std::path::PathBuf::from(manifest_dir);

    let mut ncnn_path = std::env::var("NCNN_SOURCE_DIR").unwrap_or_default();
    if ncnn_path.is_empty() {
        ncnn_path = manifest_path
            .join("third_party")
            .join("ncnn")
            .to_string_lossy()
            .to_string();
    }

    let mut zxing_path = std::env::var("ZXING_SOURCE_DIR").unwrap_or_default();
    if zxing_path.is_empty() {
        zxing_path = manifest_path
            .join("third_party")
            .join("zxing-cpp")
            .to_string_lossy()
            .to_string();
    }

    build(&ncnn_path, &zxing_path);

    println!("cargo:rustc-link-lib=static=ncnn");
    println!("cargo:rustc-link-lib=static=ZXing");
    println!("cargo:rustc-link-lib=static=detect");
    println!("cargo:rustc-link-lib=c++");

    println!("cargo:rerun-if-changed=detect.cpp");
    println!("cargo:rerun-if-changed=CMakeLists.txt");
}

// export ANDROID_NDK_ROOT=$ANDROID_HOME/ndk/21.4.7075529
// cmake -DNCNN_SOURCE_DIR=/tmp/ncnn -DCMAKE_INSTALL_PREFIX=install ..
// cmake -DCMAKE_INSTALL_PREFIX=install -DANDROID_ABI=arm64-v8a -DANDROID_PLATFORM=android-21 -DCMAKE_TOOLCHAIN_FILE="$ANDROID_HOME/ndk/21.4.7075529/build/cmake/android.toolchain.cmake" -DANDROID_CPP_FEATURES="exceptions" ..

fn build(ncnn_path: &str, zxing_cpp_path: &str) {
    let out = &std::env::var("OUT_DIR").unwrap_or_default();

    build_dependency(
        &out,
        ncnn_path,
        &[
            ("NCNN_OPENMP", "OFF"),
            ("NCNN_VULKAN", "OFF"),
            ("NCNN_SIMPLEOCV", "ON"),
            ("NCNN_BUILD_TOOLS", "OFF"),
            ("NCNN_BUILD_EXAMPLES", "OFF"),
            ("NCNN_BUILD_BENCHMARK", "OFF"),
            ("CMAKE_OSX_DEPLOYMENT_TARGET", "11.0"),
        ],
        &[],
    );

    build_dependency(
        &out,
        zxing_cpp_path,
        &[
            ("ZXING_READERS", "ON"),
            ("ZXING_WRITERS", "OFF"),
            ("ZXING_EXAMPLES", "OFF"),
            ("BUILD_SHARED_LIBS", "OFF"),
        ],
        &[],
    );

    build_dependency(
        &out,
        ".",
        &vec![],
        &[("PKG_CONFIG_PATH", &format!("{}/lib/pkgconfig", out))],
    );
}

fn build_dependency(out: &str, path: &str, defines: &[(&str, &str)], envs: &[(&str, &str)]) {
    let mut config = cmake::Config::new(path);

    for kv in envs {
        config.env(kv.0, kv.1);
    }

    for kv in defines {
        config.define(kv.0, kv.1);
    }

    let target = std::env::var("CARGO_CFG_TARGET_OS").unwrap();

    if target == "android" {
        let target = std::env::var("TARGET").unwrap_or_default();
        let arch = match target.as_str() {
            "i686-linux-android" => "x86",
            "x86_64-linux-android" => "x86_64",
            "aarch64-linux-android" => "arm64-v8a",
            "armv7-linux-androideabi" => "armeabi-v7a",
            _ => "",
        };

        let root = std::env::var("NDK_HOME").unwrap_or_default();
        if root.is_empty() {
            panic!("NDK_HOME can not empty");
        }

        config
            .define("ANDROID_ABI", arch)
            .define("ANDROID_PLATFORM", "android-21")
            .define(
                "CMAKE_TOOLCHAIN_FILE",
                root + "/build/cmake/android.toolchain.cmake",
            );
    }

    config
        .define("CMAKE_INSTALL_PREFIX", out)
        .profile("Release")
        .build();

    println!("cargo:rustc-link-search={}/lib", out);
}
