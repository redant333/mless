"""Tests for processing of the config file sent via --config."""

import pytest_tuitest as tt
from utils import STATUS_ERROR, config_path


@tt.with_arguments(["--config", "/dev/null/this-does-not-exist"])
def test_fails_with_appropriate_error_when_config_file_does_not_exist(terminal):
    """Verify that the appropriate error is shown when the --config file cannot be read."""
    (status, stdout, stderr) = terminal.wait_for_finished()

    assert status == STATUS_ERROR, "The process returned an unexpected return code"
    assert stdout == "", "Expected nothing on stdout, found something"

    msg = "Expected error on stderr, found something else"
    assert stderr.startswith("Could not open config file "), msg


@tt.with_arguments(["--config", config_path("config_invalid_regex.yaml")])
def test_fails_with_appropriate_error_when_config_is_invalid(terminal):
    """Verify that the appropriate error is shown when invalid config is used."""
    (status, stdout, stderr) = terminal.wait_for_finished()

    assert status == STATUS_ERROR, "The process returned an unexpected return code"
    assert stdout == "", "Expected nothing on stdout, found something"

    msg = "Expected error on stderr, found something else"
    assert stderr.startswith("Could not parse config file"), msg


@tt.with_arguments(["--config", config_path("config_match_test.yaml")])
@tt.with_stdin("test nope test nope")
def test_uses_the_provided_config_when_available(terminal):
    """Verify that the --config is used when it is available."""
    terminal.wait_for_stable_output()

    msg_marked = "Expected the word to be marked, actually not"
    msg_not_marked = "Expected the word not to be marked, actually marked"
    msg_no_output = "Expected input or mark hint, found nothing"

    assert terminal.get_string_at(0, 0, 1) != " ", msg_no_output
    assert terminal.get_background_at(0, 0) != tt.Color16.DEFAULT, msg_marked

    assert terminal.get_string_at(0, 5, 1) != " ", msg_no_output
    assert terminal.get_background_at(0, 5) == tt.Color16.DEFAULT, msg_not_marked

    assert terminal.get_string_at(0, 10, 1) != " ", msg_no_output
    assert terminal.get_background_at(0, 10) != tt.Color16.DEFAULT, msg_marked

    assert terminal.get_string_at(0, 15, 1) != " ", msg_no_output
    assert terminal.get_background_at(0, 15) == tt.Color16.DEFAULT, msg_not_marked


@tt.with_stdin("direct_home\nhome_config\nxdg_home")
@tt.with_env({"HOME": config_path("home_only_direct_config")})
def test_uses_config_file_from_home_when_not_present_in_other_locations(terminal):
    """Verify that $HOME/.mless.yaml config is used if other configs are not available."""
    terminal.wait_for_stable_output()

    msg = "Expected the word on this location to be marked, actually unmarked"
    assert terminal.get_background_at(0, 0) != tt.Color16.DEFAULT, msg

    msg = "Expected the word on this location to be unmarked, actually marked"
    assert terminal.get_background_at(1, 0) == tt.Color16.DEFAULT, msg
    assert terminal.get_background_at(2, 0) == tt.Color16.DEFAULT, msg
