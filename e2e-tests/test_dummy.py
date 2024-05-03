"""Tests that verify that that the testing setup works."""
import pytest_tuitest as tt

@tt.test_executable("/usr/bin/pwd")
def test_dummy(terminal):
    "Verify that pytest-tuitest works"
    terminal.wait_for_finished()
    assert terminal.get_string_at(0,0,5) == "/home"
