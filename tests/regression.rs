//! Regression tests for behaviour of the raw FFI that is easy to lose again.
//!
//! Several kinds of behaviour are pinned here:
//!
//! * bounds checks that guard raw slice indexing on the C++ side. `rust::Slice`
//!   only `assert`s its bounds and `NDEBUG` is defined in release, so a missing
//!   check is an out-of-bounds read reachable from safe Rust.
//! * C++ exceptions reaching Rust as `Err`. cxx generates a `noexcept` shim for
//!   every function that is not declared `-> Result<...>`, so a throw crossing
//!   such a function terminates the process instead.
//! * the substitution bookkeeping of the PBES rewrite context, which has to
//!   forget the previous assignment before applying the next one.
//! * the interaction between the aterm pool's automatic garbage collection /
//!   resizing flags and the functions that trigger those operations
//!   explicitly, which the two have to agree on even while automatic
//!   collection is disabled around parallel work.
//! * PBES preprocessing steps (global variable instantiation, quantifier
//!   simplification, the one point rule, quantifier variable ordering) that
//!   are exposed individually so callers can compose and report on them.

use std::sync::Mutex;
use std::sync::MutexGuard;

use mcrl2_sys::atermpp::ffi::_aterm;
use mcrl2_sys::atermpp::ffi::mcrl2_aterm_create_int;
use mcrl2_sys::atermpp::ffi::mcrl2_aterm_from_string;
use mcrl2_sys::atermpp::ffi::mcrl2_aterm_get_address;
use mcrl2_sys::atermpp::ffi::mcrl2_aterm_get_argument;
use mcrl2_sys::atermpp::ffi::mcrl2_aterm_pool_collect_garbage;
use mcrl2_sys::atermpp::ffi::mcrl2_aterm_pool_enable_automatic_garbage_collection;
use mcrl2_sys::atermpp::ffi::mcrl2_aterm_pool_resize;
use mcrl2_sys::atermpp::ffi::mcrl2_aterm_pool_resize_is_needed;
use mcrl2_sys::atermpp::ffi::mcrl2_aterm_pool_size;
use mcrl2_sys::atermpp::ffi::mcrl2_aterm_print;
use mcrl2_sys::cxx::CxxVector;
use mcrl2_sys::cxx::Exception;
use mcrl2_sys::cxx::UniquePtr;
use mcrl2_sys::data::ffi::assignment_pair;
use mcrl2_sys::data::ffi::mcrl2_create_rewriter_jitty;
use mcrl2_sys::data::ffi::mcrl2_data_expression_remove_index;
use mcrl2_sys::data::ffi::mcrl2_data_expression_replace_variables;
use mcrl2_sys::data::ffi::mcrl2_data_specification_from_string;
use mcrl2_sys::data::ffi::mcrl2_data_specification_user_defined_aliases;
use mcrl2_sys::data::ffi::mcrl2_data_specification_user_defined_constructors;
use mcrl2_sys::data::ffi::mcrl2_data_specification_user_defined_equations;
use mcrl2_sys::data::ffi::mcrl2_data_specification_user_defined_mappings;
use mcrl2_sys::data::ffi::mcrl2_data_specification_user_defined_sorts;
use mcrl2_sys::data::ffi::mcrl2_pbes_expression_replace_variables;
use mcrl2_sys::lps::ffi::mcrl2_lps_action_summand;
use mcrl2_sys::lps::ffi::mcrl2_lps_action_summand_assignments;
use mcrl2_sys::lps::ffi::mcrl2_lps_action_summand_condition;
use mcrl2_sys::lps::ffi::mcrl2_lps_action_summand_multi_action;
use mcrl2_sys::lps::ffi::mcrl2_lps_action_summand_summation_variables;
use mcrl2_sys::lps::ffi::mcrl2_lps_create_learn_successors_context;
use mcrl2_sys::lps::ffi::mcrl2_lps_create_learn_successors_context_from_data_spec;
use mcrl2_sys::lps::ffi::mcrl2_lps_data_specification;
use mcrl2_sys::lps::ffi::mcrl2_lps_enumerate;
use mcrl2_sys::lps::ffi::mcrl2_lps_load_from_text_file;
use mcrl2_sys::lps::ffi::mcrl2_lps_multi_action_to_string;
use mcrl2_sys::lps::ffi::mcrl2_lps_process_initializer;
use mcrl2_sys::lps::ffi::mcrl2_lps_process_initializer_expressions;
use mcrl2_sys::lps::ffi::mcrl2_lps_process_parameters;
use mcrl2_sys::lps::ffi::mcrl2_lps_rewrite_under_sigma;
use mcrl2_sys::lps::ffi::mcrl2_lps_set_assignments;
use mcrl2_sys::lps::ffi::stochastic_specification;
use mcrl2_sys::pbes::ffi::mcrl2_load_pbes_from_text;
use mcrl2_sys::pbes::ffi::mcrl2_local_control_flow_graph_vertex;
use mcrl2_sys::pbes::ffi::mcrl2_local_control_flow_graph_vertices;
use mcrl2_sys::pbes::ffi::mcrl2_pbes_create_rewrite_context;
use mcrl2_sys::pbes::ffi::mcrl2_pbes_data_specification;
use mcrl2_sys::pbes::ffi::mcrl2_pbes_equation_formula;
use mcrl2_sys::pbes::ffi::mcrl2_pbes_equations;
use mcrl2_sys::pbes::ffi::mcrl2_pbes_expression_replace_propositional_variables;
use mcrl2_sys::pbes::ffi::mcrl2_pbes_initial_state;
use mcrl2_sys::pbes::ffi::mcrl2_pbes_instantiate_global_variables;
use mcrl2_sys::pbes::ffi::mcrl2_pbes_one_point_rule;
use mcrl2_sys::pbes::ffi::mcrl2_pbes_order_quantified_variables;
use mcrl2_sys::pbes::ffi::mcrl2_pbes_rewrite_formula;
use mcrl2_sys::pbes::ffi::mcrl2_pbes_rewrite_set_assignments;
use mcrl2_sys::pbes::ffi::mcrl2_pbes_simplify_quantifiers;
use mcrl2_sys::pbes::ffi::mcrl2_pbes_to_string;
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

