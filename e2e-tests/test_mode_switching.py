"""Tests for mode switching mode."""

import pytest_tuitest as tt
from utils import config_path


@tt.with_stdin("aaa bbb")
@tt.with_terminal_size(80, 40)
@tt.with_arguments(["--config", config_path("config_aaa_bbb.yaml")])
def test_can_switch_between_modes(terminal):
    """Verify that it is possible to switch between selection modes."""
    terminal.wait_for_stable_output()

    # Verify that the start state is as expected
    msg = "Expected aaa to be marked and bbb not"
    assert terminal.get_background_at(0, 0) != tt.Color16.DEFAULT, msg
    assert terminal.get_background_at(0, 4) == tt.Color16.DEFAULT, msg

    # Switch to mode selection
    mode_selection_hotkey = " "
    terminal.send(mode_selection_hotkey)
    terminal.wait_for_stable_output()

    msg = "Mode hints not displayed when expected"
    assert "[a] aaa" in terminal.get_string_at(1, 0, 80), msg
    assert "[b] bbb" in terminal.get_string_at(2, 0, 80), msg

    # Switch to mode b
    terminal.send("b")
    terminal.wait_for_stable_output()

    # Verify that the hints have changed
    msg = "Expected bbb to be marked and aaa not"
    assert terminal.get_background_at(0, 0) == tt.Color16.DEFAULT, msg
    assert terminal.get_background_at(0, 4) != tt.Color16.DEFAULT, msg


@tt.with_stdin("test")
@tt.with_terminal_size(80, 40)
@tt.with_arguments(["--config", config_path("config_aaa_bbb.yaml")])
def test_pressing_non_hotkey_does_not_affect_mode_switching(terminal):
    """Verify that mode switching stays open when pressing a hotkey that
    is not associated with any mode.
    """
    terminal.wait_for_stable_output()

    # Enter mode selection
    mode_selection_hotkey = " "
    terminal.send(mode_selection_hotkey)
    terminal.wait_for_stable_output()

    msg = "Mode hints not displayed when expected"
    assert "[a] aaa" in terminal.get_string_at(1, 0, 80), msg
    assert "[b] bbb" in terminal.get_string_at(2, 0, 80), msg

    # Send invalid hotkey
    terminal.send("x")
    terminal.wait_for_stable_output()

    # Verify that we are still in the mode selection
    msg = "Mode hints not displayed when expected"
    assert "[a] aaa" in terminal.get_string_at(1, 0, 80), msg
    assert "[b] bbb" in terminal.get_string_at(2, 0, 80), msg
