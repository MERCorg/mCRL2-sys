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
std::unique_ptr<stochastic_specification> mcrl2_load_lps_from_lps_file(rust::Str filename)
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

} // namespace mcrl2::lps

#endif // MCRL2_SYS_CPP_LPS_H