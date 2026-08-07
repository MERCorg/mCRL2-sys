/// Raw bindings to the PBES library of the mCRL2 toolset.
///
/// The accessors returning `*const _aterm` yield unprotected addresses of terms
/// owned by the object they were obtained from, and are non-null as long as
/// that object is alive. They follow the contract described in
/// [`crate::atermpp`]: the term must be protected before it can outlive the
/// current shared section or its owner.
#[cxx::bridge(namespace = "mcrl2::pbes_system")]
pub mod ffi {
    /// A helper struct for std::pair<const local_control_flow_graph_vertex*, UniquePtr<CxxVector<usize>>>
    #[derive(Debug)]
    struct vertex_outgoing_edge {
        vertex: *const local_control_flow_graph_vertex,
        edges: UniquePtr<CxxVector<usize>>,
    }

    /// A helper struct for a single substitution entry, where `lhs` is a
    /// data::variable and `rhs` the data::data_expression replacing it.
    ///
    /// NOTE: An identical struct is declared in the `data` bridge (see
    /// `src/data.rs`). A shared type cannot be used here since `cxx` generates
    /// the definition per bridge module, so both declarations must be kept in
    /// sync.
    struct assignment_pair {
        pub lhs: *const _aterm,
        pub rhs: *const _aterm,
    }

