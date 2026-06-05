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

inline std::unique_ptr<stochastic_specification>
mcrl2_lps_preprocess_symbolic_exploration(
    const stochastic_specification &lpsspec) {
  stochastic_specification result = lpsspec;
  lps::detail::instantiate_global_variables(result);
  lps::order_summand_variables(result);
  resolve_summand_variable_name_clashes(
      result); // N.B. This is a required preprocessing step.
  one_point_rule_rewrite(result);
  // replace_constants_by_variables(result, m_rewr, m_sigma);

  return std::make_unique<stochastic_specification>(result);
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

  learn_successors_context(const data::data_specification &dataspec)
      : rewr(dataspec, data::used_data_equation_selector(dataspec)),
        enumerator(rewr, dataspec, rewr, id_generator, false) {}
};

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
inline void mcrl2_lps_enumerate(
    learn_successors_context &context, const atermpp::detail::_aterm &condition,
    const atermpp::detail::_aterm &summation_variables,
    const atermpp::detail::_aterm &assignments,
    const atermpp::detail::_aterm &multi_action,
    std::uint8_t *callback_context,
    rust::Fn<void(std::uint8_t *,
                  rust::Slice<const atermpp::detail::_aterm *const>,
                  const atermpp::detail::_aterm *)>
        callback) {
  using enumerator_element = data::enumerator_list_element_with_substitution<>;
  auto &sigma = context.sigma;
  auto &rewr = context.rewr;
  auto &enumerator = context.enumerator;
  auto &values = context.values;
  auto &value_terms = context.value_terms;

  // Rewind the identifier generator so each enumeration reuses the same fresh
  // variable names (x_0, x_1, ...) instead of minting new function symbols on
  // every call. Function symbols are interned globally and never collected, so
  // without this the symbol pool grows for the whole exploration. This mirrors
  // mCRL2's own explorer, which clears the generator before each
  // generate_transitions call (see lps/explorer.h, the !m_recursive branch).
  context.id_generator.clear();

  // Interpret the summation variables as a variable_list.
  atermpp::unprotected_aterm_core tmp_vars(&summation_variables);
  const auto &variables = atermpp::down_cast<data::variable_list>(tmp_vars);

  // Interpret the assignments as an assignment_list.
  atermpp::unprotected_aterm_core tmp_assign(&assignments);
  const auto &assignment_list =
      atermpp::down_cast<data::assignment_list>(tmp_assign);

  // Interpret the multi-action term as a lps::multi_action.
  atermpp::unprotected_aterm_core tmp_multi_action(&multi_action);
  const auto &multi_action_template =
      atermpp::down_cast<lps::multi_action>(tmp_multi_action);

  // Rewrite the condition under the current substitution.
  atermpp::unprotected_aterm_core cond_term(&condition);
  data::data_expression rewritten_condition =
      rewr(atermpp::down_cast<data::data_expression>(cond_term), sigma);

  if (!data::is_false(rewritten_condition)) {
    enumerator.enumerate(
        enumerator_element(variables, rewritten_condition), sigma,
        [&](const enumerator_element &p) {
          p.add_assignments(variables, sigma, rewr);

          // Rewrite the right-hand sides of the assignments and keep them
          // protected in value_terms. We must not store only their raw
          // addresses here: constructing the rewritten multi-action below
          // creates new terms, which can reclaim these otherwise unprotected
          // terms before the callback observes them.
          value_terms.clear();
          for (const auto &assignment : assignment_list) {
            value_terms.push_back(rewr(assignment.rhs(), sigma));
          }

          // Rewrite the arguments of each action in the multi-action under the
          // current substitution.
          const process::action_list &actions = multi_action_template.actions();
          const data::data_expression &time = multi_action_template.time();
          lps::multi_action rewritten_multi_action(
              process::action_list(
                  actions.begin(), actions.end(),
                  [&](const process::action &a) {
                    const auto &args = a.arguments();
                    return process::action(
                        a.label(),
                        data::data_expression_list(
                            args.begin(), args.end(),
                            [&](const data::data_expression &x) {
                              return rewr(x, sigma);
                            }));
                  }),
              multi_action_template.has_time() ? rewr(time, sigma) : time);

          // All term creation is done; collect the addresses of the protected
          // value terms for the callback slice.
          values.clear();
          for (const auto &value : value_terms) {
            values.push_back(atermpp::detail::address(value));
          }

          callback(callback_context, {values.data(), values.size()},
                   atermpp::detail::address(rewritten_multi_action));
          return false;
        },
        data::is_false);
  }

  // Keep process-parameter assignments and only remove summation variables.
  remove_assignments(sigma, variables);
}

/// \brief Computes the rewritten next-state values and multi-action for a
///        single solution, assuming the current substitution already contains
///        the relevant assignments, and passes them to \p emit.
///
/// \p emit is invoked as emit(values_slice, multi_action_ptr). The terms
/// referenced by the slice and pointer are only valid for the duration of the
/// call.
template <typename Emit>
inline void mcrl2_lps_emit_successor(learn_successors_context &context,
                                     const data::assignment_list &assignment_list,
                                     const lps::multi_action &multi_action_template,
                                     Emit emit) {
  auto &sigma = context.sigma;
  auto &rewr = context.rewr;
  auto &values = context.values;
  auto &value_terms = context.value_terms;

  // Rewrite the right-hand sides of the assignments and keep them protected in
  // value_terms (see the note in mcrl2_lps_enumerate).
  value_terms.clear();
  for (const auto &assignment : assignment_list) {
    value_terms.push_back(rewr(assignment.rhs(), sigma));
  }

  // Rewrite the arguments of each action in the multi-action under sigma.
  const process::action_list &actions = multi_action_template.actions();
  const data::data_expression &time = multi_action_template.time();
  lps::multi_action rewritten_multi_action(
      process::action_list(actions.begin(), actions.end(),
                           [&](const process::action &a) {
                             const auto &args = a.arguments();
                             return process::action(
                                 a.label(),
                                 data::data_expression_list(
                                     args.begin(), args.end(),
                                     [&](const data::data_expression &x) {
                                       return rewr(x, sigma);
                                     }));
                           }),
      multi_action_template.has_time() ? rewr(time, sigma) : time);

  values.clear();
  for (const auto &value : value_terms) {
    values.push_back(atermpp::detail::address(value));
  }

  emit(rust::Slice<const atermpp::detail::_aterm *const>{values.data(),
                                                         values.size()},
       atermpp::detail::address(rewritten_multi_action));
}

