#!/usr/bin/env python3
"""
This script fetches the public suffix list from https://publicsuffix.org/list/public_suffix_list.dat,
parses and cleans the entries (removing leading "!" and "*."), sorts them,
and writes them to twistrs/src/tlds.rs.

If the file has changed, it creates a branch, commits the changes, pushes them,
and opens a pull request using the GitHub CLI. Otherwise it does nothing.
"""

import re
import subprocess
import sys
import os
import requests
from datetime import datetime

PSL_URL = "https://publicsuffix.org/list/public_suffix_list.dat"
RUST_OUTPUT_PATH = "twistrs/src/tlds.rs"
GIT_BRANCH = "update-tlds"

today = datetime.now().strftime("%Y-%m-%d")
COMMIT_MESSAGE = f"misc: update tld list {today} [skip ci]"
PR_TITLE = f"misc: update tld list {today}"
PR_BODY = "This PR updates the TLD list automatically from the Public Suffix List."

def fetch_psl(url):
    print(f"Fetching PSL from {url}...")
    response = requests.get(url)
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
    for s in suffixes:
        rust_lines.append(f'    "{s}",')
    rust_lines.append("];\n")
    
    content = "\n".join(rust_lines)

    with open(output_path, "w", encoding="utf-8") as f:
        f.write(content)

    print(f"wrote {array_len} suffixes to {output_path}")
    return content

def git_diff_exists(path):
    result = subprocess.run(["git", "diff", "--exit-code", path])
    return result.returncode != 0

def git_commit_and_push(file_path, branch_name, commit_message):
    # Make sure we're on the branch.
    subprocess.run(["git", "checkout", branch_name], check=True)
    subprocess.run(["git", "add", file_path], check=True)
    subprocess.run(["git", "commit", "-m", commit_message], check=True)
    subprocess.run(["git", "push", "origin", branch_name], check=True)

def create_pull_request(branch_name, title, body):
    print("creating pull request...")
    subprocess.run(["gh", "pr", "create", "--base", "main", "--head", branch_name,
                    "--title", title, "--body", body], check=True)

def main():
    try:
        psl_text = fetch_psl(PSL_URL)
    except Exception as e:
        print(f"error fetching psl: {e}", file=sys.stderr)
        sys.exit(1)

    suffixes = parse_psl(psl_text)
    print(f"parsed {len(suffixes)} suffix entries")

    generate_rust_array(suffixes, RUST_OUTPUT_PATH)

    if git_diff_exists(RUST_OUTPUT_PATH):
        print("changes detected in the tld file")

        # Create (or switch to) the branch.
        # Try to create a new branch; if it already exists, just checkout.
        subprocess.run(["git", "checkout", "-B", GIT_BRANCH], check=True)
        git_commit_and_push(RUST_OUTPUT_PATH, GIT_BRANCH, COMMIT_MESSAGE)
        create_pull_request(GIT_BRANCH, PR_TITLE, PR_BODY)
    else:
        print("no changes detected. exiting")

if __name__ == "__main__":
    main()

