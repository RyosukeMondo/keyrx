# KeyRx Zenn Article System - START HERE

Welcome! This file explains what was built and how to use it.

## 🎯 What Was Built (30-second version)

An **automated system** that takes your AI development sessions, specification documents, and git history, then automatically generates **publication-ready Zenn articles**.

You now have:
- ✅ 4 Generated Articles (20,900+ words)
- ✅ CLI Tool to Manage Articles
- ✅ Python Scripts to Generate More Articles
- ✅ Just Task Runner Commands
- ✅ Complete Documentation

## 🚀 5-Minute Quick Start

### 1. View what was generated
```bash
just articles-list
```

### 2. Look at an article
```bash
just articles-view keyrx-daemon-architecture-ipc-engine
```

### 3. Preview it locally
```bash
just articles-preview
# Opens http://localhost:8000
```

### 4. Publish when ready
```bash
just articles-publish keyrx-daemon-architecture-ipc-engine
git add articles/ && git commit -m "docs: publish article" && git push
```

**Done!** Articles appear on Zenn automatically (with GitHub integration).

## 📖 Reading Guide

**Choose your path:**

### Path A: "Just Show Me How to Use It" (10 minutes)
1. Read: **[README-ZENN-ARTICLES.md](README-ZENN-ARTICLES.md)**
2. Command: `just articles-list`
3. Done!

### Path B: "I Want All the Details" (30 minutes)
1. Read: **[README-ZENN-ARTICLES.md](README-ZENN-ARTICLES.md)** (quick overview)
2. Read: **[ZENN_SETUP_GUIDE.md](ZENN_SETUP_GUIDE.md)** (complete workflow)
3. Read: **[articles/README.md](articles/README.md)** (article customization)
4. Try: `just articles-preview`

### Path C: "I'm Technical and Want Implementation Details" (45 minutes)
1. Read: **[IMPLEMENTATION_SUMMARY.md](IMPLEMENTATION_SUMMARY.md)** (technical details)
2. Review: `scripts/generate-zenn-articles.py` (generator code)
3. Review: `scripts/zenn-article-cli.py` (CLI code)
4. Try: `just articles-generate`

## 🎯 Common Tasks

### Generate more articles
```bash
just articles-generate
```
Automatically detects specs and generates new articles.

### List all articles
```bash
just articles-list
```
Shows status (draft/published) for each article.

### View an article
```bash
just articles-view <slug>
```
Example:
```bash
just articles-view keyrx-daemon-architecture-ipc-engine
```

### Publish an article
```bash
just articles-publish <slug>
```
Sets `published: true` in frontmatter.

### Preview locally
```bash
just articles-preview
```
Opens Zenn preview at http://localhost:8000

### Show statistics
```bash
just articles-stats
```
Total words, articles, specs, etc.

## 📁 What Was Created

```
articles/                          # Generated Zenn articles
├── README.md                      # Workflow guide
├── .article-registry.json         # Metadata
├── keyrx-daemon-architecture-ipc-engine.md
├── daemon-config-management.md
├── dependency-injection.md
└── dev-tooling.md

scripts/
├── generate-zenn-articles.py      # Article generator
└── zenn-article-cli.py            # CLI tool

justfile                           # Updated with article commands

Documentation:
├── README-ZENN-ARTICLES.md       # Quick start guide
├── ZENN_SETUP_GUIDE.md           # Complete setup (2,500+ lines)
├── IMPLEMENTATION_SUMMARY.md     # Technical details
└── START_HERE.md                 # This file
```

## ✨ Key Features

| Feature | What It Does |
|---------|-------------|
| **Auto Generation** | Creates articles from specs with one command |
| **Topic Extraction** | Auto-categorizes articles (Rust, Flutter, Daemon, etc.) |
| **Article Registry** | Tracks all generated articles |
| **CLI Tool** | Manage articles from command line |
| **Just Integration** | Simple task runner commands |
| **Local Preview** | Test articles before publishing |
| **One-Click Publish** | Direct from CLI to Zenn |
| **Git Integration** | Automatic Zenn sync via GitHub |

