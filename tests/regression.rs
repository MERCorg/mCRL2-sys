//! Regression tests for behaviour of the raw FFI that is easy to lose again.
//!
//! Three kinds of behaviour are pinned here:
//!
//! * bounds checks that guard raw slice indexing on the C++ side. `rust::Slice`
//!   only `assert`s its bounds and `NDEBUG` is defined in release, so a missing
//!   check is an out-of-bounds read reachable from safe Rust.
//! * C++ exceptions reaching Rust as `Err`. cxx generates a `noexcept` shim for
//!   every function that is not declared `-> Result<...>`, so a throw crossing
//!   such a function terminates the process instead.
//! * the substitution bookkeeping of the PBES rewrite context, which has to
//!   forget the previous assignment before applying the next one.

use std::sync::Mutex;
use std::sync::MutexGuard;

use mcrl2_sys::atermpp::ffi::_aterm;
use mcrl2_sys::atermpp::ffi::mcrl2_aterm_get_address;
use mcrl2_sys::atermpp::ffi::mcrl2_aterm_get_argument;
use mcrl2_sys::atermpp::ffi::mcrl2_aterm_pool_size;
use mcrl2_sys::atermpp::ffi::mcrl2_aterm_print;
use mcrl2_sys::cxx::CxxVector;
use mcrl2_sys::cxx::Exception;
use mcrl2_sys::cxx::UniquePtr;
use mcrl2_sys::data::ffi::assignment_pair;
use mcrl2_sys::data::ffi::mcrl2_create_rewriter_jitty;
use mcrl2_sys::data::ffi::mcrl2_data_expression_replace_variables;
use mcrl2_sys::data::ffi::mcrl2_data_specification_from_string;
use mcrl2_sys::data::ffi::mcrl2_data_specification_user_defined_aliases;
use mcrl2_sys::data::ffi::mcrl2_data_specification_user_defined_constructors;
use mcrl2_sys::data::ffi::mcrl2_data_specification_user_defined_equations;
use mcrl2_sys::data::ffi::mcrl2_data_specification_user_defined_mappings;
use mcrl2_sys::data::ffi::mcrl2_data_specification_user_defined_sorts;
use mcrl2_sys::data::ffi::mcrl2_pbes_expression_replace_variables;
use mcrl2_sys::lps::ffi::mcrl2_lps_create_learn_successors_context_from_data_spec;
use mcrl2_sys::lps::ffi::mcrl2_lps_set_assignments;
use mcrl2_sys::pbes::ffi::mcrl2_load_pbes_from_text;
use mcrl2_sys::pbes::ffi::mcrl2_local_control_flow_graph_vertex;
use mcrl2_sys::pbes::ffi::mcrl2_local_control_flow_graph_vertices;
use mcrl2_sys::pbes::ffi::mcrl2_pbes_create_rewrite_context;
use mcrl2_sys::pbes::ffi::mcrl2_pbes_data_specification;
use mcrl2_sys::pbes::ffi::mcrl2_pbes_equation_formula;
use mcrl2_sys::pbes::ffi::mcrl2_pbes_equations;
use mcrl2_sys::pbes::ffi::mcrl2_pbes_expression_replace_propositional_variables;
use mcrl2_sys::pbes::ffi::mcrl2_pbes_initial_state;
use mcrl2_sys::pbes::ffi::mcrl2_pbes_rewrite_formula;
use mcrl2_sys::pbes::ffi::mcrl2_pbes_rewrite_set_assignments;
use mcrl2_sys::pbes::ffi::mcrl2_stategraph_local_algorithm_cfg;
use mcrl2_sys::pbes::ffi::mcrl2_stategraph_local_algorithm_cfgs;
use mcrl2_sys::pbes::ffi::mcrl2_stategraph_local_algorithm_equation;
use mcrl2_sys::pbes::ffi::mcrl2_stategraph_local_algorithm_equations;
use mcrl2_sys::pbes::ffi::mcrl2_stategraph_local_algorithm_run;
use mcrl2_sys::pbes::ffi::pbes;
use mcrl2_sys::pbes::ffi::pbes_equation;

