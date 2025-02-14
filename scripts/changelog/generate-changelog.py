#!/usr/bin/env python3
from argparse import ArgumentParser
from argparse import RawTextHelpFormatter

from git import Repo

CHANGELOG_MARKER = "CHANGELOG:"
REPO_ROOT = f"{__file__}/../../.."
POST_PROCESSING_REPLACEMENTS = {
    "[BUGFIX]": "`BUGFIX `",
    "[CHANGE]": "`CHANGE `",
    "[FEATURE]": "`FEATURE`",
}


def parse_args():
    parser = ArgumentParser(
        description="Generate a Markdown changelog from commit messages containing"
        + f"{CHANGELOG_MARKER}.\n\n"
        + f"For any commit message that contains '{CHANGELOG_MARKER}', the lines after it\n"
        + "will be included in the generated changelog.\n\n"
        + "The following strings are styled in a special way:\n"
        + ", ".join(f"'{val}'" for val in POST_PROCESSING_REPLACEMENTS),
        formatter_class=RawTextHelpFormatter,
    )

    parser.add_argument(
        "commit_range",
        help="commit range to use. Follows the same format as e.g. git log",
    )

    return parser.parse_args()


def generate_entry(commit_message):
    changelog_entry = commit_message.split(CHANGELOG_MARKER)[-1]
    changelog_entry = changelog_entry.strip()

    for replace, replace_with in POST_PROCESSING_REPLACEMENTS.items():
        changelog_entry = changelog_entry.replace(replace, replace_with)

    return changelog_entry


def main():
    args = parse_args()

    repo = Repo(REPO_ROOT)

    for commit in repo.iter_commits(args.commit_range):
        message = commit.message

        if CHANGELOG_MARKER not in message:
            # No changelog in this message
            continue

        print(generate_entry(message))


if __name__ == "__main__":
    main()
