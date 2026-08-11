#include "atermpp.h"
#include "mcrl2-sys/cpp/pbes.h"
#include "mcrl2-sys/src/data.rs.h" // for mcrl2::data::assignment_pair.
#include "mcrl2-sys/src/pbes.rs.h"

#include <cstddef>
#include <optional>

namespace mcrl2::pbes_system
{

std::unique_ptr<std::vector<vertex_outgoing_edge>> mcrl2_local_control_flow_graph_vertex_outgoing_edges(const detail::local_control_flow_graph_vertex& vertex)
{
  // The edges are stored in node based containers that cxx cannot expose
  // directly, so they are copied into vectors here. Reserve up front such that
  // this costs one allocation per (inner) vector.
  std::vector<vertex_outgoing_edge> result;
  result.reserve(vertex.outgoing_edges().size());
  for (const auto& edge : vertex.outgoing_edges())
  {
    vertex_outgoing_edge voe;
    voe.vertex = edge.first;
    voe.edges = std::make_unique<std::vector<std::size_t>>();
    voe.edges->reserve(edge.second.size());
    for (const auto& e : edge.second)
    {
      voe.edges->emplace_back(e);
    }
    result.emplace_back(std::move(voe));
  }
  return std::make_unique<std::vector<vertex_outgoing_edge>>(std::move(result));
}

std::unique_ptr<atermpp::aterm> mcrl2_pbes_expression_replace_variables(const atermpp::detail::_aterm& term,
    const rust::Vec<data::assignment_pair>& sigma)
{
  atermpp::unprotected_aterm_core tmp_expr(&term);
  MCRL2_ASSERT(is_pbes_expression(atermpp::down_cast<atermpp::aterm>(tmp_expr)));

  data::mutable_map_substitution<> tmp;
  for (const auto& assign : sigma)
  {
    atermpp::unprotected_aterm_core tmp_lhs(assign.lhs);
    atermpp::unprotected_aterm_core tmp_rhs(assign.rhs);

    tmp[atermpp::down_cast<data::variable>(tmp_lhs)]
        = atermpp::down_cast<data::data_expression>(tmp_rhs);
  }

  return std::make_unique<atermpp::aterm>(
      pbes_system::replace_variables(atermpp::down_cast<pbes_expression>(tmp_expr), tmp));
}

std::unique_ptr<atermpp::aterm> mcrl2_pbes_expression_replace_propositional_variables(const atermpp::detail::_aterm& term,
    const rust::Vec<std::size_t>& pi)
{
  atermpp::unprotected_aterm_core tmp_expr(&term);
  MCRL2_ASSERT(is_pbes_expression(atermpp::down_cast<atermpp::aterm>(tmp_expr)));

  // pi must be a permutation of the parameter positions: every source position
  // maps to a distinct valid target position. Otherwise the write into
  // new_parameters[pi[i]] below would go out of bounds, or leave some entries
  // default constructed (an unassigned term) because a target position was
  // written twice. This is a property of pi alone, so it is established once
  // here rather than re-derived for every propositional variable occurrence the
  // traversal reaches, and a violation is reported before any term is rebuilt.
  std::vector<bool> assigned(pi.size(), false);
  for (const std::size_t target : pi)
  {
    if (target >= pi.size())
    {
      throw mcrl2::runtime_error("Index out of bounds in replace_propositional_variables");
    }

    if (assigned[target])
    {
      throw mcrl2::runtime_error("Duplicate target index in replace_propositional_variables");
    }
    assigned[target] = true;
  }

  pbes_expression result;
  pbes_system::replace_propositional_variables(result,
      atermpp::down_cast<pbes_expression>(tmp_expr),
      [&pi](const propositional_variable_instantiation& v) -> pbes_expression
      {
        // After unify_parameters all propositional variables share one arity, so
        // a mismatch means pi does not describe this PBES.
        if (v.parameters().size() != pi.size())
        {
          throw mcrl2::runtime_error("Permutation does not match the number of parameters in replace_propositional_variables");
        }

        // The parameters are a term list, so walk it once instead of using
        // std::next per position, which would be quadratic.
        std::vector<data::data_expression> new_parameters(pi.size());
        std::size_t i = 0;
        for (const data::data_expression& parameter : v.parameters())
        {
          new_parameters[pi[i]] = parameter;
          ++i;
        }

        return propositional_variable_instantiation(v.name(), data::data_expression_list(new_parameters));
      });
  return std::make_unique<atermpp::aterm>(result);
}

const atermpp::detail::_aterm* mcrl2_pbes_rewrite_formula(
    pbes_rewrite_context& ctx,
    const atermpp::detail::_aterm& formula)
{
  atermpp::unprotected_aterm_core tmp(&formula);

  // The rewriter mints fresh variable names for every quantifier enumeration
  // from a generator whose index only ever increases.
  ctx.m_R.clear_identifier_generator();

  try
  {
    ctx.m_R(ctx.m_result,
            atermpp::down_cast<pbes_expression>(tmp),
            ctx.m_sigma);
  }
  catch (...)
  {
    // While enumerating quantifiers the rewriter binds the quantified variables
    // in sigma and unbinds them again afterwards.
    ctx.m_sigma.clear();
    ctx.m_assigned.clear();
    throw;
  }

  return atermpp::detail::address(ctx.m_result);
}

} // namespace mcrl2::pbes_system