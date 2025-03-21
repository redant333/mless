"""Tests for handling input text longer than the screen."""

import pytest_tuitest as tt
from utils import config_path


@tt.with_stdin("1\n2\n3\n4\n5\n6\n7\n8")
@tt.with_terminal_size(10, 5)
@tt.with_arguments(["--config", config_path("config_match_test.yaml")])
def test_displays_first_page_of_long_text(terminal):
    """Verify that the beginning of the long text is rendered.

    If the first page is not determined correctly, the text will be scrolled
    or miss some expected lines.
    """
    terminal.wait_for_stable_output()

    first_line = terminal.get_string_at(0, 0, 1)
    assert first_line == "1", f"Expected 1 at the first line, found {first_line}"

    last_line = terminal.get_string_at(4, 0, 1)
    assert last_line == "5", f"Expected 5 at the last line, found {last_line}"
