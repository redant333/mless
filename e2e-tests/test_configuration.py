"""Tests for various configuration options."""

import pytest_tuitest as tt
from pytest_tuitest import Color256
from utils import config_path


@tt.with_stdin("test, test indeed")
@tt.with_arguments(["--config", config_path("config_match_test_custom_colors.yaml")])
def test_can_customize_hint_style(terminal):
    """Verify that hint style can be customized in the config file."""
    terminal.wait_for_stable_output()

    msg = "Expected to find the color from the config, found something else"
    # Note that crossterm does not use \e[<number>m sequences for 16 colors which
    # results in pyte interpreting them the same way 256 colors
    assert terminal.get_background_at(0, 0) == Color256.COLOR9, msg
    assert terminal.get_foreground_at(0, 0) == Color256.COLOR10, msg


@tt.with_stdin("test, test indeed")
@tt.with_arguments(["--config", config_path("config_match_test_custom_colors.yaml")])
def test_can_customize_highlight_style(terminal):
    """Verify that hint style can be customized in the config file."""
    terminal.wait_for_stable_output()

    msg = "Expected to find the color from the config, found something else"
    # Note that crossterm does not use \e[<number>m sequences for 16 colors which
    # results in pyte interpreting them the same way 256 colors
    assert terminal.get_background_at(0, 1) == Color256.COLOR12, msg
    assert terminal.get_foreground_at(0, 1) == Color256.COLOR11, msg


@tt.with_stdin("test, test indeed")
@tt.with_arguments(
    ["--config", config_path("config_match_test_various_color_formats.yaml")]
)
def test_all_color_formats_supported(terminal):
    """Verify that a color can be specified as word, 256 color or RGB color."""
    terminal.wait_for_stable_output()

    msg = "Expected to find the color from the config, found something else"
    assert terminal.get_background_at(0, 0) == Color256.COLOR9, msg
    assert terminal.get_foreground_at(0, 0) == Color256.COLOR166, msg
    assert terminal.get_background_at(0, 1) == "102030", msg
