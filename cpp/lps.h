/// Wrapper around the lps library of the mCRL2 toolset.

#ifndef MCRL2_SYS_CPP_LPS_H
#define MCRL2_SYS_CPP_LPS_H

#include "mcrl2/data/consistency.h"
#include "mcrl2/lps/detail/instantiate_global_variables.h"
#include "mcrl2/lps/detail/replace_global_variables.h"
#include "mcrl2/lps/io.h"
#include "mcrl2/lps/lpsreach.h"
#include "mcrl2/lps/parse.h"
#include "mcrl2/lps/multi_action.h"
#include "mcrl2/lps/one_point_rule_rewrite.h"
#include "mcrl2/lps/order_summand_variables.h"
#include "mcrl2/lps/replace_constants_by_variables.h"
#include "mcrl2/lps/resolve_name_clashes.h"
#include "mcrl2/lps/stochastic_specification.h"
#include "mcrl2/process/process_expression.h"

#include "mcrl2/data/enumerator.h"
#include "mcrl2/data/substitution_utility.h"
#include "mcrl2/data/substitutions/mutable_indexed_substitution.h"

#include "rust/cxx.h"
#include <mcrl2/data/rewriter.h>
#include <mcrl2/data/substitutions/assignment_sequence_substitution.h>

namespace mcrl2::lps {

inline std::unique_ptr<stochastic_specification>
mcrl2_lps_load_from_lps_file(rust::Str filename) {
  stochastic_specification result;
  load_lps(result, static_cast<std::string>(filename));
  return std::make_unique<stochastic_specification>(result);
}

inline std::unique_ptr<stochastic_specification>
mcrl2_lps_load_from_text_file(rust::Str filename) {
  stochastic_specification result;
  std::ifstream ifs(static_cast<std::string>(filename));
  if (!ifs.good())
  {
    throw mcrl2::runtime_error("Could not open file " + static_cast<std::string>(filename) + ".");
  }
  parse_lps(ifs, result);
  return std::make_unique<stochastic_specification>(result);
}

/// \brief Bundles a preprocessed specification together with the constant
///        substitution produced by replace_constants_by_variables.
///
/// The mCRL2 explorer keeps the assignments of the fresh @rewr_var variables in
/// its global substitution (m_global_sigma) and copies them into every worker's
/// sigma. We mirror that by recording them here as an assignment_list so they
/// can be seeded into each enumeration context's substitution on the Rust side.
struct preprocessed_specification {
  stochastic_specification spec;
  data::assignment_list constant_assignments;
};

/// \brief Preprocesses an LPS for state-space exploration, applying the same
///        steps (and in the same order) as the mCRL2 explorer (see
///        lps/explorer.h and lps/lpsreach.h). Each step can be toggled
///        individually.
///
/// When \p replace_constants_by_variables_flag is set, constant subexpressions
/// are replaced by fresh variables and the corresponding (variable := value)
/// substitution is recorded in the returned struct so it can be applied to the
/// enumeration substitution, exactly as the explorer does via m_global_sigma.
std::unique_ptr<preprocessed_specification>
mcrl2_lps_preprocess_symbolic_exploration(
    const stochastic_specification &lpsspec,
    bool instantiate_global_variables_flag,
    bool order_summand_variables_flag,
    bool resolve_name_clashes_flag,
    bool one_point_rule_rewrite_flag,
    bool replace_constants_by_variables_flag);

/// \brief Returns a copy of the preprocessed specification.
inline std::unique_ptr<stochastic_specification>
mcrl2_preprocessed_specification_spec(const preprocessed_specification &pp) {
  return std::make_unique<stochastic_specification>(pp.spec);
}

/// \brief Returns the address of the constant substitution (an assignment_list)
///        captured during preprocessing. The list is empty when
///        replace_constants_by_variables was not applied.
inline const atermpp::detail::_aterm *
mcrl2_preprocessed_specification_constant_assignments(
    const preprocessed_specification &pp) {
  return atermpp::detail::address(pp.constant_assignments);
}

inline std::size_t
mcrl2_lps_num_of_action_summands(const stochastic_specification &spec) {
  return spec.process().action_summands().size();
}

inline std::unique_ptr<stochastic_action_summand>
mcrl2_lps_action_summand(const stochastic_specification &spec,
                         std::size_t index) {
  return std::make_unique<stochastic_action_summand>(
      spec.process().action_summands()[index]);
}

inline std::unique_ptr<stochastic_process_initializer>
mcrl2_lps_process_initializer(const stochastic_specification &spec) {
  return std::make_unique<stochastic_process_initializer>(
      spec.initial_process());
}

/// \brief Pretty-prints a multi-action term using the mCRL2 pretty printer.
inline rust::String
mcrl2_lps_multi_action_to_string(const atermpp::detail::_aterm &input) {
  atermpp::unprotected_aterm_core tmp(&input);
  return lps::pp(atermpp::down_cast<lps::multi_action>(tmp));
}

/// \brief Returns the address of the tau (empty) multi-action term.
///
/// The returned aterm is kept alive by a function-local static, so the pointer
/// is valid for the lifetime of the process. Callers must still protect the
/// term on the Rust side before relying on it across other aterm operations.
inline const atermpp::detail::_aterm *mcrl2_lps_tau_multi_action() {
  static const lps::multi_action tau{};
  return atermpp::detail::address(tau);
}

inline const atermpp::detail::_aterm *
mcrl2_lps_action_summand_condition(const stochastic_action_summand &summand) {
  return atermpp::detail::address(summand.condition());
}

inline const atermpp::detail::_aterm *mcrl2_lps_action_summand_multi_action(
    const stochastic_action_summand &summand) {
  return atermpp::detail::address(summand.multi_action());
}

inline const atermpp::detail::_aterm *
mcrl2_lps_action_summand_summation_variables(
    const stochastic_action_summand &summand) {
  return atermpp::detail::address(summand.summation_variables());
}

inline const atermpp::detail::_aterm *
mcrl2_lps_action_summand_assignments(const stochastic_action_summand &summand) {
  return atermpp::detail::address(summand.assignments());
}

inline const atermpp::detail::_aterm *
mcrl2_lps_process_parameters(const stochastic_specification &spec) {
  return atermpp::detail::address(spec.process().process_parameters());
}

inline const atermpp::detail::_aterm *mcrl2_lps_process_initializer_expressions(
    const stochastic_process_initializer &init) {
  return atermpp::detail::address(init.expressions());
}

struct learn_successors_context {
  // Owned copy of the data specification. enumerator_algorithm stores a const
  // reference to the data spec (m_dataspec in enumerator.h), so this member
  // must outlive the enumerator. Declared first so it is initialized before
  // rewr and enumerator in the member-initializer list.
  data::data_specification m_dataspec;
  data::rewriter rewr;
  data::mutable_indexed_substitution<> sigma;
  data::enumerator_identifier_generator id_generator;
  data::enumerator_algorithm<> enumerator;
  // Protected next-state values for the current solution. These keep the
  // rewritten terms alive across term creation that happens before the
  // callback is invoked (e.g. building the rewritten multi-action).
  std::vector<data::data_expression> value_terms;
  // Raw addresses of value_terms, handed to the callback as a slice.
  std::vector<const atermpp::detail::_aterm *> values;
  // Protected summation-variable solution values for the current solution, and
  // their raw addresses. Used by the recompute caching strategy, which caches
  // the summation-variable assignments and later recomputes the next state and
  // multi-action from them (see mcrl2_lps_compute_successor).
  std::vector<data::data_expression> summation_value_terms;
  std::vector<const atermpp::detail::_aterm *> summation_values;

