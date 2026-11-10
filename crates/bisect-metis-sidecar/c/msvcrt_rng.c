/* Explicit msvcrt-v1 profile: serial module-local Windows CRT rand sequence.
 * The native engine is not changed; other libc runtimes are not implied.
 */
#include <stdint.h>
static uint32_t bisect_rng_state=1;
void bisect_msvcrt_srand(unsigned int seed) { bisect_rng_state=(uint32_t)seed; }
int bisect_msvcrt_rand(void) {
    bisect_rng_state=bisect_rng_state*UINT32_C(214013)+UINT32_C(2531011);
    return (int)((bisect_rng_state>>16)&UINT32_C(32767));
}
