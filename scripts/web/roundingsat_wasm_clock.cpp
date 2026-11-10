// RoundingSat measures differences of getrusage().ru_utime. Emscripten's
// syscall is a stub, so use the monotonic browser/Node clock for this WASM port.
// This affects timing and resource limits only, never integer proof arithmetic.
#include <emscripten.h>
#include <cerrno>
#include <cstring>
#include <sys/resource.h>

extern "C" double bisect_solver_elapsed_seconds() {
  return emscripten_get_now() / 1000.0;
}

extern "C" int __wrap_getrusage(int who, struct rusage* usage) {
  if (who != RUSAGE_SELF || !usage) {
    errno = EINVAL;
    return -1;
  }
  std::memset(usage, 0, sizeof(*usage));
  const double now = bisect_solver_elapsed_seconds();
  usage->ru_utime.tv_sec = static_cast<decltype(usage->ru_utime.tv_sec)>(now);
  usage->ru_utime.tv_usec = static_cast<decltype(usage->ru_utime.tv_usec)>((now - usage->ru_utime.tv_sec) * 1000000.0);
  return 0;
}
