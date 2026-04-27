#[cxx::bridge(namespace = "mcrl2::lps")]
pub mod ffi {
    unsafe extern "C++" {
        include!("mcrl2-sys/cpp/lps.h");
        include!("mcrl2-sys/cpp/exception.h");

        type stochastic_specification;

        type stochastic_action_summand;

        /// Loads an LPS from a file.
        fn mcrl2_load_lps_from_lps_file(filename: &str) -> Result<UniquePtr<stochastic_specification>>;

        /// Preprocess the LPS in a way that is suitable for symbolic exploration.
        fn mcrl2_lps_preprocess_symbolic_exploration(lps: &stochastic_specification) -> Result<UniquePtr<stochastic_specification>>;      
    }
}