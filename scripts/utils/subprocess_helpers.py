"""
Subprocess command building utilities.

This module provides helper functions to build subprocess commands
with common arguments, reducing duplication in command construction
across pipeline scripts.
"""

import sys
from pathlib import Path


def build_pipeline_command(script_path, args=None, **extra_args):
    """
    Build pipeline command with common arguments.

    Constructs a subprocess command list with Python executable,
    script path, and common arguments from argparse namespace.

    Args:
        script_path: Path to script (Path or str)
        args: Parsed arguments from argparse (optional)
        **extra_args: Additional arguments as key=value pairs

    Returns:
        list: Command arguments for subprocess.run()

    Example:
        >>> from argparse import Namespace
        >>> args = Namespace(year='2020', dpi=150, print_only=True)
        >>> cmd = build_pipeline_command(
        ...     'scripts/pipeline/process_single_state.py',
        ...     args,
        ...     state='CA',
        ...     version='v1'
        ... )
        >>> print(' '.join(cmd))
        python scripts/pipeline/process_single_state.py --year 2020 --dpi 150 --print-only --state CA --version v1
    """
    cmd = [sys.executable, str(script_path)]

    # Add args from argparse namespace if provided
    if args is not None:
        if hasattr(args, 'year'):
            cmd.extend(['--year', str(args.year)])
        if hasattr(args, 'dpi'):
            cmd.extend(['--dpi', str(args.dpi)])
        if hasattr(args, 'print_only') and args.print_only:
            cmd.append('--print-only')
        if hasattr(args, 'debug') and args.debug:
            cmd.append('--debug')
        if hasattr(args, 'force') and args.force:
            cmd.append('--force')

    # Add extra arguments
    for key, value in extra_args.items():
        if value is None:
            continue

        # Convert underscore to hyphen for argument names
        arg_name = f'--{key.replace("_", "-")}'

        # Handle boolean arguments
        if isinstance(value, bool):
            if value:
                cmd.append(arg_name)
        else:
            cmd.extend([arg_name, str(value)])

    return cmd


def build_command_string(script_path, args=None, **extra_args):
    """
    Build pipeline command as string (for display/logging).

    Similar to build_pipeline_command but returns a space-separated
    string instead of a list. Useful for logging or print-only mode.

    Args:
        script_path: Path to script (Path or str)
        args: Parsed arguments from argparse (optional)
        **extra_args: Additional arguments as key=value pairs

    Returns:
        str: Command as space-separated string

    Example:
        >>> cmd_str = build_command_string(
        ...     'scripts/pipeline/process_single_state.py',
        ...     year='2020',
        ...     state='CA'
        ... )
        >>> print(cmd_str)
        python scripts/pipeline/process_single_state.py --year 2020 --state CA
    """
    cmd_list = build_pipeline_command(script_path, args, **extra_args)
    return ' '.join(cmd_list)


def add_common_flags_to_command(cmd, args):
    """
    Add common flags from argparse namespace to existing command list.

    Modifies command list in-place to add standard flags like
    --print-only, --debug, --force if present in args.

    Args:
        cmd: Command list to modify (modified in-place)
        args: Parsed arguments from argparse

    Returns:
        list: The modified command list (same object as input)

    Example:
        >>> from argparse import Namespace
        >>> cmd = ['python', 'script.py']
        >>> args = Namespace(print_only=True, debug=False)
        >>> add_common_flags_to_command(cmd, args)
        >>> print(cmd)
        ['python', 'script.py', '--print-only']
    """
    if hasattr(args, 'print_only') and args.print_only:
        cmd.append('--print-only')
    if hasattr(args, 'debug') and args.debug:
        cmd.append('--debug')
    if hasattr(args, 'force') and args.force:
        cmd.append('--force')

    return cmd


def get_python_executable():
    """
    Get path to current Python executable.

    Returns:
        str: Path to Python executable

    Example:
        >>> python = get_python_executable()
        >>> print(python)
        C:\\Python\\python.exe
    """
    return sys.executable
