"""Tests for using regex mode."""

import pytest_tuitest as tt
from utils import config_path, STATUS_OK


@tt.with_stdin("test, test indeed")
@tt.with_arguments(["--config", config_path("config_match_test.yaml")])
def test_can_select_in_regex_mode(terminal):
    """Verify that text can be selected in regex mode."""
    terminal.wait_for_stable_output()

    first_word_hint = terminal.get_string_at(0, 0, 1)
    assert first_word_hint != " ", "Expected first word hint, found nothing"

    terminal.send(first_word_hint)

    (status, stdout, stderr) = terminal.wait_for_finished()

    assert status == STATUS_OK, "The proces unexpectedly failed"
    assert stdout == "test", "Returned stdout not as expected"
    assert stderr == "", "Expected empty stderr, got something"