/// \brief Like mcrl2_lps_enumerate, but additionally passes the
///        summation-variable solution (the assignment to the summation
///        variables) for each solution to the callback.
///
/// The summation-variable solution can be cached and later replayed via
/// mcrl2_lps_compute_successor to recompute the transition without
/// re-enumerating the condition.
inline void mcrl2_lps_enumerate_solutions(
    learn_successors_context &context, const atermpp::detail::_aterm &condition,
    const atermpp::detail::_aterm &summation_variables,
    const atermpp::detail::_aterm &assignments,
    const atermpp::detail::_aterm &multi_action,
    std::uint8_t *callback_context,
    rust::Fn<void(std::uint8_t *,
                  rust::Slice<const atermpp::detail::_aterm *const>,
                  const atermpp::detail::_aterm *,
                  rust::Slice<const atermpp::detail::_aterm *const>)>
        callback) {
  using enumerator_element = data::enumerator_list_element_with_substitution<>;
  auto &sigma = context.sigma;
  auto &rewr = context.rewr;
  auto &enumerator = context.enumerator;
  auto &summation_value_terms = context.summation_value_terms;
  auto &summation_values = context.summation_values;

  context.id_generator.clear();

  atermpp::unprotected_aterm_core tmp_vars(&summation_variables);
  const auto &variables = atermpp::down_cast<data::variable_list>(tmp_vars);

  atermpp::unprotected_aterm_core tmp_assign(&assignments);
  const auto &assignment_list =
      atermpp::down_cast<data::assignment_list>(tmp_assign);

  atermpp::unprotected_aterm_core tmp_multi_action(&multi_action);
  const auto &multi_action_template =
      atermpp::down_cast<lps::multi_action>(tmp_multi_action);

  atermpp::unprotected_aterm_core cond_term(&condition);
  data::data_expression rewritten_condition =
      rewr(atermpp::down_cast<data::data_expression>(cond_term), sigma);

  if (!data::is_false(rewritten_condition)) {
    enumerator.enumerate(
        enumerator_element(variables, rewritten_condition), sigma,
        [&](const enumerator_element &p) {
          p.add_assignments(variables, sigma, rewr);

          // Capture the summation-variable solution, protected in
          // summation_value_terms so it survives the term creation done while
          // building the rewritten multi-action below.
          summation_value_terms.clear();
          for (const data::variable &v : variables) {
            summation_value_terms.push_back(sigma(v));
          }
          summation_values.clear();
          for (const auto &value : summation_value_terms) {
            summation_values.push_back(atermpp::detail::address(value));
          }

          mcrl2_lps_emit_successor(
              context, assignment_list, multi_action_template,
              [&](rust::Slice<const atermpp::detail::_aterm *const> values,
                  const atermpp::detail::_aterm *rewritten_multi_action) {
                callback(callback_context, values, rewritten_multi_action,
                         {summation_values.data(), summation_values.size()});
              });
          return false;
        },
        data::is_false);
  }

  remove_assignments(sigma, variables);
}

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
inline void mcrl2_lps_compute_successor(
    learn_successors_context &context,
    const atermpp::detail::_aterm &summation_variables,
    rust::Slice<const atermpp::detail::_aterm *const> summation_values,
    const atermpp::detail::_aterm &assignments,
    const atermpp::detail::_aterm &multi_action,
    std::uint8_t *callback_context,
    rust::Fn<void(std::uint8_t *,
                  rust::Slice<const atermpp::detail::_aterm *const>,
                  const atermpp::detail::_aterm *)>
        callback) {
  auto &sigma = context.sigma;

  atermpp::unprotected_aterm_core tmp_vars(&summation_variables);
  const auto &variables = atermpp::down_cast<data::variable_list>(tmp_vars);

  // Assign the cached summation-variable solution into sigma.
  std::size_t i = 0;
  for (const data::variable &v : variables) {
    atermpp::unprotected_aterm_core value_term(summation_values[i]);
    sigma[v] = atermpp::down_cast<data::data_expression>(value_term);
    ++i;
  }

  atermpp::unprotected_aterm_core tmp_assign(&assignments);
  const auto &assignment_list =
      atermpp::down_cast<data::assignment_list>(tmp_assign);

  atermpp::unprotected_aterm_core tmp_multi_action(&multi_action);
  const auto &multi_action_template =
      atermpp::down_cast<lps::multi_action>(tmp_multi_action);

  mcrl2_lps_emit_successor(
      context, assignment_list, multi_action_template,
      [&](rust::Slice<const atermpp::detail::_aterm *const> values,
          const atermpp::detail::_aterm *rewritten_multi_action) {
        callback(callback_context, values, rewritten_multi_action);
      });

  remove_assignments(sigma, variables);
}

} // namespace mcrl2::lps

#endif // MCRL2_SYS_CPP_LPS_H