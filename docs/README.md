# Documentation

This directory contains all KeyRx2 project documentation organized by audience and purpose.

## Structure

```
docs/
├── user-guide/          # End-user documentation
│   ├── dsl-manual.md    # Complete DSL reference
│   ├── linux-setup.md   # Linux installation and setup
│   ├── macos-setup.md   # macOS installation and setup
│   ├── multi-device-configuration.md  # Multi-device setup guide
│   ├── setup.md         # General setup instructions
│   └── uat-guide.md     # User acceptance testing guide
├── development/         # Developer documentation
│   ├── architecture.md  # System architecture and design decisions
│   └── design/          # Detailed design documents
│       └── recursive-data-structures.md
├── rfcs/                # Proposals and specifications
│   └── keyboard-internationalization.md
├── research/            # Research notes and investigations
│   └── research.md
└── archive/             # Historical documents
    ├── next-steps.md
    └── violations-fixed.md
```

## Documentation Categories

### User Guide (`user-guide/`)
Documentation for end users who want to install and use KeyRx2:
- Installation and setup instructions
- DSL language reference
- Configuration examples
- Testing and validation guides

### Development (`development/`)
Documentation for contributors and developers:
- Architecture overview
- Design decisions and rationale
- Implementation details
- Coding standards and patterns

### RFCs (`rfcs/`)
Request for Comments - proposals for new features or significant changes:
- Feature proposals
- API design documents
- Breaking changes

### Research (`research/`)
Research notes, performance analysis, and exploratory work:
- Performance benchmarks
- Alternative approaches
- Technology evaluations

### Archive (`archive/`)
Historical documents kept for reference:
- Completed migration guides
- Resolved issues and fixes
- Deprecated documentation

## Quick Links

**Getting Started:**
- [Setup Guide](user-guide/setup.md)
- [Linux Setup](user-guide/linux-setup.md)
- [macOS Setup](user-guide/macos-setup.md)
- [Multi-Device Configuration](user-guide/multi-device-configuration.md)
- [DSL Manual](user-guide/dsl-manual.md)

**For Developers:**
- [Architecture](development/architecture.md)
- [Design Documents](development/design/)

**For Contributors:**
- See main [README.md](../README.md) for contribution guidelines
- See [.claude/CLAUDE.md](../.claude/CLAUDE.md) for AI development guide
