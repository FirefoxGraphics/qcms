#!/bin/sh
# Usage: sync.sh <path/to/filtered>
#
# A script to sync QCMS from Mozilla's git mirror of Gecko [1] to the
# FirefoxGraphics GitHub organisation's standalone QCMS repository [2]. This
# script runs as part of a GitHub Action in this repository triggered on regular
# intervals. This script has been copied from Servo's webrender fork. [3]
#
# The procedure for the sync is:
#
# 1. Clone a copy of the GitHub gecko-dev repository to the "_cache" directory.
# 2. Filter that repository using `git-filter-repo` and create a new local git
#    repository with the filtered contents into a directory specified by the
#    argument passed to this script. The filtered contents are determined
#    by the configuration in `.github/sync/qcms.paths`.
# 3. Cherry-pick the new commits into the repository in the current working
#    directory. The commits applied from the filtered repository are determined
#    by choosing every commit after the hash found in the file
#    `.github/sync/UPSTREAM_COMMIT`
#
# Note that this script relies on the idea that filtering `gecko-dev` the same
# way more than once will result in the same commit hashes.
#
# If at some point, `qcms.paths` is modified and the commit hashes change,
# then a single manual filter will have to happen in order to translate the
# hash in the original filtered repository to the new one. The procedure for this
# is roughly:
#
# 1. Run `git-filter-repo` locally and note the new hash of the latest
#    commit included from upstream.
# 2. Replace the contents `UPSTREAM_COMMIT` with that hash and commit
#    it together with your changes to `qcms.paths`.
#
# [1]: <https://github.com/mozilla/gecko-dev/> mirrored from
#      <https://hg.mozilla.org/mozilla-central>
# [2]: <https://github.com/FirefoxGraphics/qcms/>
# [3]: <https://github.com/servo/webrender/>
set -eu

root_dir=$(pwd)
cache_dir=$root_dir/_cache

# Configure git because we will be making commits.
git_name="QCMS Upstream Sync"
git_email="noreply@github.com"

step() {
    if [ "${TERM-}" != '' ]; then
        tput setaf 12
    fi
    >&2 printf '* %s\n' "$*"
    if [ "${TERM-}" != '' ]; then
        tput sgr0
    fi
}

step "Creating directory for filtered upstream repo if needed"
mkdir -p "$1"
cd -- "$1"
filtered=$(pwd)

step "Creating cache directory if needed"
mkdir -p "$cache_dir"
cd "$cache_dir"
export PATH="$PWD:$PATH"

step "Downloading git-filter-repo if needed"
if ! git filter-repo --version 2> /dev/null; then
    curl -O https://raw.githubusercontent.com/newren/git-filter-repo/v2.38.0/git-filter-repo
    chmod +x git-filter-repo

    git filter-repo --version
fi

step "Cloning upstream if needed"
if ! [ -e upstream ]; then
    git clone --bare --single-branch --progress https://github.com/mozilla/gecko-dev.git upstream
fi

step "Updating upstream"
branch=$(git -C upstream rev-parse --abbrev-ref HEAD)
git -C upstream fetch origin $branch:$branch

step "Filtering upstream"
# Cloning and filtering is much faster than git filter-repo --source --target.
git clone upstream -- "$filtered"
git -C "$filtered" filter-repo --force --paths-from-file "$root_dir/.github/sync/qcms.paths"

step "Adding filtered repository as a remote"
cd "$root_dir"
git remote add filtered-upstream "$filtered"
git fetch filtered-upstream

hash_file=".github/sync/UPSTREAM_COMMIT"
hash=`cat $hash_file`
number_of_commits=`git log $hash..filtered-upstream/master --pretty=oneline | wc -l`

if [ $number_of_commits != '0' ]; then
    step "Applying $number_of_commits new commits"
    git -c user.email="$git_email" -c user.name="$git_name" cherry-pick $hash..filtered-upstream/master
    git rev-parse filtered-upstream/master > "$hash_file"
    git -c user.email="$git_email" -c user.name="$git_name" commit "$hash_file" -m "Syncing to upstream (`cat $hash_file`)"
else
    step "No new commits. Doing nothing."
fi
