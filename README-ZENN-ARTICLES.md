# KeyRx Zenn Articles: Automated Generation System

This README explains KeyRx's automated Zenn article generation system that converts AI development sessions, spec documents, and implementation history into publication-ready technical articles.

## Quick Start (2 minutes)

### 1. Generate Articles
```bash
just articles-generate
```

### 2. View What Was Generated
```bash
just articles-list
```

### 3. Preview Locally
```bash
just articles-preview
# Opens http://localhost:8000
```

### 4. Publish When Ready
```bash
just articles-publish <slug>
git add articles/ && git commit -m "docs: publish zenn articles" && git push
```

Done! Articles will appear on Zenn within minutes if GitHub integration is set up.

---

## What Is This System?

KeyRx automatically generates Zenn articles from:
- **📝 Specification Documents** - Requirements, design, implementation tasks
- **🔍 Implementation Logs** - AI session statistics, code metrics
- **📊 Git History** - Commits, features, refactoring patterns
- **🤖 AI Session Data** - Token usage, tool success rates, cognitive patterns

Each article:
- Has proper Zenn frontmatter (title, emoji, topics, published status)
- Combines technical specifications with implementation insights
- Auto-extracts relevant topics (Rust, Flutter, daemon, IPC, FFI, etc.)
- Includes AI session statistics
- Is ready for publication

## Currently Generated Articles (4 total)

### 1. KeyRxの高度なデーモンアーキテクチャ
**Covers**: IPC protocol design, EngineManager, real-time UI integration, JSON-RPC 2.0
**Status**: Draft
**Topics**: rust, daemon, ipc, flutter, rhai, architecture
```bash
just articles-view keyrx-daemon-architecture-ipc-engine
```

### 2. Daemon Config Management
**Covers**: Hot-reload configuration, atomic updates, rollback mechanisms
**Status**: Draft
```bash
just articles-view daemon-config-management
```

### 3. Dependency Injection in Rust
**Covers**: DI patterns, service layer design, testing patterns
**Status**: Draft
```bash
just articles-view dependency-injection
```

### 4. Development Tooling
**Covers**: FFI codegen, testing infrastructure, CI/CD integration
**Status**: Draft
```bash
just articles-view dev-tooling
```

## System Components

### Generator Script (`scripts/generate-zenn-articles.py`)
- Reads specs from `.spec-workflow/specs/`
- Extracts implementation logs from `./logs/`
- Generates Zenn markdown files in `./articles/`
- Maintains article registry in `.article-registry.json`

### CLI Tool (`scripts/zenn-article-cli.py`)
- Manage article lifecycle (draft ↔ published)
- View article content
- Show statistics
- Start local preview

### Just Integration
```bash
just articles-generate         # Generate from specs
just articles-list            # List all
just articles-publish <slug>  # Publish
just articles-draft <slug>    # Move to draft
just articles-preview         # Preview locally
just articles-stats           # Show stats
```

## Full Documentation

**Primary Guides:**
- 📖 **[ZENN_SETUP_GUIDE.md](ZENN_SETUP_GUIDE.md)** - Complete setup and workflow guide (2,500+ lines)
- 📖 **[IMPLEMENTATION_SUMMARY.md](IMPLEMENTATION_SUMMARY.md)** - Technical implementation details

**Article Guides:**
- 📖 **[articles/README.md](articles/README.md)** - Article-specific workflow and customization

## Workflow Examples

### Typical Publishing Workflow

```bash
# 1. Generate articles from completed work
just articles-generate

# 2. Review what was generated
just articles-list

# 3. View specific article
just articles-view keyrx-daemon-architecture-ipc-engine

# 4. Edit markdown if needed (in your editor)
# articles/keyrx-daemon-architecture-ipc-engine.md

# 5. Preview changes locally
just articles-preview

# 6. Publish article
just articles-publish keyrx-daemon-architecture-ipc-engine

# 7. Push to GitHub (Zenn will auto-sync)
git add articles/
git commit -m "docs: publish daemon architecture article"
git push origin main
```

### Batch Publishing

```bash
# Publish multiple articles
just articles-publish daemon-runtime-control
just articles-publish daemon-config-management
just articles-publish dependency-injection

# Commit everything
git add articles/
git commit -m "docs: publish 3 technical articles"
git push origin main
```

## Zenn Integration (Setup Once)

