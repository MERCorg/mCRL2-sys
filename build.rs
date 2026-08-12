use cargo_emit::rerun_if_changed;
use cc::Build;
use std::path::Path;
use std::path::PathBuf;

fn main() {
    // Path to the mCRL2 location
    let mcrl2_path = PathBuf::from("3rd-party/mCRL2");
    let mcrl2_workarounds_path = PathBuf::from("3rd-party/mCRL2-workarounds");

    #[cfg(feature = "cpptrace")]
    {
        // The debug flags must be set on all the standard libraries used.
        let mut debug_build = Build::new();
        add_debug_defines(&mut debug_build);
        add_compile_flags(&mut debug_build, &mcrl2_path);

        // Use the `cmake` crate to build cpptrace.
        let mut dst = cmake::Config::new("3rd-party/cpptrace")
            .define("BUILD_SHARED_LIBS", "OFF") // Build a static library.
            .define("CPPTRACE_USE_EXTERNAL_LIBDWARF", "OFF") // Compile libdwarf as part of cpptrace.
            .init_cxx_cfg(debug_build)
            .build();
        dst.push("lib");

        cargo_emit::rustc_link_search!(dst.display() => "native");
        // Link the required libraries for cpptrace (Can this be derived from the cmake somehow?)
        cargo_emit::rustc_link_lib!("cpptrace" => "static");

        // cpptrace resolves symbols through libdwarf on every Unix target, so on
        // macOS just as much as on Linux, and on MinGW as well; only MSVC uses
        // dbghelp instead. cpptrace builds libdwarf itself and installs it next
        // to libcpptrace.a, and libdwarf in turn uses zstd and zlib. Those are
        // either built by cpptrace as well (into `dst`) or taken from the system.
        if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() != Ok("msvc") {
            cargo_emit::rustc_link_lib!("dwarf" => "static");
            link_compression_library(&dst, "zstd");
            link_compression_library(&dst, "z");
        }
    }

    // The mCRL2 source files that we need to build for our Rust wrapper.
    let atermpp_source_files = [
        "aterm_implementation.cpp",
        "aterm_io_binary.cpp",
        "aterm_io_text.cpp",
        "function_symbol.cpp",
        "function_symbol_pool.cpp",
        "gc_stress_thread.cpp",
    ];

    let core_source_files = ["dparser.cpp", "core.cpp"];

    let data_source_files = [
        "data.cpp",
        "data_io.cpp",
        "data_specification.cpp",
        "machine_word.cpp",
        "typecheck.cpp",
        "detail/prover/smt_lib_solver.cpp",
        "detail/rewrite/jitty.cpp",
        "detail/rewrite/rewrite.cpp",
        "detail/rewrite/strategy.cpp",
    ];

    let dparser_source_files = [
        "arg.c",
        "parse.c",
        "scan.c",
        "dsymtab.c",
        "util.c",
        "read_binary.c",
        "dparse_tree.c",
    ];

    let lps_source_files = [
        "lps.cpp",
        "lps_io.cpp",
        //"linearise.cpp",
        //"lpsparunfoldlib.cpp",
        //"next_state_generator.cpp",
        //"symbolic_lts_io.cpp",
    ];

    let utilities_source_files = [
        "bitstream.cpp",
        "cache_metric.cpp",
        "logger.cpp",
        //"command_line_interface.cpp",
        "text_utility.cpp",
        "toolset_version.cpp",
    ];

    let pbes_sources_files = [
        "algorithms.cpp",
        "io.cpp",
        "pbes.cpp",
        "pbes_explorer.cpp",
        "pgsolver.cpp",
    ];

    let process_source_files = ["process.cpp"];

    // Build dparser separately since it's a C library.
    let mut build_dparser = cc::Build::new();
    build_dparser
        .include(mcrl2_path.join("3rd-party/dparser"))
        .files(add_prefix(
            mcrl2_path.join("3rd-party/dparser"),
            &dparser_source_files,
        ));

    add_compile_flags(&mut build_dparser, &mcrl2_path);
    build_dparser.compile("dparser");

    // These are the files for which we need to call cxxbuild to produce the bridge code.
    let mut build = cxx_build::bridges([
        "src/atermpp.rs",
        "src/data.rs",
        "src/pbes.rs",
        "src/lps.rs",
        "src/log.rs",
    ]);

    // Additional files needed to compile the bridge, basically to build mCRL2 itself.
    build
        .cpp(true)
        .std("c++20")
        .define("MCRL2_NO_RECURSIVE_SOUNDNESS_CHECKS", "1") // These checks overflow the stack, and are extremely slow.
        .define("LPS_NO_RECURSIVE_SOUNDNESS_CHECKS", "1")
        .define("MERC_MCRL2_VERSION", "\"internal_merc_build\"") // Sets the mCRL2 version to something recognized as our internal build.
        .includes(add_prefix(
            &mcrl2_path,
            &[
                "3rd-party/dparser/",
                "libraries/atermpp/include",
                "libraries/core/include",
                "libraries/data/include",
                // "libraries/gui/include",
                "libraries/lps/include",
                // "libraries/lts/include",
                // "libraries/modal_formula/include",
                "libraries/pbes/include",
                // "libraries/pg/include",
                // "libraries/pres/include",
                "libraries/process/include",
                // "libraries/smt/include",
                "libraries/symbolic/include",
                "libraries/utilities/include",
            ],
        ))
        .include(mcrl2_workarounds_path.join("include"))
        .include("3rd-party/boost-include-only/")
        .include("dparser")
        .include(PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("include")) // This is where cmake generates the headers for cpptrace.
        .files(add_prefix(
            mcrl2_path.join("libraries/atermpp/source"),
            &atermpp_source_files,
        ))
        .files(add_prefix(
            mcrl2_path.join("libraries/core/source"),
            &core_source_files,
        ))
        .files(add_prefix(
            mcrl2_path.join("libraries/data/source"),
            &data_source_files,
        ))
        .files(add_prefix(
            mcrl2_path.join("libraries/lps/source"),
            &lps_source_files,
        ))
        .files(add_prefix(
            mcrl2_path.join("libraries/pbes/source"),
            &pbes_sources_files,
        ))
        .files(add_prefix(
            mcrl2_path.join("libraries/process/source"),
            &process_source_files,
        ))
        .files(add_prefix(
            mcrl2_path.join("libraries/utilities/source"),
            &utilities_source_files,
        ))
        .file("cpp/pbes.cpp")
        .file("cpp/data.cpp")
        .file("cpp/lps.cpp")
        .file(mcrl2_workarounds_path.join("mcrl2_syntax.c")); // This is to avoid generating the dparser grammer.

    #[cfg(feature = "jittyc")]
    build.files(add_prefix(
        mcrl2_path.join("libraries/data/source"),
        &["detail/rewrite/jittyc.cpp"],
    ));

    #[cfg(feature = "jittyc")]
    build.define("MCRL2_ENABLE_JITTYC", "1");

    #[cfg(feature = "cpptrace")]
    build.define("MCRL2_ENABLE_CPPTRACE", "1");

    // Enable thread safety since Rust executes its tests at least by default, and allow threading in general.
    build.define("MCRL2_ENABLE_MULTITHREADING", "1");

    // Enable machine numbers.
    build.define("MCRL2_ENABLE_MACHINENUMBERS", "1");

    add_compile_flags(&mut build, &mcrl2_path);
    add_debug_defines(&mut build);

    build.compile("mcrl2-sys");

    // These files should trigger a rebuild.
    rerun_if_changed!("build.rs");
    rerun_if_changed!("cpp/assert.h");
    rerun_if_changed!("cpp/atermpp.h");
    rerun_if_changed!("cpp/data.cpp");
    rerun_if_changed!("cpp/data.h");
    rerun_if_changed!("cpp/exception.h");
    rerun_if_changed!("cpp/log.h");
    rerun_if_changed!("cpp/lps.h");
    rerun_if_changed!("cpp/lps.cpp");
    rerun_if_changed!("cpp/pbes.cpp");
    rerun_if_changed!("cpp/pbes.h");
    rerun_if_changed!(mcrl2_workarounds_path.join("mcrl2_syntax.c").display());
}

