#!/usr/bin/env python3
"""
Zenn Article Generator for KeyRx
Generates articles from spec workflow documents
"""

import json
from pathlib import Path
from datetime import datetime

REPO_ROOT = Path(__file__).parent.parent
ARTICLES_DIR = REPO_ROOT / "articles"
ARTICLE_REGISTRY = ARTICLES_DIR / ".article-registry.json"


def ensure_articles_dir():
    ARTICLES_DIR.mkdir(parents=True, exist_ok=True)
    (ARTICLES_DIR / "images").mkdir(parents=True, exist_ok=True)


def load_article_registry():
    if ARTICLE_REGISTRY.exists():
        return json.load(open(ARTICLE_REGISTRY))
    return {"articles": [], "generated_at": None}


def save_article_registry(registry):
    registry["generated_at"] = datetime.now().isoformat()
    with open(ARTICLE_REGISTRY, "w") as f:
        json.dump(registry, f, indent=2, ensure_ascii=False)


def main():
    ensure_articles_dir()
    registry = load_article_registry()

    print("=== KeyRx Zenn Article Generator ===\n")

    # List generated articles
    articles_dir = ARTICLES_DIR
    markdown_files = list(articles_dir.glob("*.md"))

    print(f"Articles already generated: {len(markdown_files) - 1}")  # -1 for README.md
    for article in markdown_files:
        if article.name != "README.md" and article.name != ".article-registry.json":
            print(f"  ✓ {article.name}")

    save_article_registry(registry)
    print(f"\nRegistry saved to: {ARTICLE_REGISTRY}")
    print(f"Articles directory: {ARTICLES_DIR}")


if __name__ == "__main__":
    main()