/// Parses a linear process specification given as mCRL2 source text.
///
/// `mcrl2_lps_load_from_text_file` only reads from a file, so this writes
/// `text` to a uniquely-named file under the system temp directory first. The
/// name is salted with the process id and `salt` (a caller-chosen tag) so
/// concurrently-running tests, and multiple calls within the same test, never
/// collide on the same path.
fn parse_lps_text(salt: &str, text: &str) -> UniquePtr<stochastic_specification> {
    let path = std::env::temp_dir().join(format!(
        "mcrl2-sys-regression-{}-{salt}.mcrl2",
        std::process::id()
    ));
    std::fs::write(&path, text).expect("the temporary LPS source should be writable");
    let lps = mcrl2_lps_load_from_text_file(path.to_str().expect("a UTF-8 path"));
    std::fs::remove_file(&path).ok();
    lps.expect("the linear process above should parse")
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

#[test]
fn aterm_pool_resize_is_needed_reacts_to_growth() {
    let _guard = lock_pool();

    // A large, distinct batch of unprotected terms drives at least one of the
    // pool's hash tables (there are several, one per arity plus the function
    // symbol table) past its load factor; the automatic resizing that
    // `mcrl2_aterm_pool_enable_automatic_resize` documents then catches up
    // with it. Growing the pool like this is the scenario the flag exists to
    // guard, so it must end up reporting "no resize needed" rather than
    // getting stuck reporting "needed" forever.
    //
    // How many terms that takes depends on what the pool already holds
    // (shared with every other test in this binary), so grow it in batches
    // until the guard clears instead of picking one fixed count.
    let mut value = 0u64;
    for _ in 0..40 {
        if !mcrl2_aterm_pool_resize_is_needed() {
            break;
        }
        for _ in 0..50_000 {
            mcrl2_aterm_create_int(value);
            value += 1;
        }
    }
    assert!(
        !mcrl2_aterm_pool_resize_is_needed(),
        "growing the pool should have been caught by automatic resizing after {value} fresh terms"
    );

    // An explicit resize on top of that is a no-op that leaves the guard
    // satisfied, which is the pattern the function exists to support: check
    // it from a shared section, resize from a quiescent point when it says
    // to.
    mcrl2_aterm_pool_resize();
    assert!(
        !mcrl2_aterm_pool_resize_is_needed(),
        "resize_is_needed should stay false after an explicit resize"
    );
}

#[test]
fn collect_garbage_reclaims_terms_while_automatic_collection_is_disabled() {
    let _guard = lock_pool();

    // Automatic collection is disabled around parallel work so that the pool
    // never collects on its own from inside a shared section (see
    // `mcrl2_aterm_pool_enable_automatic_resize`). `collect_garbage` schedules
    // a collection explicitly from a quiescent point instead, so it must still
    // do its job while automatic collection is off; the underlying
    // `aterm_pool::collect_impl` used to ignore every collection, including
    // this explicit one, while the flag was disabled.
    mcrl2_aterm_pool_enable_automatic_garbage_collection(false);

    // Create a batch of unprotected, unreferenced terms: nothing roots them,
    // so they are garbage as soon as they are created. `aterm_int` values are
    // interned, so the range is offset well away from the small values other
    // tests in this binary create, or creating them again would just return
    // the existing (still live) term instead of growing the pool.
    let before = mcrl2_aterm_pool_size();
    for value in 900_000_000..900_010_000 {
        mcrl2_aterm_create_int(value);
    }
    let after_creating = mcrl2_aterm_pool_size();
    assert!(
        after_creating > before,
        "the batch of fresh terms should have grown the pool: {before} -> {after_creating}"
    );

    mcrl2_aterm_pool_collect_garbage();
    let after_collecting = mcrl2_aterm_pool_size();

    mcrl2_aterm_pool_enable_automatic_garbage_collection(true);

    assert!(
        after_collecting < after_creating,
        "collect_garbage should reclaim unreferenced terms even while automatic \
         collection is disabled: {after_creating} -> {after_collecting}"
    );
}

#[test]
fn data_expression_remove_index_strips_nested_op_id_markers() {
    let _guard = lock_pool();

    // `remove_index` is a bottom-up replacement keyed only on the `OpId`
    // function symbol, so it is exercised directly on raw terms rather than
    // through a data specification: this pins that it recurses into
    // subterms instead of only stripping the head.
    let term = mcrl2_aterm_from_string("DataAppl(OpId(f,Bool,0),OpId(c,Bool,1))")
        .expect("a raw OpId term should parse");
    let stripped = mcrl2_data_expression_remove_index(unsafe { &*mcrl2_aterm_get_address(&term) });
    assert_eq!(
        print(mcrl2_aterm_get_address(&stripped)),
        "DataAppl(OpIdNoIndex(f,Bool),OpIdNoIndex(c,Bool))"
    );

    // A term without any OpId subterm is returned unchanged.
    let plain = mcrl2_aterm_from_string("f(a,b)").expect("a plain term should parse");
    let unchanged = mcrl2_data_expression_remove_index(unsafe { &*mcrl2_aterm_get_address(&plain) });
    assert_eq!(print(mcrl2_aterm_get_address(&unchanged)), "f(a,b)");
}

#[test]
fn lps_data_specification_returns_the_lps_own_data() {
    let _guard = lock_pool();

    let lps = parse_lps_text(
        "lps-data-specification",
        "sort D = struct d1 | d2;
         map f: D -> D;
         var x: D;
         eqn f(x) = x;
         act a: D;
         proc P(x: D) = a(f(x)) . P(x);
         init P(d1);
        ",
    );

    let spec = mcrl2_lps_data_specification(&lps);

    // The struct sort is a system-recognisable alias, and the mapping that
    // uses it is reported alongside it, so both round trip through the copy
    // handed back by `mcrl2_lps_data_specification`.
    let aliases = print(mcrl2_aterm_get_address(&mcrl2_data_specification_user_defined_aliases(&spec)));
    assert!(
        aliases.contains("SortId(D)"),
        "the LPS's own sort D should be in its data specification: {aliases}"
    );

    let mappings = print(mcrl2_aterm_get_address(&mcrl2_data_specification_user_defined_mappings(&spec)));
    assert!(
        mappings.contains("OpIdNoIndex(f,"),
        "the LPS's own mapping f should be in its data specification: {mappings}"
    );

    // The copy is usable on its own, independent of the LPS it came from.
    assert!(
        !mcrl2_create_rewriter_jitty(&spec).is_null(),
        "a rewriter should be buildable from the returned data specification"
    );
}

#[test]
fn lps_load_from_text_file_reports_a_missing_file() {
    let _guard = lock_pool();

    // `std::ifstream::good()` is checked explicitly before parsing so that a
    // missing file surfaces as a catchable exception instead of parsing an
    // empty stream (which would fail with a much more confusing parse error,
    // or worse, silently produce an empty specification).
    let path = std::env::temp_dir().join(format!(
        "mcrl2-sys-regression-{}-does-not-exist.mcrl2",
        std::process::id()
    ));
    std::fs::remove_file(&path).ok();

    let error = expect_error(
        mcrl2_lps_load_from_text_file(path.to_str().expect("a UTF-8 path")),
        "the file does not exist",
    );
    assert!(
        error.what().contains("Could not open file"),
        "unexpected error: {error}"
    );
}

/// Everything `mcrl2_lps_enumerate` reported for one solution, copied out of
/// the borrowed callback arguments since they are only valid for the
/// duration of the call.
struct EnumerateSolution {
    values: Vec<String>,
    multi_action: String,
}

fn record_enumerate_solution(context: *mut u8, values: &[*const _aterm], multi_action: *const _aterm) {
    let solutions = unsafe { &mut *(context as *mut Vec<EnumerateSolution>) };
    solutions.push(EnumerateSolution {
        values: values.iter().map(|&v| print(v)).collect(),
        multi_action: mcrl2_lps_multi_action_to_string(unsafe { &*multi_action }),
    });
}

#[test]
fn lps_enumerate_rewrites_the_multi_action_and_keeps_process_parameters_assigned() {
    let _guard = lock_pool();

    // `b` is a summation variable of the summand (bound only for the duration
    // of one solution), `p` is a process parameter (set once per state and
    // read by every summand). The multi-action uses `b`, so this pins two
    // historical fixes at once: the multi-action's own arguments must be
    // rewritten under each solution's substitution (mCRL2#b652ad3), and
    // `mcrl2_lps_enumerate` must only remove the summation variable from the
    // context's substitution afterwards, not the process parameter it shares
    // the substitution with (mCRL2#6e9d394).
    let lps = parse_lps_text(
        "lps-enumerate",
        "act a: Bool;
         proc P(p: Nat) = sum b: Bool . (true) -> a(b) . P(p);
         init P(5);
        ",
    );

    let summand =
        mcrl2_lps_action_summand(&lps, 0).expect("the LPS has exactly one action summand");
    let condition = mcrl2_lps_action_summand_condition(&summand);
    let multi_action = mcrl2_lps_action_summand_multi_action(&summand);
    let summation_variables = mcrl2_lps_action_summand_summation_variables(&summand);
    let assignments = mcrl2_lps_action_summand_assignments(&summand);

    // The head of the process parameter list is `p`; the head of the
    // summation variable list is `b`.
    let p = mcrl2_aterm_get_argument(unsafe { &*mcrl2_lps_process_parameters(&lps) }, 0);
    let b = mcrl2_aterm_get_argument(unsafe { &*summation_variables }, 0);

    // The concrete value 5 comes from the LPS's own initial state, so it is
    // guaranteed to be a term of the same data specification as everything
    // else the context works with.
    let init = mcrl2_lps_process_initializer(&lps).expect("the LPS has an initial state");
    let five = mcrl2_aterm_get_argument(unsafe { &*mcrl2_lps_process_initializer_expressions(&init) }, 0);

    let mut context = mcrl2_lps_create_learn_successors_context(&lps);
    mcrl2_lps_set_assignments(context.pin_mut(), &[p], &[five]).expect("assign p := 5");

    let mut solutions: Vec<EnumerateSolution> = Vec::new();
    unsafe {
        mcrl2_lps_enumerate(
            context.pin_mut(),
            &*condition,
            &*summation_variables,
            &*assignments,
            &*multi_action,
            (&mut solutions as *mut Vec<EnumerateSolution>).cast::<u8>(),
            record_enumerate_solution,
        );
    }

    // `b: Bool` has exactly two solutions, and each must carry its own value
    // through into the rewritten multi-action rather than leaking the
    // previous iteration's value or the raw variable (b652ad3, b0cedc4).
    let mut multi_actions: Vec<&str> = solutions.iter().map(|s| s.multi_action.as_str()).collect();
    multi_actions.sort();
    assert_eq!(
        multi_actions,
        vec!["a(false)", "a(true)"],
        "each solution's multi-action should carry that solution's own value for b"
    );

    // `p` does not depend on `b`, so both solutions must report the same
    // value for it, and it must still be the value assigned before
    // enumerate ran.
    for solution in &solutions {
        assert_eq!(
            solution.values,
            vec![print(five)],
            "p's assignment should be visible, unaffected by enumerating b"
        );
    }

    // After enumerate returns, `p` must still be assigned (6e9d394: only the
    // summation variable is removed from the substitution)...
    let p_after = unsafe { mcrl2_lps_rewrite_under_sigma(context.pin_mut(), &*p) };
    assert_eq!(
        print(p_after),
        print(five),
        "p's assignment should survive mcrl2_lps_enumerate"
    );

    // ...while `b` must be free again (it was only ever bound per-solution
    // for the duration of the callback).
    let b_after = unsafe { mcrl2_lps_rewrite_under_sigma(context.pin_mut(), &*b) };
    assert_eq!(
        print(b_after),
        print(b),
        "b should be unbound again once mcrl2_lps_enumerate returns"
    );
}

fn ignore_enumerate_solution(_context: *mut u8, _values: &[*const _aterm], _multi_action: *const _aterm) {}

#[test]
fn lps_enumerate_does_not_grow_the_term_pool() {
    let _guard = lock_pool();

    // Every quantifier-style enumeration mints fresh variable names from an
    // identifier generator whose counter only ever increases; `context.id_generator.clear()`
    // rewinds it before each call so repeated enumeration reuses the same
    // names instead of interning a fresh, never-collected function symbol on
    // every call. Comparing batches (as in `pbes_rewrite_formula_does_not_grow_the_term_pool`)
    // keeps this independent of what the rest of the process put in the pool.
    let lps = parse_lps_text(
        "lps-enumerate-pool-size",
        "act a: Bool;
         proc P(p: Nat) = sum b: Bool . (true) -> a(b) . P(p);
         init P(5);
        ",
    );
    let summand =
        mcrl2_lps_action_summand(&lps, 0).expect("the LPS has exactly one action summand");
    let condition = mcrl2_lps_action_summand_condition(&summand);
    let multi_action = mcrl2_lps_action_summand_multi_action(&summand);
    let summation_variables = mcrl2_lps_action_summand_summation_variables(&summand);
    let assignments = mcrl2_lps_action_summand_assignments(&summand);

    let mut context = mcrl2_lps_create_learn_successors_context(&lps);

    let mut sizes = Vec::new();
    for _ in 0..3 {
        for _ in 0..100 {
            unsafe {
                mcrl2_lps_enumerate(
                    context.pin_mut(),
                    &*condition,
                    &*summation_variables,
                    &*assignments,
                    &*multi_action,
                    std::ptr::null_mut(),
                    ignore_enumerate_solution,
                );
            }
        }
        sizes.push(mcrl2_aterm_pool_size());
    }

    assert!(
        sizes.windows(2).all(|pair| pair[0] == pair[1]),
        "repeatedly enumerating the same summand grew the term pool: {sizes:?}"
    );
}

#[test]
fn pbes_instantiate_global_variables_substitutes_a_representative_value() {
    let _guard = lock_pool();

    let mut pbes = TestPbes::parse(
        "glob n: Nat;\n\npbes mu X(b: Bool) = val(n < 1) || X(b);\n\ninit X(true);",
    );

    mcrl2_pbes_instantiate_global_variables(pbes.pbes.pin_mut()).expect("Nat has a representative value");
    let printed = mcrl2_pbes_to_string(&pbes.pbes);
    assert!(
        !printed.contains("glob"),
        "the global variable declaration should be gone: {printed}"
    );
    assert!(
        printed.contains("val(0 < 1)"),
        "n should have been substituted by its representative value 0: {printed}"
    );

    // An empty set of global variables is accepted as a no-op.
    mcrl2_pbes_instantiate_global_variables(pbes.pbes.pin_mut()).expect("no global variables left");
}

#[test]
fn pbes_instantiate_global_variables_rejects_an_uninhabited_sort() {
    let _guard = lock_pool();

    // `D` has no constructors, so `representative_generator` cannot produce a
    // closed term of that sort and `instantiate_global_variables` must throw
    // rather than substitute a default-constructed (empty) term.
    let mut pbes = TestPbes::parse(
        "sort D;\n\nglob d: D;\n\npbes mu X = val(d == d);\n\ninit X;",
    );

    let error = expect_error(
        mcrl2_pbes_instantiate_global_variables(pbes.pbes.pin_mut()),
        "D has no representative value",
    );
    assert!(
        error.what().contains("Cannot find a term of sort"),
        "unexpected error: {error}"
    );
}

#[test]
fn pbes_simplify_quantifiers_evaluates_data_subterms() {
    let _guard = lock_pool();

    // `1 < 2` is a closed data subterm that the rewriter can evaluate outright,
    // and the universal quantifier ranges over nothing once its body is `true`.
    let mut pbes = TestPbes::parse("pbes mu X = forall b: Bool . val(1 < 2);\n\ninit X;");

    mcrl2_pbes_simplify_quantifiers(pbes.pbes.pin_mut()).expect("simplification should succeed");
    let printed = mcrl2_pbes_to_string(&pbes.pbes);
    assert!(
        !printed.contains("forall"),
        "the quantifier over an always-true body should have been eliminated: {printed}"
    );
}

#[test]
fn pbes_one_point_rule_substitutes_the_pinned_value() {
    let _guard = lock_pool();

    // `n == 0` pins n to a single value, so the one point rule replaces
    // `exists n: Nat . n == 0 && X(n)` by `X(0)`.
    let mut pbes =
        TestPbes::parse("pbes mu X(m: Nat) = exists n: Nat . val(n == 0) && X(n);\n\ninit X(0);");

    mcrl2_pbes_one_point_rule(pbes.pbes.pin_mut()).expect("the one point rule should apply");
    let printed = mcrl2_pbes_to_string(&pbes.pbes);
    assert!(
        !printed.contains("exists"),
        "the pinned quantifier should have been eliminated: {printed}"
    );
}

#[test]
fn pbes_order_quantified_variables_makes_reordered_binders_equal() {
    let _guard = lock_pool();

    // `order_variables_to_optimise_enumeration` groups quantified variables by
    // sort, putting variables of an enumerated sort (one whose constructors
    // are all constants) ahead of the rest, with more constructors first.
    // Ordering is only guaranteed across such groups: `Three` and `Two` are
    // each the sole variable of their group, so however they were declared,
    // `x: Three` (3 constructors) must end up before `y: Two` (2
    // constructors). This is what lets a caller regenerate a PBES the same
    // way independent of the declaration order chosen by whoever wrote it.
    const PREAMBLE: &str = "sort Three = struct h1 | h2 | h3;
         Two = struct t1 | t2;

         pbes mu X = ";

    let mut first = TestPbes::parse(&format!(
        "{PREAMBLE}forall y: Two, x: Three . val(x == h1) || val(y == t1) || X;\n\ninit X;"
    ));
    let mut second = TestPbes::parse(&format!(
        "{PREAMBLE}forall x: Three, y: Two . val(x == h1) || val(y == t1) || X;\n\ninit X;"
    ));

    mcrl2_pbes_order_quantified_variables(first.pbes.pin_mut()).expect("ordering should succeed");
    mcrl2_pbes_order_quantified_variables(second.pbes.pin_mut()).expect("ordering should succeed");

    // Re-snapshot the equations: `TestPbes::formula` reads through the
    // snapshot taken at `parse` time, which the in-place rewrite above does
    // not update.
    let mut first_equations = CxxVector::new();
    mcrl2_pbes_equations(first_equations.pin_mut(), &first.pbes);
    let mut second_equations = CxxVector::new();
    mcrl2_pbes_equations(second_equations.pin_mut(), &second.pbes);

    assert_eq!(
        print(mcrl2_pbes_equation_formula(first_equations.get(0).unwrap())),
        print(mcrl2_pbes_equation_formula(second_equations.get(0).unwrap())),
        "quantifiers differing only in variable order should become identical"
    );
}
