#!/usr/bin/env python3
"""
Updates data files used by Twistrs:
- Public Suffix List -> twistrs/src/tlds.rs
- WHOIS/RDAP servers -> twistrs/data/whois-servers.json
- GeoIP (MaxMind-DB) submodule should be updated before running this script

If any tracked files change, it creates/updates a branch, commits, pushes, and
opens a PR using the GitHub CLI.
"""

import json
import re
import subprocess
import sys
from datetime import datetime

import requests

PSL_URL = "https://publicsuffix.org/list/public_suffix_list.dat"
WHOIS_URL = "https://raw.githubusercontent.com/FurqanSoftware/node-whois/master/servers.json"

RUST_OUTPUT_PATH = "twistrs/src/tlds.rs"
WHOIS_OUTPUT_PATH = "twistrs/data/whois-servers.json"
SUBMODULE_PATH = "twistrs/data/MaxMind-DB"

GIT_BRANCH = "github-bot-update-data-files"

today = datetime.utcnow().strftime("%Y-%m-%d")
COMMIT_MESSAGE = f"misc: update data files {today} [skip ci]"
PR_TITLE = f"misc: update data files {today}"
PR_BODY = """This PR updates data files automatically.

Sources:
- Public Suffix List (publicsuffix.org)
- WHOIS/RDAP servers (FurqanSoftware/node-whois)
- GeoIP (MaxMind-DB submodule)
"""


def fetch_text(url):
    print(f"Fetching {url}...")
    response = requests.get(url, timeout=30)
    response.raise_for_status()
    return response.text


def parse_psl(psl_text):
    """
    Parses the PSL text and returns a sorted list of suffixes,
    ignoring any lines in comments, blank lines, and any entries
    that fall within the "PRIVATE DOMAINS" block.
    """
    lines = psl_text.splitlines()
    in_private_block = False
    valid_lines = []

    for line in lines:
        stripped = line.strip()
        # Detect start and end of private domains block
        if "===BEGIN PRIVATE DOMAINS===" in stripped:
            in_private_block = True
            continue
        if "===END PRIVATE DOMAINS===" in stripped:
            in_private_block = False
            continue

        # Skip comments and blank lines
        if not stripped or stripped.startswith("//"):
            continue

        # Skip any lines inside the private block
        if in_private_block:
            continue

        # Remove any leading exclamation marks or wildcards (e.g. "!city." or "*.")
        cleaned = re.sub(r"^(?:!\*\.|!\.|!\*|^[\*.!]+)", "", stripped)
        valid_lines.append(cleaned)

    return sorted(valid_lines)


def generate_rust_array(suffixes, output_path):
    array_len = len(suffixes)

    rust_lines = []
    rust_lines.append("// This file is auto-generated. Do not edit manually.")
    rust_lines.append(f"pub const TLDS: [&str; {array_len}] = [")
    for suffix in suffixes:
        rust_lines.append(f'    "{suffix}",')
    rust_lines.append("];\n")

    content = "\n".join(rust_lines)

    with open(output_path, "w", encoding="utf-8") as f:
        f.write(content)

    print(f"wrote {array_len} suffixes to {output_path}")


def update_psl():
    try:
        psl_text = fetch_text(PSL_URL)
    except Exception as exc:
        print(f"error fetching psl: {exc}", file=sys.stderr)
        raise

    suffixes = parse_psl(psl_text)
    print(f"parsed {len(suffixes)} suffix entries")
    generate_rust_array(suffixes, RUST_OUTPUT_PATH)


def update_whois_servers():
    try:
        whois_text = fetch_text(WHOIS_URL)
    except Exception as exc:
        print(f"error fetching whois servers: {exc}", file=sys.stderr)
        raise

    try:
        json.loads(whois_text)
    except json.JSONDecodeError as exc:
        print(f"error parsing whois servers json: {exc}", file=sys.stderr)
        raise

    if whois_text and not whois_text.endswith("\n"):
        whois_text += "\n"

    with open(WHOIS_OUTPUT_PATH, "w", encoding="utf-8") as f:
        f.write(whois_text)

    print(f"wrote whois servers to {WHOIS_OUTPUT_PATH}")


def git_diff_exists(paths):
    result = subprocess.run(["git", "diff", "--exit-code", "--", *paths])
    return result.returncode != 0


def git_commit_and_push(paths, branch_name, commit_message):
    subprocess.run(["git", "checkout", "-B", branch_name], check=True)
    subprocess.run(["git", "add", *paths], check=True)
    subprocess.run(["git", "commit", "-m", commit_message], check=True)
    subprocess.run(["git", "push", "--force", "origin", branch_name], check=True)


def create_pull_request(branch_name, title, body):
    print("creating pull request...")
    subprocess.run(
        [
            "gh",
            "pr",
            "create",
            "--base",
            "main",
            "--head",
            branch_name,
            "--title",
            title,
            "--body",
            body,
        ],
        check=True,
    )


def main():
    update_psl()
    update_whois_servers()

    tracked_paths = [RUST_OUTPUT_PATH, WHOIS_OUTPUT_PATH, SUBMODULE_PATH]

    if git_diff_exists(tracked_paths):
        print("changes detected in data files")
        git_commit_and_push(tracked_paths, GIT_BRANCH, COMMIT_MESSAGE)
        create_pull_request(GIT_BRANCH, PR_TITLE, PR_BODY)
    else:
        print("no changes detected. exiting")


if __name__ == "__main__":
    main()
