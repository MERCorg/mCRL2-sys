#include "atermpp.h"
#include "mcrl2/data/substitutions/mutable_map_substitution.h"
#include "mcrl2/data/replace.h"

#include "mcrl2-sys/cpp/data.h"
#include "mcrl2-sys/src/data.rs.h"

namespace mcrl2::data
{

std::unique_ptr<atermpp::aterm> mcrl2_data_expression_replace_variables(const atermpp::detail::_aterm& term,
    const rust::Vec<assignment_pair>& sigma)
{
  atermpp::unprotected_aterm_core tmp_expr(&term);
  MCRL2_ASSERT(is_data_expression(atermpp::down_cast<atermpp::aterm>(tmp_expr)));

  data::mutable_map_substitution<> tmp;
  for (const auto& assign : sigma)
  {
    atermpp::unprotected_aterm_core tmp_lhs(assign.lhs);
    atermpp::unprotected_aterm_core tmp_rhs(assign.rhs);

    tmp[atermpp::down_cast<data::variable>(tmp_lhs)]
        = atermpp::down_cast<data::data_expression>(tmp_rhs);
  }

  return std::make_unique<atermpp::aterm>(
      replace_variables(atermpp::down_cast<data_expression>(tmp_expr), tmp));
}

std::unique_ptr<atermpp::aterm> mcrl2_data_parse_variables(rust::Str text, const data_specification& spec)
{
  std::vector<variable> result;
  parse_variables(std::string(text), std::back_inserter(result), spec);
  // Unlike mcrl2_data_specification_components's aterm_list construction,
  // variable_list's iterator-range constructor conses front-to-back without
  // a backward optimisation, so it reverses a forward range; iterate the
  // vector backward here to hand callers back the declared order.
  return std::make_unique<atermpp::aterm>(variable_list(result.rbegin(), result.rend()));
}

std::unique_ptr<atermpp::aterm> mcrl2_data_parse_data_expression(
    rust::Str text, const atermpp::detail::_aterm& variables, const data_specification& spec)
{
  atermpp::unprotected_aterm_core tmp_variables(&variables);
  const variable_list& var_list = atermpp::down_cast<variable_list>(tmp_variables);

  data_expression result = parse_data_expression(std::string(text), var_list, spec);
  return std::make_unique<atermpp::aterm>(result);
}

}