/// The aterm pool is global mutable state and these tests share one process, so
/// they run one at a time.
static POOL: Mutex<()> = Mutex::new(());

fn lock_pool() -> MutexGuard<'static, ()> {
    // A panicking test must not make every later test fail with a poison error.
    POOL.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Prints a term, for use in assertion messages and structural comparisons.
///
/// # Safety
/// `term` must point at a live term; every caller below obtains it from an
/// object that it keeps alive for the duration of the test.
fn print(term: *const _aterm) -> String {
    assert!(!term.is_null(), "the FFI never returns a null term");
    mcrl2_aterm_print(unsafe { &*term })
}

/// Unwraps the error of a `Result` whose success type has no `Debug`, which is
/// the case for every opaque C++ type the bridge returns.
fn expect_error<T>(result: Result<T, Exception>, message: &str) -> Exception {
    match result {
        Ok(_) => panic!("expected an error: {message}"),
        Err(error) => error,
    }
}

/// A PBES together with a snapshot of its equations, which owns the terms the
/// tests point into.
struct TestPbes {
    pbes: UniquePtr<pbes>,
    equations: UniquePtr<CxxVector<pbes_equation>>,
}

impl TestPbes {
    fn parse(text: &str) -> TestPbes {
        let pbes = mcrl2_load_pbes_from_text(text).expect("the test PBES should parse");
        let mut equations = CxxVector::new();
        mcrl2_pbes_equations(equations.pin_mut(), &pbes);
        TestPbes { pbes, equations }
    }

    /// The right-hand side of the first equation.
    fn formula(&self) -> *const _aterm {
        mcrl2_pbes_equation_formula(self.equations.get(0).expect("the PBES has an equation"))
    }

    fn initial_state(&self) -> *const _aterm {
        mcrl2_pbes_initial_state(&self.pbes)
    }
}

#[test]
fn a_jitty_rewriter_can_be_built_from_a_parsed_data_specification() {
    let _guard = lock_pool();

    let spec = mcrl2_data_specification_from_string(
        "map f: Nat -> Nat;

         var x: Nat;
         eqn f(x) = x + 1;
        ",
    );

    assert!(
        !mcrl2_create_rewriter_jitty(&spec).is_null(),
        "the rewriter should be built"
    );
}

#[test]
fn pbes_rewrite_set_assignments_rejects_a_length_mismatch() {
    let _guard = lock_pool();

    // `val(b)` is the data expression `b`, so the formula doubles as the
    // variable to substitute, and the initial state supplies a value for it.
    let pbes = TestPbes::parse("pbes mu X(b: Bool) = val(b);\n\ninit X(true);");
    let b = pbes.formula();
    let parameters = mcrl2_aterm_get_argument(unsafe { &*pbes.initial_state() }, 1);
    let value = mcrl2_aterm_get_argument(unsafe { &*parameters }, 0);

    let spec = mcrl2_pbes_data_specification(&pbes.pbes);
    let mut context = mcrl2_pbes_create_rewrite_context(&spec).expect("the context should be built");

    // Without the length check the loop indexes `values` up to `variables.len()`
    // and reads past the end of the slice.
    let error = mcrl2_pbes_rewrite_set_assignments(context.pin_mut(), &[b, b], &[value])
        .expect_err("two variables and one value must be rejected");
    assert!(
        error.what().contains("as many values as variables"),
        "unexpected error: {error}"
    );

    // The mirrored case, where there are more values than variables.
    mcrl2_pbes_rewrite_set_assignments(context.pin_mut(), &[b], &[value, value])
        .expect_err("one variable and two values must be rejected");

    // Equal lengths are still accepted.
    mcrl2_pbes_rewrite_set_assignments(context.pin_mut(), &[b], &[value])
        .expect("equal lengths should be accepted");
}

