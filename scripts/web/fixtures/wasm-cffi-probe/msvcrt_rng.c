/* Explicit experimental profile: Windows CRT rand-compatible sequence.
 * This is NOT the default Emscripten libc RNG, portable GK RNG, or a change to
 * the Cargo cache. The runner must verify this against the actual native CRT.
 * One serial module/worker owns this state; concurrent calls are not supported.
 */
#include <stdint.h>
static uint32_t bisect_rng_state=1;
void bisect_msvcrt_srand(unsigned int seed) { bisect_rng_state=(uint32_t)seed; }
int bisect_msvcrt_rand(void) {
    bisect_rng_state=bisect_rng_state*UINT32_C(214013)+UINT32_C(2531011);
    return (int)((bisect_rng_state>>16)&UINT32_C(32767));
}