  // Protected result of the most recent mcrl2_lps_rewrite_under_sigma call.
  // Keeps the rewritten term alive until the Rust side protects it.
  data::data_expression rewrite_result;

  learn_successors_context(const data::data_specification &dataspec)
      : m_dataspec(dataspec),
        rewr(m_dataspec, data::used_data_equation_selector(m_dataspec)),
        enumerator(rewr, m_dataspec, rewr, id_generator, false) {}
};

/// \brief Rewrites \p expr under the context's current substitution (sigma) and
///        returns the address of the resulting term.
///
/// Used to normalise the initial state expressions, which may contain the fresh
/// @rewr_var variables introduced by replace_constants_by_variables. This
/// mirrors the explorer's compute_state, which rewrites every state expression
/// under the global sigma so the constant variables resolve to their values.
///
/// The result is kept alive in the context's `rewrite_result` member, so the
/// returned pointer is valid until the next call. Callers must protect the term
/// on the Rust side before performing further aterm operations.
const atermpp::detail::_aterm *
mcrl2_lps_rewrite_under_sigma(learn_successors_context &context,
                              const atermpp::detail::_aterm &expr);

inline std::unique_ptr<learn_successors_context>
mcrl2_lps_create_learn_successors_context(
    const stochastic_specification &spec) {
  return std::make_unique<learn_successors_context>(spec.data());
}

inline std::unique_ptr<learn_successors_context>
mcrl2_lps_create_learn_successors_context_from_data_spec(
    const data::data_specification &data_spec) {
  return std::make_unique<learn_successors_context>(data_spec);
}

/// Assign variables in the context substitution (sigma).
inline void mcrl2_lps_set_assignments(
    learn_successors_context &context,
    rust::Slice<const atermpp::detail::_aterm *const> variables,
    rust::Slice<const atermpp::detail::_aterm *const> values) {
  auto &sigma = context.sigma;

  for (std::size_t i = 0; i < variables.size(); i++) {
    atermpp::unprotected_aterm_core var_term(variables[i]);
    atermpp::unprotected_aterm_core value_term(values[i]);
    sigma[atermpp::down_cast<data::variable>(var_term)] =
        atermpp::down_cast<data::data_expression>(value_term);
  }
}

/// \brief Enumerate all solutions for the summand variables that satisfy the
/// condition,
///        given the read parameter assignments from the current state.
///
/// For each solution found, calls \p callback with \p callback_context, a
/// slice of the rewritten next-state values (one per assignment in the
/// summand) and a pointer to the rewritten multi-action term obtained by
/// applying the current substitution to \p multi_action.
void mcrl2_lps_enumerate(
    learn_successors_context &context, const atermpp::detail::_aterm &condition,
    const atermpp::detail::_aterm &summation_variables,
    const atermpp::detail::_aterm &assignments,
    const atermpp::detail::_aterm &multi_action,
    std::uint8_t *callback_context,
    rust::Fn<void(std::uint8_t *,
                  rust::Slice<const atermpp::detail::_aterm *const>,
                  const atermpp::detail::_aterm *)>
        callback);

/// \brief Like mcrl2_lps_enumerate, but additionally passes the
///        summation-variable solution (the assignment to the summation
///        variables) for each solution to the callback.
///
/// The summation-variable solution can be cached and later replayed via
/// mcrl2_lps_compute_successor to recompute the transition without
/// re-enumerating the condition.
void mcrl2_lps_enumerate_solutions(
    learn_successors_context &context, const atermpp::detail::_aterm &condition,
    const atermpp::detail::_aterm &summation_variables,
    const atermpp::detail::_aterm &assignments,
    const atermpp::detail::_aterm &multi_action,
    std::uint8_t *callback_context,
    rust::Fn<void(std::uint8_t *,
                  rust::Slice<const atermpp::detail::_aterm *const>,
                  const atermpp::detail::_aterm *,
                  rust::Slice<const atermpp::detail::_aterm *const>)>
        callback);

/// \brief Recomputes the next-state values and multi-action for a previously
///        cached summation-variable solution, without re-enumerating the
///        condition.
///
/// Assigns \p summation_values to the summation variables in the current
/// substitution (which is expected to already contain the read-parameter
/// assignments of the source state), rewrites the next-state assignments and
/// the multi-action, and invokes the callback once. This mirrors mCRL2's own
/// explorer, which caches enumeration solutions and re-derives the state and
/// action via compute_state.
void mcrl2_lps_compute_successor(
    learn_successors_context &context,
    const atermpp::detail::_aterm &summation_variables,
    rust::Slice<const atermpp::detail::_aterm *const> summation_values,
    const atermpp::detail::_aterm &assignments,
    const atermpp::detail::_aterm &multi_action,
    std::uint8_t *callback_context,
    rust::Fn<void(std::uint8_t *,
                  rust::Slice<const atermpp::detail::_aterm *const>,
                  const atermpp::detail::_aterm *)>
        callback);

} // namespace mcrl2::lps

#endif // MCRL2_SYS_CPP_LPS_H