#[test]
fn lps_set_assignments_rejects_a_length_mismatch() {
    let _guard = lock_pool();

    let pbes = TestPbes::parse("pbes mu X(b: Bool) = val(b);\n\ninit X(true);");
    let b = pbes.formula();
    let parameters = mcrl2_aterm_get_argument(unsafe { &*pbes.initial_state() }, 1);
    let value = mcrl2_aterm_get_argument(unsafe { &*parameters }, 0);

    let spec = mcrl2_pbes_data_specification(&pbes.pbes);
    let mut context = mcrl2_lps_create_learn_successors_context_from_data_spec(&spec);

    let error = mcrl2_lps_set_assignments(context.pin_mut(), &[b, b], &[value])
        .expect_err("two variables and one value must be rejected");
    assert!(
        error.what().contains("as many values as variables"),
        "unexpected error: {error}"
    );

    mcrl2_lps_set_assignments(context.pin_mut(), &[b], &[value])
        .expect("equal lengths should be accepted");
}

#[test]
fn pbes_rewrite_context_forgets_the_previous_assignment() {
    let _guard = lock_pool();

    let pbes = TestPbes::parse("pbes mu X(b: Bool) = val(b);\n\ninit X(true);");
    let b = pbes.formula();
    let parameters = mcrl2_aterm_get_argument(unsafe { &*pbes.initial_state() }, 1);
    let true_term = mcrl2_aterm_get_argument(unsafe { &*parameters }, 0);

    let spec = mcrl2_pbes_data_specification(&pbes.pbes);
    let mut context = mcrl2_pbes_create_rewrite_context(&spec).expect("the context should be built");

    mcrl2_pbes_rewrite_set_assignments(context.pin_mut(), &[b], &[true_term]).expect("assign b");
    let substituted =
        unsafe { mcrl2_pbes_rewrite_formula(context.pin_mut(), &*b) }.expect("rewrite under b := true");
    assert_eq!(print(substituted), print(true_term), "b := true should be applied");

    // The context maintains sigma incrementally rather than clearing it, so an
    // empty assignment has to actively undo the previous one. If it does not,
    // `b` keeps its old value here.
    mcrl2_pbes_rewrite_set_assignments(context.pin_mut(), &[], &[]).expect("clear the assignment");
    let unsubstituted =
        unsafe { mcrl2_pbes_rewrite_formula(context.pin_mut(), &*b) }.expect("rewrite without b");
    assert_eq!(
        print(unsubstituted),
        print(b),
        "b should be unbound again after an empty assignment"
    );
}

#[test]
fn pbes_rewrite_formula_does_not_grow_the_term_pool() {
    let _guard = lock_pool();

    // A quantifier over an infinite sort that the rewriter cannot enumerate
    // away. Each attempt mints fresh variable names from the rewriter's
    // identifier generator, whose counter only ever increases, so unless it is
    // reset per call every rewrite interns names that are never reused.
    let pbes = TestPbes::parse(
        "pbes mu X(n: Nat) = forall m: Nat . val(m > 0) || X(n);\n\ninit X(0);",
    );
    let formula = pbes.formula();

    let spec = mcrl2_pbes_data_specification(&pbes.pbes);
    let mut context = mcrl2_pbes_create_rewrite_context(&spec).expect("the context should be built");
    mcrl2_pbes_rewrite_set_assignments(context.pin_mut(), &[], &[]).expect("an empty assignment");

    let mut sizes = Vec::new();
    for _ in 0..3 {
        for _ in 0..100 {
            unsafe { mcrl2_pbes_rewrite_formula(context.pin_mut(), &*formula) }.expect("rewrite");
        }
        sizes.push(mcrl2_aterm_pool_size());
    }

    // Rewriting the same formula again must not add terms. Comparing batches
    // rather than an absolute number keeps this independent of what the rest of
    // the process put in the pool.
    assert!(
        sizes.windows(2).all(|pair| pair[0] == pair[1]),
        "repeatedly rewriting one formula grew the term pool: {sizes:?}"
    );
}

