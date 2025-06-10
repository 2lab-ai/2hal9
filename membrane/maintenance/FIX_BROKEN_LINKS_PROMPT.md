# HAL9 Broken Links Fix - Cascading Prompt

## 🚀 Mission
Fix all broken links in HAL9 project after the HA-based directory restructuring. Start from L9 and work down to L1, updating all markdown files and ensuring navigation integrity.

## 📋 Execution Order

### Phase 1: L9_universal
```bash
# Current directory: /Users/icedac/2lab.ai/cco4_persona/p9/hal9/L9_universal
# Fix: README.md and all other .md files
# Update paths from old structure to new L[1-9]_* structure
```

### Phase 2: L8_visionary
```bash
# After completing L9, move to:
# cd ../L8_visionary
# Repeat link fixing for all .md files
```

### Phase 3: Continue Downward
```bash
# Continue pattern:
# L7_business → L6_executive → L5_strategic → L4_tactical → 
# L3_operational → L2_implementation → L1_reflexive
```

## 🔧 What to Fix

1. **Old paths to replace:**
   ```
   ../docs/           → ../L[X]_*/
   ../src/            → ../L2_implementation/
   ../config/         → ../L3_operational/
   ../tests/          → ../L2_implementation/tests/
   substrate/tooling/ → ../substrate/tooling/
   ```

2. **Common patterns:**
   ```
   [Link](../old/path/file.md) → [Link](../L[X]_category/file.md)
   [Link](./old/file.md) → [Link](./file.md) or [Link](../L[X]_category/file.md)
   ```

3. **Special cases:**
   - NAVIGATION.md links (update to reflect new structure)
   - Cross-level references (L9 referring to L1 files)
   - External docs that moved to membrane/

## 🤖 Automation Script Template

For each level, run:
```bash
# Check current broken links
find . -name "*.md" -exec grep -l "\.\./[^L][^1-9]" {} \;

# Show what needs fixing
grep -r "\.\./docs\|\.\.src\|\.\.config" *.md

# After manual review, proceed with fixes
```

## 💡 Smart Link Resolution

When fixing a broken link:
1. First check if file exists in current level directory
2. If not, check adjacent levels (±1)
3. If still not found, search in substrate/ or membrane/
4. Last resort: search entire project

## 📝 Commit Message Template

```bash
git add -A
git commit -m "fix(L[X]): Update broken links after HA restructuring

- Fixed paths in README.md
- Updated navigation links
- Corrected cross-level references
- Verified all markdown links functional"
```

## 🔄 Continuous Process

After fixing each level:
1. Run link checker to verify
2. Test navigation flow
3. Commit changes
4. Move to next lower level
5. Repeat until L1 complete

## 🎯 Success Criteria

- [ ] All .md files in each L[X] directory have working links
- [ ] NAVIGATION.md correctly maps new structure
- [ ] Cross-level references work bidirectionally
- [ ] No 404s when clicking through documentation
- [ ] README.md in root correctly introduces hierarchy

## 🚨 Important Notes

- This is a CASCADE process - complete each level before moving down
- Each level might reference files that haven't been fixed yet - note these for later
- Some files might have been deleted - remove dead links
- New structure is CANONICAL - don't revert to old paths

---

## Start Command:
```bash
cd /Users/icedac/2lab.ai/cco4_persona/p9/hal9/L9_universal
# Begin fixing README.md
```

Good luck! Remember: L9 → L8 → L7 → ... → L1 🚀