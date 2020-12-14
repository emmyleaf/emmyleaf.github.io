#!/bin/bash
set -eu

npx npm-check-updates -u

npm install --package-lock-only --ignore-scripts

if $(git diff-index --quiet HEAD); then
  echo 'No dependencies needed to be updated!'
  exit 0
fi

DESCRIPTION="chore: update deps ($(date -I))"
PR_BRANCH=chore/deps-$(date +%s)
PULL_REQUEST_LABELS="dependencies"

git config user.name "github-actions[bot]"
git config user.email "github-actions[bot]@users.noreply.github.com"
git remote set-url origin "https://${GITHUB_ACTOR}:${GITHUB_TOKEN}@github.com/${GITHUB_REPOSITORY}.git"

git checkout -b ${PR_BRANCH}

git commit -am "${DESCRIPTION}"
git push origin ${PR_BRANCH}

RUN_LABEL="${GITHUB_WORKFLOW}@${GITHUB_RUN_NUMBER}"
RUN_ENDPOINT="${GITHUB_SERVER_URL}/${GITHUB_REPOSITORY}/actions/runs/${GITHUB_RUN_ID}"

PR_NUMBER=$(hub pull-request -b nextjs "-m ${DESCRIPTION}" -m "_Generated by [${RUN_LABEL}](${RUN_ENDPOINT})._" | grep -o '[^/]*$')
echo "Created pull request #${PR_NUMBER}."

hub issue update ${PR_NUMBER} -l ${PULL_REQUEST_LABELS}
echo "Labelled pull request #${PR_NUMBER} with '${PULL_REQUEST_LABELS}'."
