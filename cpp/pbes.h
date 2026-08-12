/// Wrapper around the PBES library of the mCRL2 toolset.
#ifndef MCRL2_SYS_CPP_PBES_H
#define MCRL2_SYS_CPP_PBES_H

#include "mcrl2/atermpp/aterm.h"
#include "mcrl2/data/data_specification.h"
#include "mcrl2/data/assignment.h"
#include "mcrl2/data/rewriter.h"
#include "mcrl2/data/substitution_utility.h"
#include "mcrl2/data/substitutions/mutable_indexed_substitution.h"
#include "mcrl2/pbes/detail/instantiate_global_variables.h"
#include "mcrl2/pbes/detail/stategraph_local_algorithm.h"
#include "mcrl2/pbes/detail/stategraph_pbes.h"
#include "mcrl2/pbes/io.h"
#include "mcrl2/pbes/pbes.h"
#include "mcrl2/pbes/propositional_variable.h"
#include "mcrl2/pbes/rewriters/enumerate_quantifiers_rewriter.h"
#include "mcrl2/pbes/rewriters/one_point_rule_rewriter.h"
#include "mcrl2/pbes/rewriters/simplify_quantifiers_rewriter.h"
#include "mcrl2/pbes/srf_pbes.h"
#include "mcrl2/pbes/transformations.h"
#include "mcrl2/pbes/unify_parameters.h"
#include "mcrl2/utilities/exception.h"

#include "mcrl2-sys/cpp/assert.h"
#include "mcrl2-sys/cpp/atermpp.h"
#include "mcrl2-sys/cpp/data.h" // for the mcrl2::data::assignment_pair declaration.
#include "rust/cxx.h"

#include <cstddef>
#include <mcrl2/atermpp/detail/aterm_core.h>
#include <memory>
#include <string>
#include <vector>

namespace mcrl2::pbes_system
{

/// Alias for templated type.
using srf_equation = detail::pre_srf_equation<false>;

/// Holds an enumerate_quantifiers_rewriter and a substitution used per-call.
///
/// Both `data::rewriter` and `enumerate_quantifiers_rewriter` store their own
/// copy of the data specification, so no member is needed to keep the
/// specification passed to the constructor alive.
struct pbes_rewrite_context
{
  data::rewriter           m_datar;
  enumerate_quantifiers_rewriter m_R;
  data::mutable_indexed_substitution<> m_sigma;
  /// The variables assigned by the last mcrl2_pbes_rewrite_set_assignments
  /// call, so that exactly those can be removed from m_sigma again.
  std::vector<data::variable> m_assigned;
  pbes_expression m_result; // keeps the rewritten term alive between FFI calls