    unsafe extern "C++" {
        include!("mcrl2-sys/cpp/pbes.h");
        include!("mcrl2-sys/cpp/exception.h");

        type pbes_rewrite_context;

        fn mcrl2_pbes_create_rewrite_context(
            dataspec: &data_specification,
        ) -> UniquePtr<pbes_rewrite_context>;

        fn mcrl2_pbes_rewrite_set_assignments(
            ctx: Pin<&mut pbes_rewrite_context>,
            variables: &[*const _aterm],
            values: &[*const _aterm],
        );

        unsafe fn mcrl2_pbes_rewrite_formula(
            ctx: Pin<&mut pbes_rewrite_context>,
            formula: &_aterm,
        ) -> *const _aterm;

        type pbes;

        type srf_summand;

        /// Loads a PBES from a file.
        fn mcrl2_load_pbes_from_pbes_file(filename: &str) -> Result<UniquePtr<pbes>>;

        fn mcrl2_load_pbes_from_text_file(filename: &str) -> Result<UniquePtr<pbes>>;

        /// Loads a PBES from a string.
        fn mcrl2_load_pbes_from_text(input: &str) -> Result<UniquePtr<pbes>>;

        /// Normalizes a PBES.
        fn mcrl2_pbes_normalize(input: Pin<&mut pbes>);

        /// Checks whether the PBES is well-typed.
        fn mcrl2_pbes_is_well_typed(input: &pbes) -> bool;

        #[namespace = "mcrl2::data"]
        type data_specification = crate::data::ffi::data_specification;

        fn mcrl2_pbes_data_specification(input: &pbes) -> UniquePtr<data_specification>;

        type pbes_equation;

        /// Returns the equations of the given pbes, in declaration order.
        fn mcrl2_pbes_equations(result: Pin<&mut CxxVector<pbes_equation>>, input: &pbes);

        /// Returns true when the equation has a least fixed-point (mu) symbol.
        fn mcrl2_pbes_equation_is_mu(equation: &pbes_equation) -> bool;

        /// Returns the bound predicate variable (name and parameters) of the equation.
        fn mcrl2_pbes_equation_variable(equation: &pbes_equation) -> *const _aterm;

        /// Returns the right-hand side predicate formula of the equation.
        fn mcrl2_pbes_equation_formula(equation: &pbes_equation) -> *const _aterm;

        type stategraph_algorithm;

        /// Run the state graph algorithm and obtain the result.
        fn mcrl2_stategraph_local_algorithm_run(
            input: &pbes,
        ) -> Result<UniquePtr<stategraph_algorithm>>;

        #[namespace = "mcrl2::pbes_system::detail"]
        type local_control_flow_graph;

        /// Get the number of control flow graphs identified by the state graph algorithm.
        fn mcrl2_stategraph_local_algorithm_cfgs(input: &stategraph_algorithm) -> usize;

        /// Returns the control flow graph at the given index.
        fn mcrl2_stategraph_local_algorithm_cfg(
            input: &stategraph_algorithm,
            index: usize,
        ) -> &local_control_flow_graph;

        #[namespace = "mcrl2::pbes_system::detail"]
        type stategraph_equation;

        fn mcrl2_stategraph_local_algorithm_equations(input: &stategraph_algorithm) -> usize;

        fn mcrl2_stategraph_local_algorithm_equation(
            input: &stategraph_algorithm,
            index: usize,
        ) -> &stategraph_equation;

        #[namespace = "mcrl2::pbes_system::detail"]
        type predicate_variable;

        /// Returns the predicate variables of a stategraph equation.
        fn mcrl2_stategraph_equation_predicate_variables(
            input: &stategraph_equation,
        ) -> UniquePtr<CxxVector<predicate_variable>>;

        fn mcrl2_pbes_is_propositional_variable(expression: &_aterm) -> bool;

        /// Pretty-prints a propositional variable using the mCRL2 pretty printer.
        fn mcrl2_propositional_variable_to_string(variable: &_aterm) -> String;

        /// Returns the propositional variable of a pbes equation
        fn mcrl2_stategraph_equation_variable(equation: &stategraph_equation) -> *const _aterm;

        /// Returns the used set of a predicate variable.
        fn mcrl2_predicate_variable_used(input: &predicate_variable) -> Vec<usize>;

        /// Returns the changed set of a predicate variable.
        fn mcrl2_predicate_variable_changed(input: &predicate_variable) -> Vec<usize>;

        /// Returns the soucre variable set of a predicate variable.
        fn mcrl2_predicate_variable_source(input: &predicate_variable) -> Vec<usize>;

        /// Returns the target variable set of a predicate variable.
        fn mcrl2_predicate_variable_target(input: &predicate_variable) -> Vec<usize>;

        /// Returns the copy variable set of a predicate variable.
        fn mcrl2_predicate_variable_copy(input: &predicate_variable) -> Vec<usize>;

        /// Returns the propositional variable instantiation of a predicate variable.
        fn mcrl2_predicate_variable_propositional_variable_instantiation(
            input: &predicate_variable,
        ) -> *const _aterm;

        #[namespace = "mcrl2::pbes_system::detail"]
        type local_control_flow_graph_vertex;

        /// Obtain the vertices of a cfg.
        fn mcrl2_local_control_flow_graph_vertices(input: &local_control_flow_graph) -> usize;

        fn mcrl2_local_control_flow_graph_vertex(
            input: &local_control_flow_graph,
            index: usize,
        ) -> &local_control_flow_graph_vertex;

        #[namespace = "atermpp::detail"]
        type _aterm = crate::atermpp::ffi::_aterm;

        /// Obtain the index of the variable associated with the vertex.
        fn mcrl2_local_control_flow_graph_vertex_index(
            vertex: &local_control_flow_graph_vertex,
        ) -> usize;

        /// Obtain the name of the variable associated with the vertex.
        fn mcrl2_local_control_flow_graph_vertex_name(
            vertex: &local_control_flow_graph_vertex,
        ) -> *const _aterm;

        /// Obtain the value of the variable associated with the vertex.
        fn mcrl2_local_control_flow_graph_vertex_value(
            vertex: &local_control_flow_graph_vertex,
        ) -> *const _aterm;

        /// Obtain the outgoing edges of the vertex.
        fn mcrl2_local_control_flow_graph_vertex_outgoing_edges(
            input: &local_control_flow_graph_vertex,
        ) -> UniquePtr<CxxVector<vertex_outgoing_edge>>;

        type srf_pbes;

        type srf_equation;

        /// Convert a PBES to an SRF PBES.
        fn mcrl2_pbes_to_srf_pbes(input: &pbes) -> Result<UniquePtr<srf_pbes>>;

        /// Returns PBES as a string.
        fn mcrl2_pbes_to_string(input: &pbes) -> String;

        /// Convert a SRF PBES to a PBES.
        fn mcrl2_srf_pbes_to_pbes(input: &srf_pbes) -> UniquePtr<pbes>;

        /// Unify all parameters of the equations, optionally ignoring the equations
        /// related to counter example information. Finally, if reset is true, reset the
        /// newly introduced parameters to a default value.
        fn mcrl2_srf_pbes_unify_parameters(
            input: Pin<&mut srf_pbes>,
            ignore_ce_equations: bool,
            reset: bool,
        );

        /// Directly unify all parameters of a pbes without converting to SRF first.
        fn mcrl2_pbes_unify_parameters(
            input: Pin<&mut pbes>,
            ignore_ce_equations: bool,
            reset: bool,
        );

        /// Returns the summands of the given srf_equation.
        fn mcrl2_srf_equations_summands(
            result: Pin<&mut CxxVector<srf_summand>>,
            input: &srf_equation,
        );

        #[namespace = "atermpp"]
        type aterm = crate::atermpp::ffi::aterm;

        /// Returns the equations of the given srf_pbes.
        fn mcrl2_srf_pbes_equations(result: Pin<&mut CxxVector<srf_equation>>, input: &srf_pbes);

        fn mcrl2_srf_summand_condition(summand: &srf_summand) -> *const _aterm;

        fn mcrl2_srf_summand_variable(summand: &srf_summand) -> *const _aterm;

        fn mcrl2_srf_summand_parameters(summand: &srf_summand) -> *const _aterm;

        fn mcrl2_srf_equation_is_conjunctive(equation: &srf_equation) -> bool;

        fn mcrl2_srf_equation_is_mu(equation: &srf_equation) -> bool;

        fn mcrl2_pbes_initial_state(input: &pbes) -> *const _aterm;

        fn mcrl2_srf_pbes_initial_state(input: &srf_pbes) -> *const _aterm;

        unsafe fn mcrl2_srf_pbes_equation_variable(equation: &srf_equation) -> *const _aterm;

        /// Build a data::assignment_list from two parallel term-lists (data::variable_list and data::data_expression_list).
        fn mcrl2_make_data_assignment_list(variables: &_aterm, values: &_aterm)
        -> UniquePtr<aterm>;

        /// Replace data variables in a pbes expression according to the given substitutions.
        fn mcrl2_pbes_expression_replace_variables(
            expression: &_aterm,
            substitutions: &Vec<assignment_pair>,
        ) -> UniquePtr<aterm>;

        /// Replace propositional variables in a pbes expression according to the given substitutions.
        fn mcrl2_pbes_expression_replace_propositional_variables(
            expression: &_aterm,
            pi: &Vec<usize>,
        ) -> UniquePtr<aterm>;

        fn mcrl2_pbes_expression_to_string(expression: &_aterm) -> String;

        fn mcrl2_pbes_is_pbes_expression(expression: &_aterm) -> bool;
        fn mcrl2_pbes_is_propositional_variable_instantiation(expression: &_aterm) -> bool;
        fn mcrl2_pbes_is_not(term: &_aterm) -> bool;
        fn mcrl2_pbes_is_and(term: &_aterm) -> bool;
        fn mcrl2_pbes_is_or(term: &_aterm) -> bool;
        fn mcrl2_pbes_is_imp(term: &_aterm) -> bool;
        fn mcrl2_pbes_is_forall(term: &_aterm) -> bool;
        fn mcrl2_pbes_is_exists(term: &_aterm) -> bool;
    }
}