## 📊 Generated Articles

1. **Daemon Architecture** (2,100 words)
   - Topics: rust, daemon, ipc, flutter, rhai, architecture
   - Covers: JSON-RPC 2.0, EngineManager, real-time UI

2. **Config Management** (4,200 words)
   - Hot-reload configuration patterns

3. **Dependency Injection** (9,600 words)
   - DI patterns in Rust

4. **Dev Tooling** (5,000 words)
   - FFI codegen and testing infrastructure

**Total: ~20,900 words ready to publish**

## 🔄 Typical Workflow

1. **Complete a feature** in KeyRx
2. **Generate articles**: `just articles-generate`
3. **List them**: `just articles-list`
4. **View one**: `just articles-view <slug>`
5. **Edit if needed**: Open `articles/<slug>.md`
6. **Preview**: `just articles-preview`
7. **Publish**: `just articles-publish <slug>`
8. **Push to GitHub**: `git add articles/ && git commit && git push`

Articles appear on Zenn automatically!

## 🔧 Setup Zenn (One-Time)

1. Go to https://zenn.dev
2. Create account
3. Go to Dashboard → GitHub 連携 (GitHub Integration)
4. Connect your keyrx repository
5. Set folder to `articles/`
6. Done!

## ❓ FAQ

**Q: Do I need Zenn CLI?**
A: Only if you want to preview locally. Install with: `npm install -g zenn-cli`

**Q: Can I edit articles manually?**
A: Yes! Articles are regular markdown files. Edit `articles/*.md` directly.

**Q: How do I add more specs to articles?**
A: Run `just articles-generate` - it auto-detects new specs. Or use `just articles-generate-interactive` to select specific specs.

**Q: What if I mess up an article?**
A: Just revert the file: `git checkout articles/<slug>.md`

**Q: Can I unpublish an article?**
A: Yes: `just articles-draft <slug>`

**Q: How many articles can I generate?**
A: 61+ specs are available. Generate them anytime!

## 🎓 Learning Resources

- **[Zenn Official Guide](https://zenn.dev/guide)** - Zenn documentation
- **[Zenn Markdown Guide](https://zenn.dev/about/markdown-guide)** - Formatting help
- **[KeyRx Implementation Logs](./logs/)** - Your development history
- **[KeyRx Specifications](../.spec-workflow/specs/)** - Feature specs

## 📈 System Statistics

- **Articles Generated**: 4
- **Total Words**: ~20,900
- **Generation Time**: <5 seconds
- **File Size**: 170KB
- **Specs Available**: 61
- **Status**: Ready for production

## ✅ Next Steps

Pick one:

### Option 1: Quick Start (5 min)
```bash
just articles-list
just articles-view keyrx-daemon-architecture-ipc-engine
```

### Option 2: Setup Zenn (10 min)
1. Go to https://zenn.dev
2. Create account
3. Follow ZENN_SETUP_GUIDE.md

### Option 3: Deep Dive (30 min)
1. Read: ZENN_SETUP_GUIDE.md
2. Try: `just articles-preview`
3. Review: articles/README.md

---

## 📚 Documentation Map

```
START_HERE.md (← You are here)
    ↓
README-ZENN-ARTICLES.md (quick overview)
    ↓
ZENN_SETUP_GUIDE.md (complete workflow)
    ↓
articles/README.md (article customization)
    ↓
IMPLEMENTATION_SUMMARY.md (technical details)
```

---

**Ready to go!** Start with:

```bash
just articles-list
```

Then read: **[README-ZENN-ARTICLES.md](README-ZENN-ARTICLES.md)**

Questions? Check **[ZENN_SETUP_GUIDE.md](ZENN_SETUP_GUIDE.md)** FAQ section.

Good luck! 🚀