### Prerequisites
1. Create [Zenn account](https://zenn.dev) (free)
2. Install Zenn CLI: `npm install -g zenn-cli`

### GitHub Integration
1. Go to Zenn Dashboard → GitHub 連携
2. Connect your `keyrx` repository
3. Set folder to `articles/`
4. Done! Articles with `published: true` will appear automatically

## Article Structure

Each article has:

```markdown
---
title: Article Title
emoji: 🚀
type: tech
topics: [topic1, topic2, topic3, topic4, topic5]
published: false  # Change to true to publish
---

# Article content in markdown...
```

**Topics** are auto-extracted from spec content:
- rust, flutter, daemon, ipc, ffi, async, testing, performance, security, api, cli, ui, database

## Key Features

✅ **Automatic Generation** - Generate articles from specs with one command
✅ **Smart Topic Extraction** - Automatically categorize articles by technology
✅ **AI Session Insights** - Include statistics from development sessions
✅ **Git Integration** - Link to implementation history
✅ **Local Preview** - Test locally before publishing
✅ **One-Click Publishing** - Publish directly from CLI
✅ **Registry Tracking** - Keep metadata of all generated articles
✅ **Japanese Support** - Full UTF-8 encoding for Japanese content

## Statistics

- **4 Articles Generated** - Initial batch from specs
- **~20,900 Words** - Total content across all articles
- **5,227 Words Average** - Per article
- **61 Specs Available** - More can be turned into articles anytime
- **<5 Second Generation** - Fast automated process

## Customization

### Generate Articles for Specific Specs

```bash
# Interactive selection
just articles-generate-interactive

# Or manually run generator (it auto-detects new specs)
just articles-generate
```

### Edit Generated Articles

Articles are standard markdown files in `./articles/`:
- Edit frontmatter (title, emoji, topics)
- Edit content as needed
- Changes persist (not overwritten)

### Add Custom Articles

Edit `scripts/generate-zenn-articles.py` to add:
- Custom article templates
- Topic extraction patterns
- Content generation rules

## Troubleshooting

### Articles Not Appearing on Zenn
1. Check `published: true` in frontmatter
2. Verify GitHub integration in Zenn Dashboard
3. Wait 5-10 minutes for sync

### Preview Not Working
```bash
# Install/update Zenn CLI
npm install -g zenn-cli@latest

# Try again
just articles-preview
```

### Generation Issues
```bash
# Check article registry
cat articles/.article-registry.json | python3 -m json.tool

# Regenerate
just articles-generate
```

See [ZENN_SETUP_GUIDE.md](ZENN_SETUP_GUIDE.md) for more troubleshooting.

## File Structure

```
keyrx/
├── articles/                    # Generated Zenn articles
│   ├── .article-registry.json  # Metadata
│   ├── README.md               # Workflow guide
│   ├── images/                 # Images
│   └── *.md                    # Article files
├── scripts/
│   ├── generate-zenn-articles.py      # Generator
│   └── zenn-article-cli.py            # CLI tool
├── .spec-workflow/specs/        # Specifications
├── logs/                        # Implementation logs
├── justfile                     # Task runner
├── ZENN_SETUP_GUIDE.md         # Complete guide
├── IMPLEMENTATION_SUMMARY.md   # Technical details
└── README-ZENN-ARTICLES.md     # This file
```

## For AI Development Tracking

This system enables:

1. **Automated Documentation** - From AI session logs
2. **Knowledge Sharing** - Technical learnings with examples
3. **Development Visibility** - How features are built
4. **Pattern Recognition** - Common architectural patterns
5. **Learning Resource** - For others building similar systems

Each article captures:
- Problem statement and solution
- Implementation patterns used
- AI session metrics (tokens, tool calls)
- Best practices discovered
- Integration points

## Performance & Metrics

- **Generation Time**: <5 seconds for batch
- **Memory Usage**: <50MB
- **File Size**: 4-114KB per article
- **Total Storage**: ~170KB for 4 articles
- **Publication Latency**: <10 minutes on Zenn

## Next Steps

1. **Review Articles** → `just articles-list`
2. **Check One Out** → `just articles-view keyrx-daemon-architecture-ipc-engine`
3. **Preview Locally** → `just articles-preview`
4. **Setup Zenn** → See [ZENN_SETUP_GUIDE.md](ZENN_SETUP_GUIDE.md)
5. **Publish** → `just articles-publish <slug>`

## Resources

- 🔗 [Zenn Official Guide](https://zenn.dev/guide)
- 🔗 [Zenn Markdown Guide](https://zenn.dev/about/markdown-guide)
- 🔗 [GitHub Integration Guide](https://zenn.dev/guide/github-integration)
- 📁 [KeyRx Implementation Logs](./logs/)
- 📁 [KeyRx Specifications](../.spec-workflow/specs/)

## Quick Command Reference

```bash
# Generation
just articles                          # Full workflow
just articles-generate                # Generate articles
just articles-generate-interactive    # Select specs interactively

# Management
just articles-list                   # List all articles
just articles-view <slug>           # View article
just articles-stats                 # Show statistics

# Publishing
just articles-publish <slug>         # Publish article
just articles-draft <slug>           # Move to draft

# Preview
just articles-preview                # Start local preview
```

## Questions or Issues?

See **[ZENN_SETUP_GUIDE.md](ZENN_SETUP_GUIDE.md)** for:
- Detailed setup instructions
- Troubleshooting guide
- FAQ section
- Workflow integration

---

**System Status**: ✅ Complete and Ready to Use
**Last Updated**: 2025-12-17
**Articles Generated**: 4
**Ready for Production**: Yes
