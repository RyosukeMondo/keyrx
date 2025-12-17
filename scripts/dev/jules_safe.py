#!/usr/bin/env python3
"""
Jules Safe Wrapper
------------------
A wrapper around `jules new` that enforces git synchronization safety.
Ensures that the code Jules sees (remote) matches your local intent.

Features:
1.  Checks for unpushed commits (CRITICAL: Jules clones remote).
2.  Checks for uncommitted changes (Warning).
3.  Supports redundant session spawning (--redundancy N).
4.  Auto-starts the racer supervisor (--race).

Usage:
  python3 jules_safe.py "Task description" --redundancy 3 --race --push
"""

import subprocess
import sys
import argparse
import time
import re
import os

def run_command(cmd, capture_output=True, check=True):
    """Run a shell command."""
    try:
        result = subprocess.run(
            cmd,
            shell=True,
            check=check,
            stdout=subprocess.PIPE if capture_output else None,
            stderr=subprocess.PIPE if capture_output else None,
            text=True
        )
        return result.stdout.strip() if capture_output else ""
    except subprocess.CalledProcessError as e:
        if check:
            print(f"Error executing: {cmd}")
            if capture_output:
                print(e.stderr)
            sys.exit(1)
        return None

def check_git_status(auto_push=False):
    """Ensure local state matches remote state."""
    print("🔍 Checking git state...")
    
    # 1. Check for uncommitted changes
    status = run_command("git status --porcelain", check=False)
    if status:
        print("⚠️  WARNING: You have uncommitted changes. Jules will NOT see these.")
        print(status)
        # We don't abort for dirty tree (user might be experimenting), but we warn strongly.
        time.sleep(2) # Give user a chance to Ctrl+C

    # 2. Check for unpushed commits
    # Get current branch
    branch = run_command("git rev-parse --abbrev-ref HEAD")
    
    # Check if branch has an upstream
    upstream = run_command(f"git rev-parse --abbrev-ref {branch}@{{u}}", check=False)
    
    if not upstream:
        print(f"⚠️  Branch '{branch}' has no upstream. Jules clones default branch or needs explicit repo arg.")
        # If no upstream, we assume user knows what they are doing or will handle it.
    else:
        # Check commits ahead
        ahead = run_command(f"git rev-list --count {upstream}..{branch}")
        if ahead and int(ahead) > 0:
            print(f"🛑 CRITICAL: Local branch is {ahead} commit(s) ahead of remote.")
            print("   Jules will clone the REMOTE state and miss your recent changes (e.g., specs/skeletons).")
            
            if auto_push:
                print("🚀 Auto-pushing changes...")
                run_command("git push")
                print("✅ Pushed.")
            else:
                print("❌ Aborting. Use --push to auto-push or push manually.")
                sys.exit(1)
        else:
            print("✅ Local branch is synced with remote.")

def spawn_session(prompt):
    """Spawn a single Jules session and return its ID."""
    print(f"🤖 Spawning session: {prompt[:50]}...")
    # Escape double quotes in prompt for shell safety if needed, 
    # though subprocess.run shell=True handles it if we are careful with f-strings.
    # Better to escape quotes in the prompt string itself for the jules command.
    safe_prompt = prompt.replace('"', '\"')
    
    output = run_command(f'jules new "{safe_prompt}"')
    
    # Extract ID
    match = re.search(r"ID:\s*(\d+)", output)
    if match:
        session_id = match.group(1)
        print(f"   -> Started: {session_id}")
        return session_id
    else:
        print("❌ Failed to parse Session ID from output:")
        print(output)
        return None

def main():
    parser = argparse.ArgumentParser(description="Safe Jules Session Spawner")
    parser.add_argument("prompt", help="The task description for Jules")
    parser.add_argument("--redundancy", "-r", type=int, default=1, help="Number of redundant sessions to spawn")
    parser.add_argument("--push", action="store_true", help="Automatically push unpushed commits before spawning")
    parser.add_argument("--race", action="store_true", help="Automatically start the jules_manager.py racer")
    parser.add_argument("--auto-apply", action="store_true", help="Pass --auto-apply to the racer")
    parser.add_argument("--auto-commit", action="store_true", help="Pass auto-commit-push behavior to the racer")
    
    args = parser.parse_args()

    # 1. Safety Checks
    check_git_status(auto_push=args.push)

    # 2. Spawn Sessions
    session_ids = []
    for i in range(args.redundancy):
        sid = spawn_session(args.prompt)
        if sid:
            session_ids.append(sid)
    
    if not session_ids:
        print("❌ No sessions started.")
        sys.exit(1)

    print(f"\n✅ Successfully spawned {len(session_ids)} sessions: {session_ids}")

    # 3. Start Racer (Optional)
    if args.race:
        manager_script = os.path.join(os.path.dirname(__file__), "jules_manager.py")
        if not os.path.exists(manager_script):
            print(f"❌ Error: Racer script not found at {manager_script}")
            sys.exit(1)

        cmd = [sys.executable, manager_script] + session_ids
        if args.auto_apply:
            cmd.append("--auto-apply")
        if not args.auto_commit: # Note: manager expects --no-commit-push to DISABLE it. Default is ENABLEd in my previous manager code?
            # Wait, let's check jules_manager.py arguments.
            # parser.add_argument("--no-commit-push", action="store_true")
            # So default is to commit-push.
            # If user wants auto-commit, we do nothing.
            # If user does NOT want auto-commit, we pass --no-commit-push.
            pass
        else:
             # Logic inversion: My wrapper arg is --auto-commit (opt-in for safety in wrapper?)
             # The manager script defaults to COMMIT unless --no-commit-push is passed.
             # I should align them.
             # Let's say: Wrapper defaults to manual (safe).
             # So if args.auto_commit is False, we pass --no-commit-push to manager.
             pass
        
        # Let's refine the logic:
        # Manager default: Commits.
        # Wrapper default: Should probably correspond to manager.
        # But for "Safe" wrapper, maybe we default to manual apply?
        
        # Args logic:
        # wrapper --auto-commit -> manager (default)
        # wrapper (no arg) -> manager --no-commit-push
        
        if not args.auto_commit:
            cmd.append("--no-commit-push")

        print(f"🏎️  Starting Racer: {' '.join(cmd)}")
        
        # Execute racer replacing current process, or in background?
        # Replacing is better for CLI usage so we see logs.
        # But if we want it backgrounded, user should run wrapper with &.
        # I'll use subprocess to run it and stream output.
        subprocess.run(cmd)

if __name__ == "__main__":
    main()
