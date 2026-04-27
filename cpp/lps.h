/// Wrapper around the lps library of the mCRL2 toolset.

#ifndef MCRL2_SYS_CPP_LPS_H
#define MCRL2_SYS_CPP_LPS_H

#include "mcrl2/lps/io.h"
#include "mcrl2/lps/detail/instantiate_global_variables.h"
#include "mcrl2/lps/detail/replace_global_variables.h"
#include "mcrl2/lps/lpsreach.h"
#include "mcrl2/lps/one_point_rule_rewrite.h"
#include "mcrl2/lps/order_summand_variables.h"
#include "mcrl2/lps/replace_constants_by_variables.h"
#include "mcrl2/lps/resolve_name_clashes.h"
#include "mcrl2/lps/stochastic_specification.h"

#include "rust/cxx.h"

namespace mcrl2::lps
{

inline 
std::unique_ptr<stochastic_specification> mcrl2_lps_load_from_lps_file(rust::Str filename)
{
  stochastic_specification result;
  load_lps(result, static_cast<std::string>(filename));
  return std::make_unique<stochastic_specification>(result);
}

inline 
std::unique_ptr<stochastic_specification> mcrl2_lps_preprocess_symbolic_exploration(const stochastic_specification& lpsspec)
{
    stochastic_specification result = lpsspec;
    lps::detail::replace_global_variables(result);
    lps::detail::instantiate_global_variables(result);
    lps::order_summand_variables(result);
    resolve_summand_variable_name_clashes(result); // N.B. This is a required preprocessing step.
    one_point_rule_rewrite(result);
    // replace_constants_by_variables(result, m_rewr, m_sigma);

    return std::make_unique<stochastic_specification>(result);
}

inline
std::size_t mcrl2_lps_num_of_action_summands(const stochastic_specification& spec)
{
  return spec.process().summand_count();
}

inline
std::unique_ptr<stochastic_action_summand> mcrl2_lps_action_summand(const stochastic_specification& spec, std::size_t index)
{
  return std::make_unique<stochastic_action_summand>(spec.process().action_summands()[index]);
}

inline
std::unique_ptr<stochastic_process_initializer> mcrl2_lps_process_initializer(const stochastic_specification& spec)
{
  return std::make_unique<stochastic_process_initializer>(spec.initial_process());
}

inline
const atermpp::detail::_aterm* mcrl2_lps_action_summand_condition(const stochastic_action_summand& summand)
{
  return atermpp::detail::address(summand.condition());
}

} // namespace mcrl2::lps

#endif // MCRL2_SYS_CPP_LPS_H