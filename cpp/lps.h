/// Wrapper around the lps library of the mCRL2 toolset.

#ifndef MCRL2_SYS_CPP_LPS_H
#define MCRL2_SYS_CPP_LPS_H

#include "mcrl2/data/consistency.h"
#include "mcrl2/lps/detail/instantiate_global_variables.h"
#include "mcrl2/lps/detail/replace_global_variables.h"
#include "mcrl2/lps/io.h"
#include "mcrl2/lps/lpsreach.h"
#include "mcrl2/lps/one_point_rule_rewrite.h"
#include "mcrl2/lps/order_summand_variables.h"
#include "mcrl2/lps/replace_constants_by_variables.h"
#include "mcrl2/lps/resolve_name_clashes.h"
#include "mcrl2/lps/stochastic_specification.h"

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
  // Temporary vector used for the returned values.
  std::vector<const atermpp::detail::_aterm *> values;

  learn_successors_context(const data::data_specification &dataspec)
      : rewr(dataspec, data::used_data_equation_selector(dataspec)),
        enumerator(rewr, dataspec, rewr, id_generator, false) {}
};

inline std::unique_ptr<learn_successors_context>
mcrl2_lps_create_learn_successors_context(
    const stochastic_specification &spec) {
  return std::make_unique<learn_successors_context>(spec.data());
}

/// \brief Enumerate all solutions for the summand variables that satisfy the
/// condition,
///        given the read parameter assignments from the current state.
///
/// For each solution found, calls \p callback with \p callback_context and a
/// slice of the rewritten next-state values (one per assignment in the
/// summand).
inline void mcrl2_lps_enumerate(
    learn_successors_context &context, const atermpp::detail::_aterm &condition,
    const atermpp::detail::_aterm &summation_variables,
    const atermpp::detail::_aterm &assignments,
    rust::Slice<const atermpp::detail::_aterm *const> read_parameters,
    rust::Slice<const atermpp::detail::_aterm *const> read_values,
    std::uint8_t *callback_context,
    rust::Fn<void(std::uint8_t *,
                  rust::Slice<const atermpp::detail::_aterm *const>)>
        callback) {
  using enumerator_element = data::enumerator_list_element_with_substitution<>;
  auto &sigma = context.sigma;
  auto &rewr = context.rewr;
  auto &enumerator = context.enumerator;
  auto &values = context.values;

  // Interpret the summation variables as a variable_list.
  atermpp::unprotected_aterm_core tmp_vars(&summation_variables);
  const auto &variables = atermpp::down_cast<data::variable_list>(tmp_vars);

  // Interpret the assignments as an assignment_list.
  atermpp::unprotected_aterm_core tmp_assign(&assignments);
  const auto &assignment_list =
      atermpp::down_cast<data::assignment_list>(tmp_assign);

  // Assign the read parameter values to sigma.
  for (std::size_t j = 0; j < read_parameters.size(); j++) {
    atermpp::unprotected_aterm_core param_term(read_parameters[j]);
    atermpp::unprotected_aterm_core value_term(read_values[j]);
    sigma[atermpp::down_cast<data::variable>(param_term)] =
        atermpp::down_cast<data::data_expression>(value_term);
  }

  // Rewrite the condition under the current substitution.
  atermpp::unprotected_aterm_core cond_term(&condition);
  data::data_expression rewritten_condition =
      rewr(atermpp::down_cast<data::data_expression>(cond_term), sigma);

  if (!data::is_false(rewritten_condition)) {
    enumerator.enumerate(
        enumerator_element(variables, rewritten_condition), sigma,
        [&](const enumerator_element &p) {
          p.add_assignments(variables, sigma, rewr);

          // Rewrite the right-hand sides of the assignments and add them to the
          // returned values.
          values.clear();
          for (const auto &assignment : assignment_list) {
            data::data_expression value = rewr(assignment.rhs(), sigma);
            values.push_back(atermpp::detail::address(value));
          }

          callback(callback_context, {values.data(), values.size()});
          return false;
        },
        data::is_false);
  }

  // Clean up sigma.
  sigma.clear();
}

} // namespace mcrl2::lps

#endif // MCRL2_SYS_CPP_LPS_H