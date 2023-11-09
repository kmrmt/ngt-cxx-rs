fn main() -> miette::Result<()> {
    let mut config = cmake::Config::new("NGT");
    let dst = config.cxxflag("-w").build();

    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=dylib=c++");

    let current_dir = std::env::current_dir().unwrap();
    println!("cargo:rustc-link-search=native={}", current_dir.display());

    cxx_build::bridge("src/main.rs")
        .file("src/helper.cpp")
        .flag_if_supported("-std=c++20")
        .compile("cxx-test");

    println!("cargo:rustc-link-lib=static=ngt");
    println!("cargo:rustc-link-lib=static=gomp");
    println!("cargo:rerun-if-changed=src/*");
    Ok(())
}
