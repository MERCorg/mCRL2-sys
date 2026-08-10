/// Wrapper around the data library of the mCRL2 toolset.

#ifndef MCRL2_SYS_CPP_DATA_H
#define MCRL2_SYS_CPP_DATA_H

#include "mcrl2/atermpp/algorithm.h"
#include "mcrl2/atermpp/aterm.h"
#include "mcrl2/atermpp/aterm_string.h"
#include "mcrl2/atermpp/algorithm.h"
#include "mcrl2/core/detail/function_symbols.h"
#include "mcrl2/data/data_expression.h"
#include "mcrl2/data/data_specification.h"
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

/// Rewrites a single `OpId(name, sort, index)` into `OpIdNoIndex(name, sort)`.
///
/// This mirrors `remove_index_impl` in mCRL2's `data/source/data_io.cpp`, which
/// is `static` there and so cannot be reused. The index is a process-local
/// lookup number rather than part of the term's identity, so it must be dropped
/// before terms can be compared against an implementation that never assigns one.
inline atermpp::aterm mcrl2_remove_op_id_index(const atermpp::aterm& term)
{
  if (term.function() == core::detail::function_symbol_OpId())
  {
    return atermpp::aterm(core::detail::function_symbol_OpIdNoIndex(), term.begin(), --term.end());
  }

  return term;
}

/// Wraps a vector of aterm-derived declarations into an owned aterm_list.
///
/// The `user_defined_*` accessors below all return a `std::vector` of some
/// aterm subclass. cxx cannot express those vectors, so each is converted into
/// a maximally shared `aterm_list` and handed to Rust as a single term. The
/// list is returned in the serialised (index-stripped) form, which is the shape
/// terms have when they are written out, and the only shape that is comparable
/// across implementations.
template <typename Container>
inline std::unique_ptr<atermpp::aterm> mcrl2_declarations_to_aterm_list(const Container& declarations)
{
  atermpp::aterm_list list(declarations.begin(), declarations.end());
  return std::make_unique<atermpp::aterm>(atermpp::bottom_up_replace(list, mcrl2_remove_op_id_index));
}

inline
std::unique_ptr<atermpp::aterm> mcrl2_data_specification_user_defined_sorts(const data_specification& specification)
{
  return mcrl2_declarations_to_aterm_list(specification.user_defined_sorts());
}

inline
std::unique_ptr<atermpp::aterm> mcrl2_data_specification_user_defined_aliases(const data_specification& specification)
{
  return mcrl2_declarations_to_aterm_list(specification.user_defined_aliases());
}

inline
std::unique_ptr<atermpp::aterm> mcrl2_data_specification_user_defined_constructors(const data_specification& specification)
{
  return mcrl2_declarations_to_aterm_list(specification.user_defined_constructors());
}

inline
std::unique_ptr<atermpp::aterm> mcrl2_data_specification_user_defined_mappings(const data_specification& specification)
{
  return mcrl2_declarations_to_aterm_list(specification.user_defined_mappings());
}

inline
std::unique_ptr<atermpp::aterm> mcrl2_data_specification_user_defined_equations(const data_specification& specification)
{
  return mcrl2_declarations_to_aterm_list(specification.user_defined_equations());
}

/// Returns the user-defined sorts of the data specification as an aterm_list.
inline
std::unique_ptr<atermpp::aterm> mcrl2_data_specification_user_defined_sorts(const data_specification& spec)
{
  return std::make_unique<atermpp::aterm>(
      atermpp::aterm_list(spec.user_defined_sorts().begin(), spec.user_defined_sorts().end()));
}

/// Returns the user-defined aliases of the data specification as an aterm_list.
inline
std::unique_ptr<atermpp::aterm> mcrl2_data_specification_user_defined_aliases(const data_specification& spec)
{
  return std::make_unique<atermpp::aterm>(
      atermpp::aterm_list(spec.user_defined_aliases().begin(), spec.user_defined_aliases().end()));
}

/// Rewrites the internal, rewriter-only `OpId(name, sort, index)` function
/// symbols into the `OpIdNoIndex(name, sort)` form that mCRL2 uses in its
/// serialised (binary aterm) representation. This mirrors `remove_index_impl`
/// in `libraries/data/source/data_io.cpp`, so callers observe exactly the
/// terms that mCRL2 would write to disk.
inline
atermpp::aterm mcrl2_remove_function_symbol_index(const atermpp::aterm& x)
{
  return atermpp::bottom_up_replace(x,
      [](const atermpp::aterm& t) -> atermpp::aterm
      {
        if (t.function() == core::detail::function_symbol_OpId())
        {
          return atermpp::aterm(core::detail::function_symbol_OpIdNoIndex(), t.begin(), --t.end());
        }
        return t;
      });
}

/// Returns the user-defined constructors of the data specification as an
/// aterm_list, in the serialised `OpIdNoIndex` form.
inline
std::unique_ptr<atermpp::aterm> mcrl2_data_specification_user_defined_constructors(const data_specification& spec)
{
  atermpp::aterm_list list(spec.user_defined_constructors().begin(), spec.user_defined_constructors().end());
  return std::make_unique<atermpp::aterm>(mcrl2_remove_function_symbol_index(list));
}

/// Returns the user-defined mappings of the data specification as an
/// aterm_list, in the serialised `OpIdNoIndex` form.
inline
std::unique_ptr<atermpp::aterm> mcrl2_data_specification_user_defined_mappings(const data_specification& spec)
{
  atermpp::aterm_list list(spec.user_defined_mappings().begin(), spec.user_defined_mappings().end());
  return std::make_unique<atermpp::aterm>(mcrl2_remove_function_symbol_index(list));
}

/// Returns the user-defined equations of the data specification as an
/// aterm_list, in the serialised `OpIdNoIndex` form.
inline
std::unique_ptr<atermpp::aterm> mcrl2_data_specification_user_defined_equations(const data_specification& spec)
{
  atermpp::aterm_list list(spec.user_defined_equations().begin(), spec.user_defined_equations().end());
  return std::make_unique<atermpp::aterm>(mcrl2_remove_function_symbol_index(list));
}

} // namespace mcrl2::data

#endif // MCRL2_SYS_CPP_DATA_H