"""Tests for various configuration options."""

import pytest_tuitest as tt
from pytest_tuitest import Color256
from utils import config_path

# Crossterm does not use \e[<number>m sequences for 16 colors which
# results in pyte interpreting them the same way 256 colors. When
# matching a mapping is needed.
RED256 = Color256.COLOR9
GREEN256 = Color256.COLOR10
YELLOW256 = Color256.COLOR11
BLUE256 = Color256.COLOR12


@tt.with_stdin("test, test indeed")
@tt.with_arguments(["--config", config_path("config_match_test_custom_colors.yaml")])
def test_can_customize_hint_style(terminal):
    """Verify that hint style can be customized in the config file."""
    terminal.wait_for_stable_output()

    msg = "Expected to find the color from the config, found something else"
    # Note that crossterm does not use \e[<number>m sequences for 16 colors which
    # results in pyte interpreting them the same way 256 colors
    assert terminal.get_background_at(0, 0) == RED256, msg
    assert terminal.get_foreground_at(0, 0) == GREEN256, msg


@tt.with_stdin("test, test indeed")
@tt.with_arguments(["--config", config_path("config_match_test_custom_colors.yaml")])
def test_can_customize_highlight_style(terminal):
    """Verify that hint style can be customized in the config file."""
    terminal.wait_for_stable_output()

    msg = "Expected to find the color from the config, found something else"
    # Note that crossterm does not use \e[<number>m sequences for 16 colors which
    # results in pyte interpreting them the same way 256 colors
    assert terminal.get_background_at(0, 1) == BLUE256, msg
    assert terminal.get_foreground_at(0, 1) == YELLOW256, msg


@tt.with_stdin("test, test indeed")
@tt.with_arguments(["--config", config_path("config_custom_mode_switcher.yaml")])
@tt.with_terminal_size(80, 40)
def test_can_customize_mode_switching_dialog(terminal):
    """Verify that different aspects of the mode switching dialog can be customized."""
    terminal.wait_for_stable_output()

    mode_selection_hotkey = " "
    terminal.send(mode_selection_hotkey)

    terminal.wait_for_stable_output()

    divider_column = 55
    assert (
        terminal.get_foreground_at(0, divider_column) == RED256
    ), "Expected divider to have the color from the config"

    hotkey_column = divider_column + 2
    assert (
        terminal.get_foreground_at(1, hotkey_column) == GREEN256
    ), "Expected hotkey to have the color from the config"

    name_column = hotkey_column + 4
    assert (
        terminal.get_foreground_at(1, name_column) == BLUE256
    ), "Expected mode name to have the color from the config"


@tt.with_stdin("test, test indeed")
@tt.with_arguments(
    ["--config", config_path("config_match_test_various_color_formats.yaml")]
)
def test_all_color_formats_supported(terminal):
    """Verify that a color can be specified as word, 256 color or RGB color."""
    terminal.wait_for_stable_output()

    msg = "Expected to find the color from the config, found something else"
    assert terminal.get_background_at(0, 0) == RED256, msg
    assert terminal.get_foreground_at(0, 0) == Color256.COLOR166, msg
    assert terminal.get_background_at(0, 1) == "102030", msg
