#[cxx::bridge(namespace = "mcrl2::data")]
pub mod ffi {

    /// A helper struct for a single substitution entry, where `lhs` is a
    /// data::variable and `rhs` the data::data_expression replacing it.
    ///
    /// NOTE: An identical struct is declared in the `pbes` bridge (see
    /// `src/pbes.rs`). A shared type cannot be used here since `cxx` generates
    /// the definition per bridge module, so both declarations must be kept in
    /// sync.
    struct assignment_pair {
        pub lhs: *const _aterm,
        pub rhs: *const _aterm,
    }

    unsafe extern "C++" {
        include!("mcrl2-sys/cpp/data.h");
        include!("mcrl2-sys/cpp/exception.h");

        type data_specification;

        /// Creates a data specification from the given string.
        fn mcrl2_data_specification_from_string(input: &str) -> UniquePtr<data_specification>;

        /// Returns the user-declared sorts as an aterm list of basic sorts.
        fn mcrl2_data_specification_user_defined_sorts(specification: &data_specification) -> UniquePtr<aterm>;

        /// Returns the user-declared sort aliases as an aterm list of aliases.
        fn mcrl2_data_specification_user_defined_aliases(specification: &data_specification) -> UniquePtr<aterm>;

        /// Returns the user-declared constructors as an aterm list of function symbols.
        fn mcrl2_data_specification_user_defined_constructors(
            specification: &data_specification,
        ) -> UniquePtr<aterm>;

        /// Returns the user-declared mappings as an aterm list of function symbols.
        fn mcrl2_data_specification_user_defined_mappings(specification: &data_specification) -> UniquePtr<aterm>;

        /// Returns the user-declared equations as an aterm list of data equations.
        fn mcrl2_data_specification_user_defined_equations(specification: &data_specification) -> UniquePtr<aterm>;

        #[namespace = "mcrl2::data::detail"]
        type RewriterJitty;

        #[cfg(feature = "jittyc")]
        #[namespace = "mcrl2::data::detail"]
        type RewriterCompilingJitty;

        /// Creates a jitty rewriter from the given data specification.
        fn mcrl2_create_rewriter_jitty(data_spec: &data_specification) -> UniquePtr<RewriterJitty>;

        /// Creates a compiling rewriter from the given data specification.
        #[cfg(feature = "jittyc")]
        fn mcrl2_create_rewriter_jittyc(
            data_spec: &data_specification,
        ) -> UniquePtr<RewriterCompilingJitty>;

        #[namespace = "atermpp::detail"]
        type _aterm = crate::atermpp::ffi::_aterm;

        #[namespace = "atermpp"]
        type aterm = crate::atermpp::ffi::aterm;

        /// Replace variables in the given data expression according to the given substitution sigma.
        fn mcrl2_data_expression_replace_variables(
            input: &_aterm,
            sigma: &Vec<assignment_pair>,
        ) -> UniquePtr<aterm>;

        // Recognizers for the various variants of data expressions.
        fn mcrl2_data_expression_is_variable(input: &_aterm) -> bool;
        fn mcrl2_data_expression_is_application(input: &_aterm) -> bool;
        fn mcrl2_data_expression_is_abstraction(input: &_aterm) -> bool;
        fn mcrl2_data_expression_is_binder_exists(input: &_aterm) -> bool;
        fn mcrl2_data_expression_is_binder_forall(input: &_aterm) -> bool;
        fn mcrl2_data_expression_is_binder_lambda(input: &_aterm) -> bool;
        fn mcrl2_data_expression_is_binder_set_comp(input: &_aterm) -> bool;
        fn mcrl2_data_expression_is_binder_bag_comp(input: &_aterm) -> bool;
        fn mcrl2_data_expression_is_binder_untyped_set_bag_comp(input: &_aterm) -> bool;
        fn mcrl2_data_expression_is_function_symbol(input: &_aterm) -> bool;
        fn mcrl2_data_expression_is_where_clause(input: &_aterm) -> bool;
        fn mcrl2_data_expression_is_machine_number(input: &_aterm) -> bool;
        fn mcrl2_data_expression_is_untyped_identifier(input: &_aterm) -> bool;
        fn mcrl2_data_expression_is_data_expression(input: &_aterm) -> bool;

        fn mcrl2_is_data_sort_expression(input: &_aterm) -> bool;

        fn mcrl2_data_expression_to_string(input: &_aterm) -> String;
        fn mcrl2_sort_expression_to_string(input: &_aterm) -> String;

        /// Returns the user-defined sorts of the data specification as an aterm_list.
        fn mcrl2_data_specification_user_defined_sorts(spec: &data_specification) -> UniquePtr<aterm>;
        /// Returns the user-defined aliases of the data specification as an aterm_list.
        fn mcrl2_data_specification_user_defined_aliases(spec: &data_specification) -> UniquePtr<aterm>;
        /// Returns the user-defined constructors of the data specification as an aterm_list.
        fn mcrl2_data_specification_user_defined_constructors(spec: &data_specification) -> UniquePtr<aterm>;
        /// Returns the user-defined mappings of the data specification as an aterm_list.
        fn mcrl2_data_specification_user_defined_mappings(spec: &data_specification) -> UniquePtr<aterm>;
        /// Returns the user-defined equations of the data specification as an aterm_list.
        fn mcrl2_data_specification_user_defined_equations(spec: &data_specification) -> UniquePtr<aterm>;
    }
}
