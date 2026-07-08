/// Wrapper around the log library of the mCRL2 toolset.
#ifndef MCRL2_SYS_CPP_LOG_H
#define MCRL2_SYS_CPP_LOG_H

#include "mcrl2/utilities/logger.h"

namespace mcrl2::log
{

inline
void mcrl2_set_reporting_level(std::size_t level)
{
    // log_level_t is a plain enum ranging from quiet (0) up to trace. Casting an
    // out-of-range integer to it would produce an invalid enumerator (undefined
    // behaviour), so clamp the requested level to the most verbose valid value
    // instead.
    constexpr std::size_t max_level = static_cast<std::size_t>(mcrl2::log::trace);
    const std::size_t clamped = level > max_level ? max_level : level;
    mcrl2::log::logger::set_reporting_level(static_cast<mcrl2::log::log_level_t>(clamped));
}

} // namespace mcrl2::log

#endif // MCRL2_SYS_CPP_LOG_H