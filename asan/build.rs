fn main() {
    println!("cargo:rerun-if-changed=cc/include/hooks.h");
    println!("cargo:rerun-if-changed=cc/include/trace.h");
    println!("cargo:rerun-if-changed=cc/include/printf.h");
    println!("cargo:rerun-if-changed=cc/src/asprintf.c");
    println!("cargo:rerun-if-changed=cc/src/log.c");
    println!("cargo:rerun-if-changed=cc/src/printf.c");

    cc::Build::new()
        .define("_GNU_SOURCE", None)
        .flag("-Werror")
        .include("cc/include/")
        .file("cc/src/printf.c")
        .compile("printf");

    cc::Build::new()
        .define("_GNU_SOURCE", None)
        .flag("-Werror")
        .include("cc/include/")
        .file("cc/src/asprintf.c")
        .compile("asprintf");

    cc::Build::new()
        .define("_GNU_SOURCE", None)
        .flag("-Werror")
        .include("cc/include/")
        .file("cc/src/vasprintf.c")
        .compile("vasprintf");
}
