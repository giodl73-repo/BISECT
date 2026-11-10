"""Check the shared States conversion against the existing frozen native body."""
from pathlib import Path

legacy = Path('crates/bisect-cli/examples/support/states_config_legacy.rs').read_text(encoding='utf-8')
shared = Path('crates/bisect-runner/src/states_config.rs').read_text(encoding='utf-8')
legacy = legacy[legacy.index('cfg.debug = args.debug;'):legacy.rfind('}')]
shared = shared[shared.index('cfg.debug = args.debug;'):shared.index('\n        config\n')]
# Keep the frozen body intact; the explicit CVD correction is tested against
# single-State resolution and direct native CVD execution, not a new golden copy.
start = shared.index('// BEGIN reviewed bulk CVD override correction')
end = shared.index('// END reviewed bulk CVD override correction') + len('// END reviewed bulk CVD override correction')
assert start < end
shared = shared[:start] + shared[end:]


def canonical(text):
    text = text.replace('use crate::state_config::{SeedCompositor, SplitStrategy};', 'use bisect_cli::runner::{SeedCompositor, SplitStrategy};')
    text = text.replace('use crate::args::SearchMode as SeM;', 'use bisect_cli::args::SearchMode as SeM;')
    return '\n'.join(line.strip() for line in text.splitlines() if line.strip())

assert canonical(shared) == canonical(legacy), 'Shared States conversion differs from the frozen native body'
cli = Path('crates/bisect-cli/src/args.rs').read_text(encoding='utf-8')
assert 'pub use bisect_runner::states_args::' in cli and 'pub struct StatesArgs' not in cli
runner = Path('crates/bisect-cli/src/runner/mod.rs').read_text(encoding='utf-8')
assert 'StatesConfiguration::from_args(args)' in runner
assert 'if settings.balance_tolerance.is_some(){self.balance_tolerance=settings.balance_tolerance;}' in runner
print('PASS States extraction: frozen conversion retained outside the reviewed CVD correction; CLI delegates to shared parser/configuration and preserves unspecified tolerance')
