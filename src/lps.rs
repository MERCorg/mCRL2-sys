#[cxx::bridge(namespace = "mcrl2::lps")]
pub mod ffi {
    unsafe extern "C++" {
        include!("mcrl2-sys/cpp/lps.h");
        include!("mcrl2-sys/cpp/exception.h");

        type stochastic_specification;

        type stochastic_action_summand;

        type stochastic_process_initializer;

        #[namespace = "atermpp::detail"]
        type _aterm = crate::atermpp::ffi::_aterm;

        /// Loads an LPS from a file.
        fn mcrl2_lps_load_from_lps_file(filename: &str) -> Result<UniquePtr<stochastic_specification>>;

        /// Preprocess the LPS in a way that is suitable for symbolic exploration.
        fn mcrl2_lps_preprocess_symbolic_exploration(lps: &stochastic_specification) -> Result<UniquePtr<stochastic_specification>>;     

        fn mcrl2_lps_num_of_action_summands(lps: &stochastic_specification) -> usize;

        fn mcrl2_lps_action_summand(lps: &stochastic_specification, index: usize) -> Result<UniquePtr<stochastic_action_summand>>; 

        fn mcrl2_lps_process_initializer(lps: &stochastic_specification) -> Result<UniquePtr<stochastic_process_initializer>>;

        fn mcrl2_lps_action_summand_condition(summand: &stochastic_action_summand) -> *const _aterm;
    }
}