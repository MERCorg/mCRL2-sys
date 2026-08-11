#ifndef MCRL2_SYS_CPP_EXCEPTION_H
#define MCRL2_SYS_CPP_EXCEPTION_H

#include <cstdlib>
#include <exception>

#ifdef MCRL2_ENABLE_CPPTRACE
  #include <cpptrace/from_current.hpp>
#endif // MCRL2_ENABLE_CPPTRACE

namespace rust::behavior {

// Overrides the try-catch block that cxx wraps every `-> Result<...>` bridge
// function in. The override exists for two reasons, and is defined
// unconditionally so that every build has the same exception behaviour:
//
//  * cxx's own implementation only catches `const std::exception&`. Since the
//    generated shims are noexcept, any other throw crossing a bridge would
//    terminate the process rather than being reported to Rust.
//  * with cpptrace enabled the stack trace of the throw site can be printed,
//    which requires catching inside the CPPTRACE_TRY/CPPTRACE_CATCH macros.
#ifdef MCRL2_ENABLE_CPPTRACE

template <typename Try, typename Fail>
static void trycatch(Try &&func, Fail &&fail) noexcept
{
  CPPTRACE_TRY {
    func();
  } CPPTRACE_CATCH(const std::exception &e) {
    if (std::getenv("RUST_BACKTRACE") != nullptr) {
      cpptrace::from_current_exception().print();
    }

    fail(e.what());
  } catch (...) {
    fail("Unknown C++ exception (not derived from std::exception)");
  }
}

#else // MCRL2_ENABLE_CPPTRACE

template <typename Try, typename Fail>
static void trycatch(Try &&func, Fail &&fail) noexcept
{
  try {
    func();
  } catch (const std::exception &e) {
    fail(e.what());
  } catch (...) {
    fail("Unknown C++ exception (not derived from std::exception)");
  }
}

#endif // MCRL2_ENABLE_CPPTRACE

} // namespace rust::behavior

#endif // MCRL2_SYS_CPP_EXCEPTION_H
