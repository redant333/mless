"""Tests that don't require user interaction apart from running."""
import pytest_tuitest as tt


@tt.with_arguments(["--help"])
def test_can_display_help(terminal):
    """Verify that help is displayed when run with --help."""
    (status, _, _) = terminal.wait_for_finished()

    assert status == 0, f"Expected the program to return success, it returned {status}"

    (row, col) = (2, 0)
    expected = "Usage:"
    actual = terminal.get_string_at(row, col, len(expected))

    msg = f"Expected '{expected}' at ({row}, {col}), found {actual}"
    assert actual == expected, msg
