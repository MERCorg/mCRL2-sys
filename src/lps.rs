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

        /// Loads an LPS from a binary file.
        fn mcrl2_lps_load_from_lps_file(filename: &str) -> Result<UniquePtr<stochastic_specification>>;

        /// Loads an LPS from a textual mCRL2 process specification file.
        fn mcrl2_lps_load_from_text_file(filename: &str) -> Result<UniquePtr<stochastic_specification>>;

        /// Preprocess the LPS in a way that is suitable for symbolic exploration.
        fn mcrl2_lps_preprocess_symbolic_exploration(lps: &stochastic_specification) -> Result<UniquePtr<stochastic_specification>>;     

        fn mcrl2_lps_num_of_action_summands(lps: &stochastic_specification) -> usize;

        fn mcrl2_lps_action_summand(lps: &stochastic_specification, index: usize) -> Result<UniquePtr<stochastic_action_summand>>; 

        fn mcrl2_lps_process_initializer(lps: &stochastic_specification) -> Result<UniquePtr<stochastic_process_initializer>>;

        /// Pretty-prints a multi-action term using the mCRL2 pretty printer.
        fn mcrl2_lps_multi_action_to_string(input: &_aterm) -> String;

        /// Returns the address of the tau (empty) multi-action term. The
        /// pointer is backed by a static and remains valid for the process
        /// lifetime.
        fn mcrl2_lps_tau_multi_action() -> *const _aterm;

        fn mcrl2_lps_action_summand_condition(summand: &stochastic_action_summand) -> *const _aterm;

        fn mcrl2_lps_action_summand_multi_action(summand: &stochastic_action_summand) -> *const _aterm;

        fn mcrl2_lps_action_summand_summation_variables(summand: &stochastic_action_summand) -> *const _aterm;

        fn mcrl2_lps_action_summand_assignments(summand: &stochastic_action_summand) -> *const _aterm;

        fn mcrl2_lps_process_parameters(lps: &stochastic_specification) -> *const _aterm;

        fn mcrl2_lps_process_initializer_expressions(init: &stochastic_process_initializer) -> *const _aterm;

        /// Creates a learn_successors_context containing a rewriter, substitution, and enumerator.
        fn mcrl2_lps_create_learn_successors_context(spec: &stochastic_specification) -> UniquePtr<learn_successors_context>;

        /// Creates a learn_successors_context from a data specification alone (used when no LPS is available).
        #[namespace = "mcrl2::data"]
        type data_specification = crate::data::ffi::data_specification;

        fn mcrl2_lps_create_learn_successors_context_from_data_spec(data_spec: &data_specification) -> UniquePtr<learn_successors_context>;

        /// Assign variables in the context substitution (sigma).
        fn mcrl2_lps_set_assignments(
            context: Pin<&mut learn_successors_context>,
            variables: &[*const _aterm],
            values: &[*const _aterm],
        );

        /// Enumerate all solutions for the summand's condition under the given read parameter assignments.
        /// Calls the callback with the context pointer, a slice of next-state values, and a pointer to
        /// the rewritten multi-action term for each solution.
        unsafe fn mcrl2_lps_enumerate(
            context: Pin<&mut learn_successors_context>,
            condition: &_aterm,
            summation_variables: &_aterm,
            assignments: &_aterm,
            multi_action: &_aterm,
            callback_context: *mut u8,
            callback: unsafe fn(*mut u8, &[*const _aterm], *const _aterm),
        );

        /// Like `mcrl2_lps_enumerate`, but additionally passes the
        /// summation-variable solution slice to the callback for each solution.
        /// The solution can be cached and replayed via `mcrl2_lps_compute_successor`.
        unsafe fn mcrl2_lps_enumerate_solutions(
            context: Pin<&mut learn_successors_context>,
            condition: &_aterm,
            summation_variables: &_aterm,
            assignments: &_aterm,
            multi_action: &_aterm,
            callback_context: *mut u8,
            callback: unsafe fn(*mut u8, &[*const _aterm], *const _aterm, &[*const _aterm]),
        );

        /// Recomputes the next-state values and multi-action for a cached
        /// summation-variable solution, without re-enumerating the condition.
        /// The current substitution is expected to already contain the
        /// read-parameter assignments of the source state.
        unsafe fn mcrl2_lps_compute_successor(
            context: Pin<&mut learn_successors_context>,
            summation_variables: &_aterm,
            summation_values: &[*const _aterm],
            assignments: &_aterm,
            multi_action: &_aterm,
            callback_context: *mut u8,
            callback: unsafe fn(*mut u8, &[*const _aterm], *const _aterm),
        );
    }
}