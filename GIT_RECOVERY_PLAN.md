# Git Repository Recovery Plan

## CRITICAL ISSUE IDENTIFIED

During the HA reorganization, the `.git` folder was moved from the root directory to `.substrate/version-control/git`. This has effectively broken the Git repository.

## What Happened

1. The migration script moved `.git` → `.substrate/version-control/git`
2. Git requires `.git` to be in the repository root to function
3. The repository is now "detached" and Git commands won't work
4. We cannot commit any of the reorganization changes

## Current State

- **Location of .git**: `.substrate/version-control/git`
- **Repository status**: Broken (Git doesn't recognize it)
- **Risk level**: HIGH - We could lose work or corrupt history

## Recovery Plan

### Option 1: Simple Move Back (RECOMMENDED)
**Risk**: Low  
**Complexity**: Simple  
**Steps**:
1. Move `.substrate/version-control/git` back to `.git`
2. Move `.substrate/version-control/.gitignore` back to `.gitignore`
3. Verify Git functionality
4. Stage and commit the reorganization changes

### Option 2: Symlink Approach
**Risk**: Medium  
**Complexity**: Medium  
**Steps**:
1. Create a symlink from `.git` → `.substrate/version-control/git`
2. Test if Git recognizes the symlink
3. If not, revert to Option 1

### Option 3: Fresh Clone + Apply Changes
**Risk**: High (could lose uncommitted work)  
**Complexity**: High  
**Steps**:
1. Clone fresh repository elsewhere
2. Copy reorganized structure over
3. Very risky - NOT recommended

## Immediate Action Required

We should execute Option 1 immediately to restore Git functionality:

```bash
# Move .git back to root
mv .substrate/version-control/git .git

# Move .gitignore back to root
mv .substrate/version-control/.gitignore .gitignore

# Verify Git is working
git status

# Stage all reorganization changes
git add -A

# Create a comprehensive commit
git commit -m "PARADIGM: Complete HA reorganization of codebase

Level: L9 Universal + All Levels
Scope: Fundamental restructuring

- Reorganized entire codebase by cognitive levels (L1-L9)
- Eliminated forced context switching
- Each role now has a cognitive home
- Infrastructure moved to substrate/
- Technical artifacts hidden in .substrate/

This transforms the codebase from computer-centric to human-centric organization."
```

## Lessons Learned

1. **Never move .git** - It must stay in the root
2. **Critical infrastructure** should be excluded from reorganization
3. **Test with a small repo first** before major reorganizations

## Post-Recovery Actions

After restoring Git:
1. Update `.gitignore` to properly handle new structure
2. Document that `.git` must never be moved
3. Add pre-commit hooks to prevent future issues
4. Create a backup branch before any further changes