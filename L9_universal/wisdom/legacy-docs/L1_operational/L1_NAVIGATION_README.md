# L1: Operational Documentation

**Abstraction Level**: L1 - Daily Operations (Lowest)  
**Audience**: Operators, Support Staff, End Users, Anyone Who Needs Help NOW

## Welcome to Operational Level

This level contains quick commands, troubleshooting guides, and daily operational procedures. Get HAL9 running and fix problems FAST.

## Who Should Read This Level

- **System Operators** running HAL9 daily
- **Support Staff** helping users
- **New Users** getting started
- **Anyone** with HAL9 problems
- **On-Call Engineers** fixing issues at 3 AM

## What You'll Find Here

### Core Documents

All documents prefixed with L1_ including:
- Quick start guides
- Troubleshooting procedures
- Daily task checklists
- Monitoring guides
- Emergency procedures

## Key Concepts at This Level

- **Commands** - What to type RIGHT NOW
- **Fixes** - Solutions to common problems
- **Monitoring** - What to watch
- **Procedures** - Step-by-step operations
- **Quick Wins** - Get it working fast

## Operational Principles

1. **Keep It Running** - Uptime is king
2. **Fix It Fast** - Every minute counts
3. **Document Issues** - Help the next person
4. **Monitor Everything** - Catch problems early
5. **Stay Calm** - Panic helps nobody

## What NOT to Expect

- Theory or philosophy
- Architecture discussions
- Business strategy
- Code implementations
- Long explanations

## Navigation

- **Need Code?** → [L2 Implementation](../L2_implementation/)
- **Need Design?** → [L3 Design](../L3_design/)
- **Need Help NOW?** → Stay here!

## Quick Commands

```bash
# Start HAL9
./hal9-server

# Check status
curl http://localhost:8080/health

# View logs
tail -f /var/log/hal9/hal9.log

# Restart
systemctl restart hal9

# Emergency stop
pkill -9 hal9
```

## Common Problems

- **Won't Start** → Check L1_TROUBLESHOOTING.md
- **Running Slow** → Check L1_MONITORING_GUIDE.md
- **Errors** → Check logs, then L1_TROUBLESHOOTING.md
- **Need Help** → Check L1_QUICK_START.md first

## Emergency Contacts

- **Slack**: #hal9-help
- **On-Call**: See PagerDuty
- **Escalation**: ops-team@hal9.ai

## Contributing to L1

When contributing:
1. Be VERY clear
2. Use simple language
3. Include exact commands
4. Test procedures work
5. Add to troubleshooting

---

*"In case of emergency, break glass. In case of HAL9 emergency, read L1."*

**Welcome to L1 - Where Problems Get Solved**