#[test]
fn replace_propositional_variables_permutes_the_parameters() {
    let _guard = lock_pool();

    let pbes = TestPbes::parse("pbes mu X(n, m: Nat) = X(m, n);\n\ninit X(0, 1);");
    let formula = pbes.formula();
    assert_eq!(
        print(formula),
        "PropVarInst(X,[DataVarId(m,SortId(Nat)),DataVarId(n,SortId(Nat))])"
    );

    let swapped = mcrl2_pbes_expression_replace_propositional_variables(unsafe { &*formula }, &vec![1, 0])
        .expect("[1, 0] is a permutation");
    assert_eq!(
        print(mcrl2_aterm_get_address(&swapped)),
        "PropVarInst(X,[DataVarId(n,SortId(Nat)),DataVarId(m,SortId(Nat))])"
    );

    // The identity permutation leaves the instantiation alone.
    let identity = mcrl2_pbes_expression_replace_propositional_variables(unsafe { &*formula }, &vec![0, 1])
        .expect("[0, 1] is a permutation");
    assert_eq!(print(mcrl2_aterm_get_address(&identity)), print(formula));
}

#[test]
fn replace_propositional_variables_rejects_a_non_permutation() {
    let _guard = lock_pool();

    let pbes = TestPbes::parse("pbes mu X(n, m: Nat) = X(m, n);\n\ninit X(0, 1);");
    let formula = pbes.formula();

    // Writing two source positions to the same target would leave an entry of
    // the new parameter list default constructed, that is, an unassigned term.
    let error = expect_error(
        mcrl2_pbes_expression_replace_propositional_variables(unsafe { &*formula }, &vec![0, 0]),
        "[0, 0] is not a permutation",
    );
    assert!(error.what().contains("Duplicate target index"), "unexpected error: {error}");

    // A target position beyond the parameter list would be written out of bounds.
    let error = expect_error(
        mcrl2_pbes_expression_replace_propositional_variables(unsafe { &*formula }, &vec![0, 5]),
        "5 is not a parameter position",
    );
    assert!(error.what().contains("Index out of bounds"), "unexpected error: {error}");

    // A permutation of the wrong arity cannot describe this PBES. This one is
    // only detected once the traversal reaches an instantiation, which is
    // exactly the path that used to abort the process.
    let error = expect_error(
        mcrl2_pbes_expression_replace_propositional_variables(unsafe { &*formula }, &vec![0]),
        "the instantiation has two parameters",
    );
    assert!(
        error.what().contains("does not match the number of parameters"),
        "unexpected error: {error}"
    );
}

#[test]
fn stategraph_accessors_reject_an_out_of_range_index() {
    let _guard = lock_pool();

    let pbes = TestPbes::parse(
        "pbes mu X(n: Nat, b: Bool) = val(n < 3) && X(n + 1, b);\n\ninit X(0, true);",
    );
    let algorithm = mcrl2_stategraph_local_algorithm_run(&pbes.pbes).expect("stategraph should run");

    let equations = mcrl2_stategraph_local_algorithm_equations(&algorithm);
    assert!(equations > 0, "the PBES has an equation");
    assert!(
        mcrl2_stategraph_local_algorithm_equation(&algorithm, equations - 1).is_ok(),
        "the last equation should be reachable"
    );
    assert!(
        mcrl2_stategraph_local_algorithm_equation(&algorithm, equations).is_err(),
        "one past the last equation must be rejected"
    );

    let graphs = mcrl2_stategraph_local_algorithm_cfgs(&algorithm);
    assert!(
        mcrl2_stategraph_local_algorithm_cfg(&algorithm, graphs).is_err(),
        "one past the last control flow graph must be rejected"
    );

    for index in 0..graphs {
        let graph = mcrl2_stategraph_local_algorithm_cfg(&algorithm, index)
            .unwrap_or_else(|error| panic!("control flow graph {index} should exist: {error}"));
        let vertices = mcrl2_local_control_flow_graph_vertices(graph);
        assert!(
            mcrl2_local_control_flow_graph_vertex(graph, vertices).is_err(),
            "one past the last vertex of graph {index} must be rejected"
        );
    }
}

