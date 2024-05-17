e2e_tests_dir := "e2e-tests"
test_venv_path := e2e_tests_dir / "venv"
test_venv_activate := test_venv_path / "bin" / "activate"
test_venv_requirements := e2e_tests_dir / "requirements.txt"
executable_debug := "target/debug/mouseless-selector"

# Initialize end to end testing venv. Does nothing if it already exists.
test-venv-init:
    #!/bin/bash -eu
    echo "Setting up testing venv at {{test_venv_path}}"

    if [ -d "{{test_venv_path}}" ]
    then
        echo "Testing venv already set up at {{test_venv_path}}"
        exit 0
    fi

    python3 -m venv "{{test_venv_path}}"
    source "{{test_venv_activate}}"
    pip install -r "{{test_venv_requirements}}"


# Delete the testing venv.
test-venv-delete:
    #!/bin/bash -eu
    if [ -d "{{test_venv_path}}" ]
    then
        echo "Confirm deleting an existing testing venv at {{test_venv_path}}"
        rm -rI "{{test_venv_path}}"
    fi

# Reinitialize end to end testing venv, even if it already exists.
test-venv-reinit: test-venv-delete test-venv-init

# Run end to end tests.
test-e2e-run: test-venv-init
    #!/bin/bash -eu
    source "{{test_venv_activate}}"
    executable_path=$(realpath {{executable_debug}})
    cd "{{e2e_tests_dir}}"
    pytest --tuitest-default-executable="$executable_path"

alias e := test-e2e-run

# Run all pre-commit checks.
pre-commit:
    pre-commit run --all-files
