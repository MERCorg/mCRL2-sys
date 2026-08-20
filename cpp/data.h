/// Wrapper around the data library of the mCRL2 toolset.

#ifndef MCRL2_SYS_CPP_DATA_H
#define MCRL2_SYS_CPP_DATA_H

#include "mcrl2/atermpp/aterm.h"
#include "mcrl2/atermpp/aterm_string.h"
#include "mcrl2/data/data_expression.h"
#include "mcrl2/data/data_specification.h"
#include "mcrl2/data/detail/io.h"
#include "mcrl2/data/detail/rewrite/jitty.h"
#include "mcrl2/data/parse.h"
#include "mcrl2/data/sort_expression.h"
#include "mcrl2/data/variable.h"

#ifdef MCRL2_ENABLE_JITTYC
#include "mcrl2/data/detail/rewrite/jittyc.h"
#endif // MCRL2_ENABLE_JITTYC

#include "mcrl2-sys/cpp/assert.h"
#include "mcrl2-sys/cpp/atermpp.h"

#include "rust/cxx.h"

namespace mcrl2::data
{

// Forward declaration
struct assignment_pair;

inline
std::unique_ptr<data_specification> mcrl2_data_specification_from_string(rust::Str input)
{
  return std::make_unique<data_specification>(parse_data_specification(std::string(input)));
}

inline
std::unique_ptr<detail::RewriterJitty> mcrl2_create_rewriter_jitty(const data::data_specification& specification)
{
  return std::make_unique<detail::RewriterJitty>(specification, used_data_equation_selector(specification));
}

#ifdef MCRL2_ENABLE_JITTYC

inline
std::unique_ptr<detail::RewriterCompilingJitty> mcrl2_create_rewriter_jittyc(const data::data_specification& specification)
{
  return std::make_unique<detail::RewriterCompilingJitty>(specification, used_data_equation_selector(specification));
}

#endif

std::unique_ptr<atermpp::aterm> mcrl2_data_expression_replace_variables(const atermpp::detail::_aterm& term,
    const rust::Vec<assignment_pair>& sigma);

inline
bool mcrl2_data_expression_is_variable(const atermpp::detail::_aterm& input)
{
  atermpp::unprotected_aterm_core tmp(&input);
  return data::is_variable(atermpp::down_cast<atermpp::aterm>(tmp));
}

inline
bool mcrl2_data_expression_is_application(const atermpp::detail::_aterm& input)
{
  atermpp::unprotected_aterm_core tmp(&input);
  return data::is_application(atermpp::down_cast<atermpp::aterm>(tmp));
}

inline
bool mcrl2_data_expression_is_abstraction(const atermpp::detail::_aterm& input)
{
  atermpp::unprotected_aterm_core tmp(&input);
  return data::is_abstraction(atermpp::down_cast<atermpp::aterm>(tmp));
}

inline
bool mcrl2_data_expression_is_binder_exists(const atermpp::detail::_aterm& input)
{
  atermpp::unprotected_aterm_core tmp(&input);
  return data::is_exists_binder(atermpp::down_cast<atermpp::aterm>(tmp));
}

inline
bool mcrl2_data_expression_is_binder_forall(const atermpp::detail::_aterm& input)
{
  atermpp::unprotected_aterm_core tmp(&input);
  return data::is_forall_binder(atermpp::down_cast<atermpp::aterm>(tmp));
}

inline
bool mcrl2_data_expression_is_binder_lambda(const atermpp::detail::_aterm& input)
{
  atermpp::unprotected_aterm_core tmp(&input);
  return data::is_lambda_binder(atermpp::down_cast<atermpp::aterm>(tmp));
}

inline
bool mcrl2_data_expression_is_binder_set_comp(const atermpp::detail::_aterm& input)
{
  atermpp::unprotected_aterm_core tmp(&input);
  return data::is_set_comprehension_binder(atermpp::down_cast<atermpp::aterm>(tmp));
}

inline
bool mcrl2_data_expression_is_binder_bag_comp(const atermpp::detail::_aterm& input)
{
  atermpp::unprotected_aterm_core tmp(&input);
  return data::is_bag_comprehension_binder(atermpp::down_cast<atermpp::aterm>(tmp));
}

inline
bool mcrl2_data_expression_is_binder_untyped_set_bag_comp(const atermpp::detail::_aterm& input)
{
  atermpp::unprotected_aterm_core tmp(&input);
  return data::is_untyped_set_or_bag_comprehension_binder(atermpp::down_cast<atermpp::aterm>(tmp));
}

inline
bool mcrl2_data_expression_is_function_symbol(const atermpp::detail::_aterm& input)
{
  atermpp::unprotected_aterm_core tmp(&input);
  return data::is_function_symbol(atermpp::down_cast<atermpp::aterm>(tmp));
}

inline
bool mcrl2_data_expression_is_where_clause(const atermpp::detail::_aterm& input)
{
  atermpp::unprotected_aterm_core tmp(&input);
  return data::is_where_clause(atermpp::down_cast<atermpp::aterm>(tmp));
}

inline
bool mcrl2_data_expression_is_machine_number(const atermpp::detail::_aterm& input)
{
  atermpp::unprotected_aterm_core tmp(&input);
  return data::is_machine_number(atermpp::down_cast<atermpp::aterm>(tmp));
}

inline
bool mcrl2_data_expression_is_untyped_identifier(const atermpp::detail::_aterm& input)
{
  atermpp::unprotected_aterm_core tmp(&input);
  return data::is_untyped_identifier(atermpp::down_cast<atermpp::aterm>(tmp));
}

inline
bool mcrl2_data_expression_is_data_expression(const atermpp::detail::_aterm& input)
{
  atermpp::unprotected_aterm_core tmp(&input);
  return data::is_data_expression(atermpp::down_cast<atermpp::aterm>(tmp));
}

/// Recursively strips the internal rewriter-only `OpId(name, sort, index)` markers from \p input.
inline
std::unique_ptr<atermpp::aterm> mcrl2_data_expression_remove_index(const atermpp::detail::_aterm& input)
{
  atermpp::unprotected_aterm_core tmp(&input);
  return std::make_unique<atermpp::aterm>(detail::remove_index(atermpp::down_cast<atermpp::aterm>(tmp)));
}

inline
bool mcrl2_is_data_sort_expression(const atermpp::detail::_aterm& input)
{
  atermpp::unprotected_aterm_core tmp(&input);
  return data::is_sort_expression(atermpp::down_cast<atermpp::aterm>(tmp));
}

inline
rust::String mcrl2_data_expression_to_string(const atermpp::detail::_aterm& input)
{
  atermpp::unprotected_aterm_core tmp(&input);
  return data::pp(atermpp::down_cast<data::data_expression>(tmp));
}

inline
rust::String mcrl2_sort_expression_to_string(const atermpp::detail::_aterm& input)
{
  atermpp::unprotected_aterm_core tmp(&input);
  return data::pp(atermpp::down_cast<data::sort_expression>(tmp));
}

/// Returns \p components as an aterm_list.
///
/// With \p strip_index set, the internal rewriter-only `OpId(name, sort, index)`
/// function symbols are rewritten into the `OpIdNoIndex(name, sort)` form that
/// mCRL2 uses in its serialised (binary aterm) representation, so callers
/// observe exactly the terms that mCRL2 would write to disk. mCRL2 applies that
/// transformation to a whole data specification while writing it; here it is
/// applied per list, and only to the three lists that can contain function
/// symbols. Sorts and aliases consist of sort expressions alone, so stripping
/// would be a traversal without effect there.
template <typename Container>
inline
std::unique_ptr<atermpp::aterm> mcrl2_data_specification_components(const Container& components, bool strip_index)
{
  // These containers are std::vectors, so this selects the linear
  // make_list_backward overload rather than reversing an input range.
  atermpp::aterm_list list(components.begin(), components.end());
  return std::make_unique<atermpp::aterm>(strip_index ? detail::remove_index(list) : atermpp::aterm(list));
}

/// Returns the user-defined sorts of the data specification as an aterm_list.
inline
std::unique_ptr<atermpp::aterm> mcrl2_data_specification_user_defined_sorts(const data_specification& spec)
{
  return mcrl2_data_specification_components(spec.user_defined_sorts(), /* strip_index = */ false);
}

/// Returns the user-defined aliases of the data specification as an aterm_list.
inline
std::unique_ptr<atermpp::aterm> mcrl2_data_specification_user_defined_aliases(const data_specification& spec)
{
  return mcrl2_data_specification_components(spec.user_defined_aliases(), /* strip_index = */ false);
}

/// Returns the user-defined constructors of the data specification as an
/// aterm_list, in the serialised `OpIdNoIndex` form.
inline
std::unique_ptr<atermpp::aterm> mcrl2_data_specification_user_defined_constructors(const data_specification& spec)
{
  return mcrl2_data_specification_components(spec.user_defined_constructors(), /* strip_index = */ true);
}

/// Returns the user-defined mappings of the data specification as an
/// aterm_list, in the serialised `OpIdNoIndex` form.
inline
std::unique_ptr<atermpp::aterm> mcrl2_data_specification_user_defined_mappings(const data_specification& spec)
{
  return mcrl2_data_specification_components(spec.user_defined_mappings(), /* strip_index = */ true);
}

/// Returns the user-defined equations of the data specification as an
/// aterm_list, in the serialised `OpIdNoIndex` form.
inline
std::unique_ptr<atermpp::aterm> mcrl2_data_specification_user_defined_equations(const data_specification& spec)
{
  return mcrl2_data_specification_components(spec.user_defined_equations(), /* strip_index = */ true);
}

} // namespace mcrl2::data

#endif // MCRL2_SYS_CPP_DATA_H