#[test]
fn data_specification_accessors_strip_the_function_symbol_index() {
    let _guard = lock_pool();

    let spec = mcrl2_data_specification_from_string(
        "sort D = struct a | b;
             A = Nat;
             E;

         cons c: E;
         map f: Nat -> Nat;

         var x: Nat;
         eqn f(x) = x + 1;
        ",
    );

    // Constructors, mappings and equations are reported in the serialised form
    // that mCRL2 writes to disk, in which the rewriter-internal index of a
    // function symbol is dropped: OpId(name, sort, index) becomes
    // OpIdNoIndex(name, sort). Note that OpIdNoIndex( does not contain OpId( ,
    // so the second assertion really rules out an unstripped symbol.
    for (what, list) in [
        (
            "constructors",
            mcrl2_data_specification_user_defined_constructors(&spec),
        ),
        ("mappings", mcrl2_data_specification_user_defined_mappings(&spec)),
        ("equations", mcrl2_data_specification_user_defined_equations(&spec)),
    ] {
        let printed = print(mcrl2_aterm_get_address(&list));
        assert!(
            printed.contains("OpIdNoIndex("),
            "{what} should contain function symbols: {printed}"
        );
        assert!(!printed.contains("OpId("), "{what} should carry no index: {printed}");
    }

    // The stripping reaches nested function symbols too, not just the head.
    let equations = print(mcrl2_aterm_get_address(
        &mcrl2_data_specification_user_defined_equations(&spec),
    ));
    assert!(
        equations.contains("DataAppl(OpIdNoIndex(f,"),
        "the applied symbol should be stripped as well: {equations}"
    );

    // Sorts and aliases hold sort expressions only, so they are passed through
    // unchanged rather than traversed. Struct constructors are part of the alias
    // and are not function symbols, so they are unaffected.
    assert_eq!(
        print(mcrl2_aterm_get_address(&mcrl2_data_specification_user_defined_sorts(&spec))),
        "[SortId(E)]"
    );
    assert_eq!(
        print(mcrl2_aterm_get_address(&mcrl2_data_specification_user_defined_aliases(&spec))),
        "[SortRef(SortId(D),SortStruct([StructCons(a,[],),StructCons(b,[],)])),\
SortRef(SortId(A),SortId(Nat))]"
    );
}

#[test]
fn assignment_pairs_are_shared_between_the_data_and_pbes_bridges() {
    let _guard = lock_pool();

    let pbes = TestPbes::parse("pbes mu X(b: Bool) = val(b) && X(b);\n\ninit X(true);");
    let formula = pbes.formula();
    let parameters = mcrl2_aterm_get_argument(unsafe { &*pbes.initial_state() }, 1);
    let true_term = mcrl2_aterm_get_argument(unsafe { &*parameters }, 0);
    // The left conjunct of `val(b) && X(b)` is the data expression `b`.
    let b = mcrl2_aterm_get_argument(unsafe { &*formula }, 0);

    // One substitution value, accepted by both bridges. Keeping these as two
    // distinct cxx structs would not compile.
    let sigma = vec![assignment_pair {
        lhs: b,
        rhs: true_term,
    }];

    let data_result = mcrl2_data_expression_replace_variables(unsafe { &*b }, &sigma);
    assert_eq!(print(mcrl2_aterm_get_address(&data_result)), print(true_term));

    let pbes_result = mcrl2_pbes_expression_replace_variables(unsafe { &*formula }, &sigma);
    let printed = print(mcrl2_aterm_get_address(&pbes_result));
    assert!(
        !printed.contains("DataVarId(b,"),
        "b should have been substituted everywhere: {printed}"
    );
}