/// Links the compression library `name` that libdwarf (built as part of
/// cpptrace) needs, preferring a static archive over a shared library.
///
/// `cpptrace_lib_dir` is the `lib` directory cpptrace installed into; it is
/// already on the link search path.
#[cfg(feature = "cpptrace")]
fn link_compression_library(cpptrace_lib_dir: &Path, name: &str) {
    // When the system does not provide the library, cpptrace fetches and builds
    // its own copy and installs it next to libcpptrace.a. Look there first,
    // since that is the copy libdwarf was actually linked against.
    if cpptrace_lib_dir.join(format!("lib{name}.a")).exists() {
        cargo_emit::rustc_link_lib!(name => "static");
        return;
    }

    // Path::exists() inspects the host filesystem, which says nothing about the
    // target when cross-compiling: the archive found there has the host's ABI.
    // Name the library without a search path instead, so that the link fails
    // loudly unless the target's library directory was supplied through
    // RUSTFLAGS, rather than feeding host objects to the cross linker.
    let host = std::env::var("HOST").expect("cargo should always set this variable");
    let target = std::env::var("TARGET").expect("cargo should always set this variable");
    if host != target {
        cargo_emit::warning!(
            "Cross-compiling from {} to {}: cannot locate lib{} for the target, so it is linked \
             from the search path of the target toolchain.",
            host,
            target,
            name
        );
        cargo_emit::rustc_link_lib!(name => "static");
        return;
    }

    // The directories searched below are where a Linux distribution keeps its
    // archives. On other targets the library belongs to the platform SDK
    // instead, where it exists as a shared library only (macOS, for instance,
    // ships zlib but no static libz), and the linker driver already searches
    // those locations. Naming the library without a search path leaves that
    // lookup to the driver.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("linux") {
        cargo_emit::rustc_link_lib!(name => "dylib");
        return;
    }

    let arch =
        std::env::var("CARGO_CFG_TARGET_ARCH").expect("cargo should always set this variable");
    let directories = [
        PathBuf::from(format!("/usr/lib/{arch}-linux-gnu")),
        PathBuf::from("/usr/lib64"),
        PathBuf::from("/usr/lib"),
    ];

    for directory in directories {
        if directory.join(format!("lib{name}.a")).exists() {
            // rustc resolves static libraries itself and does not know the
            // distribution's library directories, so the directory that holds
            // the archive has to be named. Only the matching directory is added,
            // since a `rustc-link-search` applies to the whole crate graph.
            cargo_emit::rustc_link_search!(directory.display() => "native");
            cargo_emit::rustc_link_lib!(name => "static");
            return;
        }

        if directory.join(format!("lib{name}.so")).exists() {
            // Shared libraries are resolved by the linker driver, which already
            // searches these directories, so no search path is needed here.
            cargo_emit::warning!(
                "Only found a shared lib{0} in {1}, so the resulting binaries depend on lib{0}.so \
                 at runtime. Install the static lib{0} of this distribution to avoid that.",
                name,
                directory.display()
            );
            cargo_emit::rustc_link_lib!(name => "dylib");
            return;
        }
    }

    cargo_emit::warning!(
        "Could not find lib{} to link, assuming that it is not required.",
        name
    );
}

