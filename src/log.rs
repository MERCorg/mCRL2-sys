#[cxx::bridge(namespace = "mcrl2::log")]
pub mod ffi {
    unsafe extern "C++" {
        include!("mcrl2-sys/cpp/log.h");
        include!("mcrl2-sys/cpp/exception.h");

        /// Sets the reporting level for mCRL2 utilities logging.
        ///
        /// `level` is `0` (quiet) up to the most verbose level. Values beyond
        /// the highest valid level are clamped on the C++ side, so an
        /// out-of-range integer can never form an invalid enumerator.
        fn mcrl2_set_reporting_level(level: usize);
    }
}
