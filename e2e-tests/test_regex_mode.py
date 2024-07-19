"""Tests for using regex mode."""

import pytest_tuitest as tt
from utils import config_path, STATUS_OK

COLOR_RED = "\033[0;31m"
COLOR_RESET = "\033[0m"


@tt.with_stdin("test, test indeed")
@tt.with_arguments(["--config", config_path("config_match_test.yaml")])
def test_can_select_from_simple_text(terminal):
    """Verify that text can be selected in regex mode."""
    terminal.wait_for_stable_output()

    first_word_hint = terminal.get_string_at(0, 0, 1)
    assert first_word_hint != " ", "Expected first word hint, found nothing"

    terminal.send(first_word_hint)

    (status, stdout, stderr) = terminal.wait_for_finished()

    assert status == STATUS_OK, "The proces unexpectedly failed"
    assert stdout == "test", "Returned stdout not as expected"
    assert stderr == "", "Expected empty stderr, got something"


@tt.with_stdin(f"{COLOR_RED}test,{COLOR_RESET} test indeed")
@tt.with_arguments(["--config", config_path("config_match_test.yaml")])
def test_can_select_from_colored_text(terminal):
    """Verify that colored text does not interfere with selecting."""
    terminal.wait_for_stable_output()

    first_word_hint = terminal.get_string_at(0, 0, 1)
    assert first_word_hint != " ", "Expected first word hint, found nothing"

    terminal.send(first_word_hint)

    (status, stdout, stderr) = terminal.wait_for_finished()

    assert status == STATUS_OK, "The proces unexpectedly failed"
    assert stdout == "test", "Returned stdout not as expected"
    assert stderr == "", "Expected empty stderr, got something"


@tt.with_stdin("123456789012345\nabcd")
@tt.with_terminal_size(10, 10)
@tt.with_arguments(["--config", config_path("config_match_test.yaml")])
def test_correctly_renders_long_lines(terminal):
    """Verify that text with lines longer than screen width is rendered correctly."""
    terminal.wait_for_stable_output()

    first_row = terminal.get_string_at(0, 0, 10)
    assert first_row == "1234567890", "First rendered row has an unexpected value"

    second_row = terminal.get_string_at(1, 0, 10)
    assert second_row == "12345     ", "Second rendered row has an unexpected value"

    third_row = terminal.get_string_at(2, 0, 10)
    assert third_row == "abcd      ", "Third rendered row has an unexpected value"


@tt.with_stdin("aåбḁ😀")
@tt.with_arguments(["--config", config_path("config_match_test.yaml")])
def test_correctly_renders_non_ascii_characters(terminal):
    """Verify that non-ASCII characters are rendered and handled correctly."""
    terminal.wait_for_stable_output()

    msg = "Non-ASCII character not handled as expected"
    assert "a" == terminal.get_string_at(0, 0, 1), msg
    assert "å" == terminal.get_string_at(0, 1, 1), msg
    assert "б" == terminal.get_string_at(0, 2, 1), msg
    assert "ḁ" == terminal.get_string_at(0, 3, 1), msg
    assert "😀" == terminal.get_string_at(0, 4, 1), msg


@tt.with_stdin("aåбḁ😀test👽")
@tt.with_arguments(["--config", config_path("config_match_test.yaml")])
def test_can_select_text_from_text_with_non_ascii_characters(terminal):
    """Verify that non-ASCII characters don't interfere with text selection."""
    terminal.wait_for_stable_output()

    # Assume the first hint is q since pytest-tuitest gets confused with
    # 😀 and thinks it's two characters.
    # TODO Update this to dynamically retrieve the hint once pytest-tuitest problem is fixed
    terminal.send("q")

    (status, stdout, stderr) = terminal.wait_for_finished()

    assert status == STATUS_OK, "The proces unexpectedly failed"
    assert stdout == "test", "Returned stdout not as expected"
    assert stderr == "", "Expected empty stderr, got something"


@tt.with_stdin("0\n1\n2\n3\n4\n5\n6\n7\n8\n9\n")
@tt.with_terminal_size(10, 10)
@tt.with_arguments(["--config", config_path("config_match_test.yaml")])
def test_correctly_renders_text_of_same_height_as_terminal(terminal):
    """Verify rendering works when text line count is same as terminal height.

    This should work even if there is a new line at the end of the file since
    this is just a terminator and does not make the line actually longer.
    """
    terminal.wait_for_stable_output()

    msg = "The last line was not rendered correctly"
    assert terminal.get_string_at(9, 0, 1) == "9", msg
