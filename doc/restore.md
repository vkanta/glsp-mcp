# Restore instructions for amt_uml_diagram checkpoint

This file explains how to restore the repository to the checkpoint created for
the `amt_uml_diagram` work. A checkpoint tag was created and pushed named:

```
checkpoint/amt_uml_diagram-20250906_225847Z
```

Safe restore (create a new branch from the checkpoint):

```bash
# fetch tags from origin (safe)
git fetch origin --tags

# create a new branch from the checkpoint tag without touching current branches
git checkout -b restore-checkpoint/amt_uml_diagram-20250906_225847Z \
    checkpoint/amt_uml_diagram-20250906_225847Z

# You now have a branch `restore-checkpoint/amt_uml_diagram-20250906_225847Z`
# containing the exact commit saved by the checkpoint tag. Inspect it, run
# tests, and merge or cherry-pick as needed.
```

Destructive restore (reset current branch to the checkpoint)

```bash
# WARNING: this will discard local commits that are not referenced elsewhere.
git fetch origin --tags
git checkout amt_uml_diagram
git reset --hard checkpoint/amt_uml_diagram-20250906_225847Z

# Optionally push the reset branch to origin (force push):
# git push --force origin amt_uml_diagram
```

Notes
- The tag `checkpoint/amt_uml_diagram-20250906_225847Z` is pushed to `origin`.
- Prefer the safe restore branch method unless you intentionally want to
  overwrite branch history.
- If you want me to create a PR from the restore branch or to perform the
  reset for you, I can do that next.