  explicit pbes_rewrite_context(const data::data_specification& ds)
    : m_datar(ds), m_R(m_datar, ds)
  {}
};

inline
std::unique_ptr<pbes_rewrite_context>
mcrl2_pbes_create_rewrite_context(const data::data_specification& dataspec)
{
  return std::make_unique<pbes_rewrite_context>(dataspec);
}

inline
void mcrl2_pbes_rewrite_set_assignments(
    pbes_rewrite_context& ctx,
    rust::Slice<const atermpp::detail::_aterm* const> variables,
    rust::Slice<const atermpp::detail::_aterm* const> values)
{
  // rust::Slice::operator[] is only assert checked, and NDEBUG is defined for
  // release builds, so without this the loop below would read past `values`.
  if (variables.size() != values.size())
  {
    throw mcrl2::runtime_error("mcrl2_pbes_rewrite_set_assignments requires as many values as variables");
  }

  // Undo the previous assignment instead of calling m_sigma.clear().
  data::remove_assignments(ctx.m_sigma, ctx.m_assigned);
  ctx.m_assigned.clear();
  ctx.m_assigned.reserve(variables.size());

  for (std::size_t i = 0; i < variables.size(); ++i)
  {
    atermpp::unprotected_aterm_core tmp_var(variables[i]);
    atermpp::unprotected_aterm_core tmp_val(values[i]);
    const data::variable& variable = atermpp::down_cast<data::variable>(tmp_var);

    ctx.m_sigma[variable] = atermpp::down_cast<data::data_expression>(tmp_val);
    ctx.m_assigned.push_back(variable);
  }
}

/// Rewrites formula under ctx.m_sigma; result kept alive in ctx.m_result.
const atermpp::detail::_aterm*
mcrl2_pbes_rewrite_formula(
    pbes_rewrite_context& ctx,
    const atermpp::detail::_aterm& formula);

// Forward declaration
struct vertex_outgoing_edge;

// mcrl2::pbes_system::pbes

inline 
std::unique_ptr<pbes> mcrl2_load_pbes_from_pbes_file(rust::Str filename)
{
  pbes result;
  load_pbes(result, static_cast<std::string>(filename));
  return std::make_unique<pbes>(std::move(result));
}

inline 
std::unique_ptr<pbes> mcrl2_load_pbes_from_text_file(rust::Str filename)
{
  pbes result;
  load_pbes(result, static_cast<std::string>(filename), pbes_format_text());
  return std::make_unique<pbes>(std::move(result));
}

inline 
std::unique_ptr<pbes> mcrl2_load_pbes_from_text(rust::Str input)
{
  pbes result;
  std::istringstream stream(static_cast<std::string>(input));
  load_pbes(result, stream, pbes_format_text());
  return std::make_unique<pbes>(std::move(result));
}

inline
std::unique_ptr<data::data_specification> mcrl2_pbes_data_specification(const pbes& pbesspec)
{
  return std::make_unique<data::data_specification>(pbesspec.data());
}

inline
void mcrl2_pbes_normalize(pbes& pbesspec)
{
  algorithms::normalize(pbesspec);
}

inline
bool mcrl2_pbes_is_well_typed(const pbes& pbesspec)
{
  return pbesspec.is_well_typed();
}

inline
rust::String mcrl2_pbes_to_string(const pbes& pbesspec)
{
  std::stringstream ss;
  ss << pbesspec;
  return ss.str();
}

inline
std::unique_ptr<pbes> mcrl2_pbes_clone(const pbes& p)
{
  return std::make_unique<pbes>(p);
}

// mcrl2::pbes_system::pbes_equation

inline
void mcrl2_pbes_equations(std::vector<pbes_equation>& result, const pbes& pbesspec)
{
  // equations() is a std::vector, so the size is known up front and copying the
  // equations costs one allocation instead of log(N) reallocations that each
  // relocate every aterm member copied so far.
  result.reserve(result.size() + pbesspec.equations().size());
  for (const auto& eqn : pbesspec.equations())
  {
    result.push_back(eqn);
  }
}

inline
bool mcrl2_pbes_equation_is_mu(const pbes_equation& equation)
{
  return equation.symbol() == fixpoint_symbol::mu();
}

inline
const atermpp::detail::_aterm* mcrl2_pbes_equation_variable(const pbes_equation& equation)
{
  return atermpp::detail::address(equation.variable());
}

inline
const atermpp::detail::_aterm* mcrl2_pbes_equation_formula(const pbes_equation& equation)
{
  return atermpp::detail::address(equation.formula());
}

inline
rust::String mcrl2_pbes_expression_to_string(const atermpp::detail::_aterm& expr)
{
  atermpp::unprotected_aterm_core tmp(&expr);
  std::stringstream ss;
  ss << atermpp::down_cast<pbes_system::pbes_expression>(tmp);
  return ss.str();
}

class stategraph_algorithm : private detail::stategraph_local_algorithm
{
  // WARNING: This derives privately from the internal
  // `detail::stategraph_local_algorithm` so that it can reach into its
  // protected member and run only a subset of the algorithm).
  using super = detail::stategraph_local_algorithm;
public:

  stategraph_algorithm(const pbes& input)
      : super(input, pbesstategraph_options{.cache_marking_updates = true})
  {}

  void run()
  {
    // We only run the preprocessing of the base algorithm, explicitly skipping
    // execute_core() (belongs/extra graph/marking steps) and execute_postprocessing().
    execute_preprocessing();

    compute_local_control_flow_graphs();

    for (decltype(m_local_control_flow_graphs)::iterator i = m_local_control_flow_graphs.begin();
        i != m_local_control_flow_graphs.end();
        ++i)
    {
      mCRL2log(log::verbose) << "--- computed local control flow graph " << (i - m_local_control_flow_graphs.begin())
                             << " --- \n"
                             << *i << std::endl;
    }
  }

  const std::vector<detail::local_control_flow_graph>& local_control_flow_graphs() const
  {
    return m_local_control_flow_graphs;
  }

  const std::vector<detail::stategraph_equation>& equations() const
  {
    return m_pbes.equations();
  }
};

inline
std::unique_ptr<stategraph_algorithm> mcrl2_stategraph_local_algorithm_run(const pbes& p)
{
  auto algorithm = std::make_unique<stategraph_algorithm>(p);
  algorithm->run();
  return algorithm;
}

inline
std::size_t mcrl2_local_control_flow_graph_vertices(const detail::local_control_flow_graph& cfg)
{
  return cfg.vertices.size();
}

inline
const detail::local_control_flow_graph_vertex& mcrl2_local_control_flow_graph_vertex(const detail::local_control_flow_graph& cfg, std::size_t index)
{
  // NOTE: `cfg.vertices` is a node-based set without random access, so we have
  // to walk it to reach the `index`-th vertex. Advance a single iterator
  // (O(index)) rather than recomputing std::distance for every element, which
  // would make this O(index^2). Enumerating all vertices one by one from the
  // Rust side is still O(n^2) node hops; a bulk accessor would be needed to
  // avoid that.
  if (index >= cfg.vertices.size())
  {
    throw std::out_of_range("Index out of range in mcrl2_local_control_flow_graph_vertex");
  }

  return *std::next(cfg.vertices.begin(), static_cast<std::ptrdiff_t>(index));
}

// namespace mcrl2::pbes_system::detail::local_control_flow_graph_vertex

inline
std::size_t mcrl2_local_control_flow_graph_vertex_index(
    const detail::local_control_flow_graph_vertex& vertex)
{
  return vertex.index();
}

inline
const atermpp::detail::_aterm* mcrl2_local_control_flow_graph_vertex_name(
    const detail::local_control_flow_graph_vertex& vertex)
{
  return atermpp::detail::address(vertex.name());
}

inline
const atermpp::detail::_aterm* mcrl2_local_control_flow_graph_vertex_value(
    const detail::local_control_flow_graph_vertex& vertex)
{
  return atermpp::detail::address(vertex.value());
}

std::unique_ptr<std::vector<vertex_outgoing_edge>> mcrl2_local_control_flow_graph_vertex_outgoing_edges(const detail::local_control_flow_graph_vertex& vertex);

inline
std::size_t mcrl2_stategraph_local_algorithm_cfgs(const stategraph_algorithm& algorithm)
{
  return algorithm.local_control_flow_graphs().size();
}

inline
const detail::local_control_flow_graph& mcrl2_stategraph_local_algorithm_cfg(const stategraph_algorithm& algorithm, std::size_t index)
{
  return algorithm.local_control_flow_graphs().at(index);
}


inline
std::size_t mcrl2_stategraph_local_algorithm_equations(const stategraph_algorithm& algorithm)
{
  return algorithm.equations().size();
}


inline
const detail::stategraph_equation& mcrl2_stategraph_local_algorithm_equation(const stategraph_algorithm& algorithm, std::size_t index)
{
  return algorithm.equations().at(index);
}

inline
const atermpp::detail::_aterm* mcrl2_stategraph_equation_variable(const detail::stategraph_equation& equation)
{
  return atermpp::detail::address(equation.variable());
}

inline
std::unique_ptr<srf_pbes> mcrl2_pbes_to_srf_pbes(const pbes& p)
{
  return std::make_unique<srf_pbes>(pbes2srf(p));
}

inline
void mcrl2_srf_pbes_unify_parameters(srf_pbes& p, bool ignore_ce_equations, bool reset)
{
  unify_parameters(p, ignore_ce_equations, reset);
}

inline
void mcrl2_pbes_unify_parameters(pbes& p, bool ignore_ce_equations, bool reset)
{
  unify_parameters(p, ignore_ce_equations, reset);
}

// The individual steps that `pbesinst_lazy_algorithm::preprocess` applies to a
// PBES before instantiating it. They are exposed separately so that the caller
// composes (and reports) them itself; applying all four in the order below is
// what mCRL2 does.
//
// Each of the three body rewrites is a pure map over the equation bodies, so
// running them as three passes over all equations gives the same result as
// mCRL2's single pass that applies all three per equation.
//
// Note that `replace_constants_by_variables` is deliberately absent: mCRL2
// applies it in `pbesinst_lazy_algorithm::run` rather than in `preprocess`,
// because it lifts closed subterms into a substitution that the instantiating
// rewriter has to be primed with, so it cannot be applied to a PBES on its own.

/// Substitutes a value for every global variable of the PBES.
///
/// Throws when a global variable cannot be instantiated.
inline
void mcrl2_pbes_instantiate_global_variables(pbes& p)
{
  detail::instantiate_global_variables(p);
}

/// Simplifies every equation body, evaluating data subterms and eliminating
/// quantifiers that range over nothing.
inline
void mcrl2_pbes_simplify_quantifiers(pbes& p)
{
  data::rewriter datar(p.data());
  simplify_quantifiers_data_rewriter<data::rewriter> simplify(datar);

  for (pbes_equation& eqn : p.equations())
  {
    eqn.formula() = simplify(eqn.formula());
  }
}

/// Applies the one point rule to every equation body, replacing a quantifier
/// that pins its variable to a single value by that instance.
inline
void mcrl2_pbes_one_point_rule(pbes& p)
{
  one_point_rule_rewriter one_point;

  for (pbes_equation& eqn : p.equations())
  {
    eqn.formula() = one_point(eqn.formula());
  }
}

/// Orders the quantified variables of every equation body, so that quantifiers
/// differing only in the order of their variables become the same term.
inline
void mcrl2_pbes_order_quantified_variables(pbes& p)
{
  for (pbes_equation& eqn : p.equations())
  {
    eqn.formula() = order_quantified_variables(eqn.formula(), p.data());
  }
}

// mcrl2::pbes_system::detail::predicate_variable

inline
std::unique_ptr<std::vector<detail::predicate_variable>> mcrl2_stategraph_equation_predicate_variables(const detail::stategraph_equation& eqn)
{
  std::vector<detail::predicate_variable> result;
  result.reserve(eqn.predicate_variables().size());
  for (const auto& v : eqn.predicate_variables())
  {
    result.push_back(v);
  }
  return std::make_unique<std::vector<detail::predicate_variable>>(std::move(result));
}

inline
rust::Vec<std::size_t> mcrl2_predicate_variable_used(const detail::predicate_variable& v)
{
  rust::Vec<std::size_t> result;
  for (const auto& index : v.used())
  {
    result.push_back(index);
  }
  return result;
}

inline
rust::Vec<std::size_t> mcrl2_predicate_variable_changed(const detail::predicate_variable& v)
{
  rust::Vec<std::size_t> result;
  for (const auto& index : v.changed())
  {
    result.push_back(index);
  }
  return result;
}

inline
rust::Vec<std::size_t> mcrl2_predicate_variable_source(const detail::predicate_variable& v)
{
  rust::Vec<std::size_t> result;
  for (const auto& entry : v.source())
  {
    result.push_back(entry.first);
  }
  return result;
}

inline
rust::Vec<std::size_t> mcrl2_predicate_variable_target(const detail::predicate_variable& v)
{
  rust::Vec<std::size_t> result;
  for (const auto& entry : v.target())
  {
    result.push_back(entry.first);
  }
  return result;
}

inline
rust::Vec<std::size_t> mcrl2_predicate_variable_copy(const detail::predicate_variable& v)
{
  rust::Vec<std::size_t> result;
  for (const auto& entry : v.copy())
  {
    result.push_back(entry.first);
  }
  return result;
}

inline
const atermpp::detail::_aterm* mcrl2_predicate_variable_propositional_variable_instantiation(const detail::predicate_variable& v)
{
  return atermpp::detail::address(v.variable());
}

// mcrl2::pbes_system::srf_pbes

inline
std::unique_ptr<pbes> mcrl2_srf_pbes_to_pbes(const srf_pbes& p)
{
  return std::make_unique<pbes>(p.to_pbes());
}

// mcrl2::pbes_system::srf_equation

inline
void mcrl2_srf_pbes_equations(std::vector<srf_equation>& result, const srf_pbes& p)
{
  result.reserve(result.size() + p.equations().size());
  for (const auto& eqn : p.equations())
  {
    result.push_back(eqn);
  }
}

inline
const atermpp::detail::_aterm* mcrl2_srf_pbes_equation_variable(const srf_equation& equation)
{
  return atermpp::detail::address(equation.variable());
}

// mcrl2::pbes_system::propositional_variable

inline
bool mcrl2_pbes_is_propositional_variable(const atermpp::detail::_aterm& variable)
{
  atermpp::unprotected_aterm_core tmp(&variable);
  return pbes_system::is_propositional_variable(atermpp::down_cast<atermpp::aterm>(tmp));
}

inline
rust::String mcrl2_propositional_variable_to_string(const atermpp::detail::_aterm& variable)
{
  atermpp::unprotected_aterm_core tmp(&variable);
  MCRL2_ASSERT(pbes_system::is_propositional_variable(atermpp::down_cast<atermpp::aterm>(tmp)));
  std::stringstream ss;
  ss << atermpp::down_cast<propositional_variable>(tmp);
  return ss.str();
}

inline
void mcrl2_srf_equations_summands(std::vector<srf_summand>& result, const srf_equation& equation)
{
  result.reserve(result.size() + equation.summands().size());
  for (const auto& summand : equation.summands())
  {
    result.push_back(summand);
  }
}

inline
const atermpp::detail::_aterm* mcrl2_srf_summand_variable(const srf_summand& summand)
{
  return atermpp::detail::address(summand.variable());
}

inline
const atermpp::detail::_aterm* mcrl2_srf_summand_condition(const srf_summand& summand)
{
  return atermpp::detail::address(summand.condition());
}

inline
const atermpp::detail::_aterm* mcrl2_srf_summand_parameters(const srf_summand& summand)
{
  return atermpp::detail::address(summand.parameters());
}

inline
bool mcrl2_srf_equation_is_conjunctive(const srf_equation& equation)
{
  return equation.is_conjunctive();
}

inline
bool mcrl2_srf_equation_is_mu(const srf_equation& equation)
{
  return equation.symbol() == fixpoint_symbol::mu();
}

inline
const atermpp::detail::_aterm* mcrl2_pbes_initial_state(const pbes& p)
{
  return atermpp::detail::address(p.initial_state());
}

inline
const atermpp::detail::_aterm* mcrl2_srf_pbes_initial_state(const srf_pbes& p)
{
  return atermpp::detail::address(p.initial_state());
}

/// Build a data::assignment_list from two parallel term lists (variables and values).
/// The resulting term can be passed directly to mcrl2_lps_enumerate as the assignments argument.
inline
std::unique_ptr<atermpp::aterm> mcrl2_make_data_assignment_list(
    const atermpp::detail::_aterm& variables_term,
    const atermpp::detail::_aterm& values_term)
{
  atermpp::unprotected_aterm_core vars_tmp(&variables_term);
  atermpp::unprotected_aterm_core vals_tmp(&values_term);
  const auto& vars = atermpp::down_cast<data::variable_list>(vars_tmp);
  const auto& vals = atermpp::down_cast<data::data_expression_list>(vals_tmp);

  return std::make_unique<atermpp::aterm>(data::make_assignment_list(vars, vals));
}

/// Note that the substitution uses mcrl2::data::assignment_pair, which is
/// declared by the data bridge (src/data.rs); this function is declared there
/// too so that both bridges share a single Rust type.
std::unique_ptr<atermpp::aterm> mcrl2_pbes_expression_replace_variables(const atermpp::detail::_aterm& expr, const rust::Vec<data::assignment_pair>& sigma);

std::unique_ptr<atermpp::aterm> mcrl2_pbes_expression_replace_propositional_variables(const atermpp::detail::_aterm& expr, const rust::Vec<std::size_t>& pi);

/// mcrl2::pbes_system::pbes_expression

inline
bool mcrl2_pbes_is_pbes_expression(const atermpp::detail::_aterm& variable)
{
  atermpp::unprotected_aterm_core tmp(&variable);
  return pbes_system::is_pbes_expression(atermpp::down_cast<atermpp::aterm>(tmp));
}

inline
bool mcrl2_pbes_is_propositional_variable_instantiation(const atermpp::detail::_aterm& variable)
{
  atermpp::unprotected_aterm_core tmp(&variable);
  return pbes_system::is_propositional_variable_instantiation(atermpp::down_cast<atermpp::aterm>(tmp));
}

inline
bool mcrl2_pbes_is_not(const atermpp::detail::_aterm& variable)
{
  atermpp::unprotected_aterm_core tmp(&variable);
  return pbes_system::is_not(atermpp::down_cast<atermpp::aterm>(tmp));
}

inline
bool mcrl2_pbes_is_and(const atermpp::detail::_aterm& variable)
{
  atermpp::unprotected_aterm_core tmp(&variable);
  return pbes_system::is_and(atermpp::down_cast<atermpp::aterm>(tmp));
}

inline
bool mcrl2_pbes_is_or(const atermpp::detail::_aterm& variable)
{
  atermpp::unprotected_aterm_core tmp(&variable);
  return pbes_system::is_or(atermpp::down_cast<atermpp::aterm>(tmp));
}

inline
bool mcrl2_pbes_is_imp(const atermpp::detail::_aterm& variable)
{
  atermpp::unprotected_aterm_core tmp(&variable);
  return pbes_system::is_imp(atermpp::down_cast<atermpp::aterm>(tmp));
}

inline
bool mcrl2_pbes_is_forall(const atermpp::detail::_aterm& variable)
{
  atermpp::unprotected_aterm_core tmp(&variable);
  return pbes_system::is_forall(atermpp::down_cast<atermpp::aterm>(tmp));
}

inline
bool mcrl2_pbes_is_exists(const atermpp::detail::_aterm& variable)
{
  atermpp::unprotected_aterm_core tmp(&variable);
  return pbes_system::is_exists(atermpp::down_cast<atermpp::aterm>(tmp));
}

inline
bool mcrl2_pbes_is_true(const atermpp::detail::_aterm& variable)
{
  atermpp::unprotected_aterm_core tmp(&variable);
  return pbes_system::is_true(atermpp::down_cast<pbes_expression>(tmp));
}

inline
bool mcrl2_pbes_is_false(const atermpp::detail::_aterm& variable)
{
  atermpp::unprotected_aterm_core tmp(&variable);
  return pbes_system::is_false(atermpp::down_cast<pbes_expression>(tmp));
}

} // namespace mcrl2::pbes_system

#endif // MCRL2_SYS_CPP_PBES_H