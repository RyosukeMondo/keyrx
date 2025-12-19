#!/usr/bin/env python3
"""
Zenn Article CLI Tool for KeyRx
Quick lightweight CLI for managing articles
"""

import json
import subprocess
import sys
from pathlib import Path
import re

REPO_ROOT = Path(__file__).parent.parent
ARTICLES_DIR = REPO_ROOT / "articles"
ARTICLE_REGISTRY = ARTICLES_DIR / ".article-registry.json"


def load_registry():
    if not ARTICLE_REGISTRY.exists():
        return {"articles": []}
    return json.loads(ARTICLE_REGISTRY.read_text())


def list_articles():
    registry = load_registry()
    articles = registry.get("articles", [])

    if not articles:
        print("No articles found.")
        return

    print(f"\n{'Status':<12} {'Title':<50} {'Spec':<30}")
    print("-" * 92)

    for article in articles:
        status = "✓ Published" if article.get("published") else "○ Draft"
        title = article["title"][:47] + "..." if len(article["title"]) > 50 else article["title"]
        spec = article["spec"][:27] + "..." if len(article["spec"]) > 30 else article["spec"]
        print(f"{status:<12} {title:<50} {spec:<30}")

    print(f"\nTotal: {len(articles)} articles")
    print(f"Published: {sum(1 for a in articles if a.get('published'))}")
    print(f"Drafts: {sum(1 for a in articles if not a.get('published'))}\n")


def view_article(slug):
    registry = load_registry()
    article = next((a for a in registry.get("articles", []) if a["slug"] == slug), None)

    if not article:
        print(f"Article not found: {slug}")
        sys.exit(1)

    file_path = Path(article["file"])
    if not file_path.exists():
        print(f"Article file not found: {file_path}")
        sys.exit(1)

    print(file_path.read_text())


def publish_article(slug):
    registry = load_registry()
    article = next((a for a in registry.get("articles", []) if a["slug"] == slug), None)

    if not article:
        print(f"Article not found: {slug}")
        sys.exit(1)

    file_path = Path(article["file"])
    content = file_path.read_text()
    content = re.sub(r"published:\s*false", "published: true", content)
    file_path.write_text(content)

    article["published"] = True
    ARTICLE_REGISTRY.write_text(json.dumps(registry, indent=2, ensure_ascii=False))

    print(f"✓ Published: {article['title']}")


def draft_article(slug):
    registry = load_registry()
    article = next((a for a in registry.get("articles", []) if a["slug"] == slug), None)

    if not article:
        print(f"Article not found: {slug}")
        sys.exit(1)

    file_path = Path(article["file"])
    content = file_path.read_text()
    content = re.sub(r"published:\s*true", "published: false", content)
    file_path.write_text(content)

    article["published"] = False
    ARTICLE_REGISTRY.write_text(json.dumps(registry, indent=2, ensure_ascii=False))

    print(f"✓ Moved to draft: {article['title']}")


def preview_articles():
    print("Starting Zenn preview server...")
    print("Preview URL: http://localhost:8000")
    print("Press Ctrl+C to stop\n")

    try:
        subprocess.run(["npx", "zenn", "preview"], cwd=REPO_ROOT)
    except KeyboardInterrupt:
        print("\nPreview stopped.")
    except FileNotFoundError:
        print("Error: npx not found. Install Node.js or zenn-cli")
        sys.exit(1)


def show_stats():
    registry = load_registry()
    articles = registry.get("articles", [])

    print("\n=== KeyRx Zenn Article Statistics ===\n")
    print(f"Total Articles: {len(articles)}")
    print(f"Published: {sum(1 for a in articles if a.get('published'))}")
    print(f"Drafts: {sum(1 for a in articles if not a.get('published'))}")

    specs = {}
    for article in articles:
        spec = article.get("spec", "unknown")
        specs[spec] = specs.get(spec, 0) + 1

    if specs:
        print(f"\nArticles by Spec:")
        for spec, count in sorted(specs.items(), key=lambda x: -x[1]):
            print(f"  {spec}: {count}")

    total_words = 0
    for article in articles:
        file_path = Path(article.get("file", ""))
        if file_path.exists():
            content = file_path.read_text()
            words = len(content.split())
            total_words += words

    print(f"\nTotal Words: {total_words:,}")
    print(f"Average Words per Article: {total_words // len(articles) if articles else 0}\n")


def show_help():
    print("""
=== KeyRx Zenn Article CLI ===

Usage: python3 scripts/zenn-article-cli.py <command> [options]

Commands:
  list                    List all articles
  view <slug>             View article content
  publish <slug>          Publish an article
  draft <slug>            Move article to draft
  preview                 Start local Zenn preview
  stats                   Show article statistics
  help                    Show this help

Examples:
  python3 scripts/zenn-article-cli.py list
  python3 scripts/zenn-article-cli.py view ai-native-software-925-sessions
  python3 scripts/zenn-article-cli.py preview
""")


def main():
    if len(sys.argv) < 2 or sys.argv[1] == "help":
        show_help()
        return

    command = sys.argv[1]

    if command == "list":
        list_articles()
    elif command == "view" and len(sys.argv) > 2:
        view_article(sys.argv[2])
    elif command == "publish" and len(sys.argv) > 2:
        publish_article(sys.argv[2])
    elif command == "draft" and len(sys.argv) > 2:
        draft_article(sys.argv[2])
    elif command == "preview":
        preview_articles()
    elif command == "stats":
        show_stats()
    else:
        print(f"Unknown command: {command}")
        show_help()
        sys.exit(1)


if __name__ == "__main__":
    main()
