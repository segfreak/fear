#/usr/bin/bash

CREATED_TIME=$(git log --reverse --format="%at" | head -n 1)
CURRENT_TIME=$(date +%s)
DIFF_DAYS=$(( (CURRENT_TIME - CREATED_TIME) / 86400 ))

LAST_COMMIT_DATE=$(git log -1 --format="%cd")
LAST_COMMIT_RELATIVE=$(git log -1 --format="%cr")
SHORT_HASH=$(git log -1 --format="%h")

echo "Last commit: $LAST_COMMIT_DATE"
echo "           : $LAST_COMMIT_RELATIVE"
echo "           : $SHORT_HASH"

echo "The repository has been active for: $DIFF_DAYS days"