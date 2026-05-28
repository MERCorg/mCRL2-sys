#[cxx::bridge(namespace = "mcrl2::lps")]
pub mod ffi {
    unsafe extern "C++" {
        include!("mcrl2-sys/cpp/lps.h");
        include!("mcrl2-sys/cpp/exception.h");

        type stochastic_specification;

        type stochastic_action_summand;

        type stochastic_process_initializer;

        type learn_successors_context;

        #[namespace = "atermpp::detail"]
        type _aterm = crate::atermpp::ffi::_aterm;

        /// Loads an LPS from a file.
        fn mcrl2_lps_load_from_lps_file(filename: &str) -> Result<UniquePtr<stochastic_specification>>;

        /// Preprocess the LPS in a way that is suitable for symbolic exploration.
        fn mcrl2_lps_preprocess_symbolic_exploration(lps: &stochastic_specification) -> Result<UniquePtr<stochastic_specification>>;     

        fn mcrl2_lps_num_of_action_summands(lps: &stochastic_specification) -> usize;

        fn mcrl2_lps_action_summand(lps: &stochastic_specification, index: usize) -> Result<UniquePtr<stochastic_action_summand>>; 

        fn mcrl2_lps_process_initializer(lps: &stochastic_specification) -> Result<UniquePtr<stochastic_process_initializer>>;

        /// Pretty-prints a multi-action term using the mCRL2 pretty printer.
        fn mcrl2_lps_multi_action_to_string(input: &_aterm) -> String;

        fn mcrl2_lps_action_summand_condition(summand: &stochastic_action_summand) -> *const _aterm;

        fn mcrl2_lps_action_summand_multi_action(summand: &stochastic_action_summand) -> *const _aterm;

        fn mcrl2_lps_action_summand_summation_variables(summand: &stochastic_action_summand) -> *const _aterm;

        fn mcrl2_lps_action_summand_assignments(summand: &stochastic_action_summand) -> *const _aterm;

        fn mcrl2_lps_process_parameters(lps: &stochastic_specification) -> *const _aterm;

        fn mcrl2_lps_process_initializer_expressions(init: &stochastic_process_initializer) -> *const _aterm;

        /// Creates a learn_successors_context containing a rewriter, substitution, and enumerator.
        fn mcrl2_lps_create_learn_successors_context(spec: &stochastic_specification) -> UniquePtr<learn_successors_context>;

        /// Enumerate all solutions for the summand's condition under the given read parameter assignments.
        /// Calls the callback with the context pointer, a slice of next-state values, and a pointer to
        /// the rewritten multi-action term for each solution.
        unsafe fn mcrl2_lps_enumerate(
            context: Pin<&mut learn_successors_context>,
            condition: &_aterm,
            summation_variables: &_aterm,
            assignments: &_aterm,
            multi_action: &_aterm,
            read_parameters: &[*const _aterm],
            read_values: &[*const _aterm],
            callback_context: *mut u8,
            callback: unsafe fn(*mut u8, &[*const _aterm], *const _aterm),
        );
    }
}