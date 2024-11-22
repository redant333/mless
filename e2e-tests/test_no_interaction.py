"""Tests that don't require user interaction apart from running."""

import re

import pytest_tuitest as tt
from utils import STATUS_OK


@tt.with_arguments(["--help"])
def test_can_display_help(terminal):
    """Verify that help is displayed when run with --help."""
    (status, stdout, stderr) = terminal.wait_for_finished()

    msg = "Expected program to finish successfully, got non-zero exit status"
    assert status == STATUS_OK, msg

    assert stderr == "", "Expected empty stderr, got something else"

    msg = "Expected stdout to contain word 'Usage', not found"
    assert "Usage" in stdout, msg

    msg = "Expected stdout to contain word 'Arguments', not found"
    assert "Arguments" in stdout, msg

    msg = "Expected stdout to contain word 'Options', not found"
    assert "Options" in stdout, msg


@tt.with_arguments(["--show-default-config"])
def test_shows_default_config_when_requested(terminal):
    """Verify that the default config is shown when run with --show-default-config."""
    (status, stdout, stderr) = terminal.wait_for_finished()

    msg = "Expected program to finish successfully, got non-zero exit status"
    assert status == STATUS_OK, msg

    assert stderr == "", "Expected empty stderr, got something else"

    msg = "Expected stdout to contain a field for hint characters, not found"
    assert "hint_characters:" in stdout, msg

    msg = "Expected stdout to at least one documentation comment, not found"
    assert re.search(r"^ *#.+$", stdout, flags=re.MULTILINE) is not None, msg