// Enable various additional debug defines based on the current profile.
fn add_debug_defines(build: &mut Build) {
    // Disable assertions and other checks in release mode. Cargo only ever sets
    // PROFILE to "debug" or "release", so the panic below is a guard against
    // future cargo changes rather than a reachable case.
    let profile = std::env::var("PROFILE").expect("cargo should always set this variable");
    match profile.as_str() {
        "debug" => {
            // Debug mode for libc++ (the LLVM standard library)
            build.define("_LIBCPP_DEBUG", "1");
            build.define("_LIBCPP_ENABLE_THREAD_SAFETY_ANNOTATIONS", "1");
            build.define("_LIBCPP_HARDENING_MODE", "_LIBCPP_HARDENING_MODE_DEBUG");
            // build.define("_LIBCPP_ABI_BOUNDED_ITERATORS", "1");
            // build.define("_LIBCPP_ABI_BOUNDED_ITERATORS_IN_STRING", "1");
            // build.define("_LIBCPP_ABI_BOUNDED_ITERATORS_IN_VECTOR", "1");
            // build.define("_LIBCPP_ABI_BOUNDED_UNIQUE_PTR", "1");
            // build.define("_LIBCPP_ABI_BOUNDED_ITERATORS_IN_STD_ARRAY", "1");

            // // Debug mode for libstdc++ (the GNU standard library)
            // build.define("_GLIBCXX_DEBUG", "1");
            // build.define("_GLIBCXX_DEBUG_PEDANTIC", "1");
            build.define("_GLIBCXX_ASSERTIONS", "1");

            // Handle overflows
            build.flag_if_supported("-ftrapv");
            build.flag_if_supported("-fstack-protector-strong");
            build.flag_if_supported("-fstack-clash-protection");
            build.flag_if_supported("-fstrict-flex-arrays=3");
        }
        "release" => {
            build.define("NDEBUG", "1");
        }
        _ => {
            panic!("Unsupported profile {}", profile);
        }
    }
}

/// Add platform specific compile flags and definitions.
fn add_compile_flags(build: &mut Build, mcrl2_path: &Path) {
    // Which flags the compiler accepts is a property of the target, not of the
    // host this build script happens to run on, so `#[cfg(windows)]` and
    // `#[cfg(unix)]` are the wrong question here: they describe the host and
    // would hand MSVC flags to gcc (and vice versa) when cross-compiling.
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

    if target_os == "windows" {
        build
            .define("WIN32", "1")
            .define("WIN32_LEAN_AND_MEAN", "1")
            .define("NOMINMAX", "1")
            .define("_USE_MATH_DEFINES", "1")
            .define("_CRT_SECURE_CPP_OVERLOAD_STANDARD_NAMES", "1")
            .define("_CRT_SECURE_NO_WARNINGS", "1")
            .define("BOOST_ALL_NO_LIB", "1");
    }

    if target_env == "msvc" {
        build
            .include(mcrl2_path.join("cmake/workarounds/msvc")) // These are the MSVC workarounds (dirent.h, unistd.h and sys/) that mCRL2 relies on for compilation.
            .flag_if_supported("/EHsc")
            .flag_if_supported("/bigobj")
            .flag_if_supported("/MP")
            .flag_if_supported("/Zc:inline")
            .flag_if_supported("/permissive-")
            .flag_if_supported("/wd4267"); // Disable implicit conversion warnings.
    } else {
        // gcc/clang style drivers, which includes the MinGW targets.
        build
            .flag_if_supported("-Wno-unused-parameter") // I don't care about unused parameters in mCRL2 code.
            .flag_if_supported("-pipe")
            .flag_if_supported("-pedantic");
    }
}

/// \returns A vector of paths where prefix is prepended to every path in paths.
fn add_prefix<P: AsRef<Path>>(prefix: P, paths: &[&str]) -> Vec<PathBuf> {
    paths
        .iter()
        .map(|path| prefix.as_ref().join(path))
        .collect()
}
