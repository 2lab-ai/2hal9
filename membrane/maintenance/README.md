# HAL9 Maintenance Tools

## 🛠️ Purpose
This directory contains meta-level tools for maintaining HAL9's hierarchical integrity. These tools operate across all levels and ensure the system remains true to HA principles.

## 📋 Available Tools

### 1. FIX_BROKEN_LINKS_PROMPT.md
- **Purpose**: Fix broken links after restructuring
- **Scope**: L9 → L1 cascade
- **When to use**: After any major file reorganization

### 2. HA_PRINCIPLE_AUDIT_PROMPT.md
- **Purpose**: Detect and fix HA principle violations
- **Scope**: Full system audit
- **When to use**: Weekly or after major updates

## 🔄 Maintenance Schedule

### Daily
- Quick link check in active development areas

### Weekly
- Full HA principle audit
- Report generation

### Monthly  
- Deep structural review
- Entropy measurement
- Process refinement

## 🚀 Quick Start

```bash
# For broken links
cat membrane/maintenance/FIX_BROKEN_LINKS_PROMPT.md | claude

# For HA audit
cat membrane/maintenance/HA_PRINCIPLE_AUDIT_PROMPT.md | claude
```

## 📊 Why in membrane/?

The membrane is the semi-permeable boundary between HAL9's internal structure and external world. Maintenance tools naturally belong here because they:

1. Operate across all levels (not confined to one)
2. Maintain system integrity (membrane function)
3. Filter what enters the system (quality control)
4. Are meta-tools (about the system, not in it)

## 🎯 Philosophy

> "A system without maintenance is a system in decay."

These tools prevent the natural entropy that occurs when multiple developers work across multiple abstraction levels. They are the immune system of HAL9.

---

*Remember: Hierarchy is not a suggestion—it's the architecture!*