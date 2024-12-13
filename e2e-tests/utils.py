"""Functions and constants to aid testing."""

from pathlib import Path

STATUS_ERROR = 255
STATUS_OK = 0

COLOR_RED = "\033[0;31m"
STYLE_BOLD = "\033[1m"

COLOR_BG_BLUE = "\033[44m"

ANSI_RESET = "\033[0m"


def config_path(path):
    """Return the full path given the path relative to supporting configs directory."""
    supporting_configs_dir = Path(__file__).parent / "support_files" / "configs"

    return supporting_configs_dir / path
