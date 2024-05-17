"""Tests that don't require user interaction apart from running."""

import pytest_tuitest as tt


@tt.with_arguments(["--help"])
def test_can_display_help(terminal):
    """Verify that help is displayed when run with --help."""
    (status, stdout, stderr) = terminal.wait_for_finished()

    msg = "Expected program to finish successfully, got non-zero exit status"
    assert status == 0, msg

    assert stderr == "", "Expected empty stderr, got something else"

    msg = "Expected stdout to contain word 'Usage', not found"
    assert "Usage" in stdout, msg

    msg = "Expected stdout to contain word 'Arguments', not found"
    assert "Arguments" in stdout, msg

    msg = "Expected stdout to contain word 'Options', not found"
    assert "Options" in stdout, msg
