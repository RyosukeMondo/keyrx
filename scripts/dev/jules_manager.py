#!/usr/bin/env python3
"""
Jules Session Racer
-------------------
Autonomous supervisor for multiple parallel Jules sessions.
Watches multiple sessions for the SAME task. The first one to succeed wins.
- Applies the winner's patch.
- Commits and pushes the changes (optional).
- Triggers the chained command.
- Ignores the slower/failed sessions.

Usage:
  python3 jules_manager.py <ID1> [ID2] [ID3] ... --next-cmd "..."
"""

import subprocess
import time
import sys
import re
import argparse
import logging

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s [%(levelname)s] %(message)s',
    datefmt='%H:%M:%S'
)
logger = logging.getLogger("jules_racer")

def run_command(cmd, exit_on_error=True, capture_output=True, log_output=True):
    """Run a shell command and return stdout."""
    logger.debug(f"Exec: {cmd}")
    try:
        use_shell = isinstance(cmd, str)
        result = subprocess.run(
            cmd, 
            shell=use_shell, 
            check=True, 
            stdout=subprocess.PIPE if capture_output else None, 
            stderr=subprocess.PIPE if capture_output else None, 
            text=True
        )
        if log_output and capture_output and result.stdout:
            logger.debug(f"Stdout: {result.stdout.strip()}")
        return result.stdout.strip() if capture_output else ""
    except subprocess.CalledProcessError as e:
        logger.error(f"Command failed: {cmd}")
        if capture_output and e.stderr:
            logger.error(f"Stderr: {e.stderr.strip()}")
        if exit_on_error:
            sys.exit(1)
        return None

def get_session_status(session_id):
    """Parse jules remote list to find status of a specific session."""
    output = run_command("jules remote list --session", exit_on_error=False)
    if not output:
        return "UNKNOWN"
    
    for line in output.split('\n'):
        if session_id in line:
            match = re.search(r"(Running|Complete|Succeeded|Awaiting|Failed|Error|Created|Queued)\s*$", line, re.IGNORECASE)
            if match:
                return match.group(1).upper()
            
    return "UNKNOWN"

def get_session_description(session_id):
    """Parses jules remote list to find the description of a specific session."""
    output = run_command("jules remote list --session", exit_on_error=False)
    if not output:
        return f"Jules session {session_id} commit"
    
    for line in output.split('\n'):
        if session_id in line:
            # Regex to capture description between ID and Repo columns
            match = re.search(fr"{session_id}\s*(.*?)\s*RyosukeMondo/keyrx", line)
            if match:
                desc = match.group(1).strip()
                if desc.startswith("Task: "):
                    desc = desc[len("Task: "):]
                return f"feat: {desc} (jules-{session_id})"
            
    return f"feat: Jules session {session_id} commit"

def extract_session_id(output):
    """Extracts ID: <number> from jules output."""
    if not output:
        return None
    match = re.search(r"ID:\s*(\d+)", output)
    if match:
        return match.group(1)
    return None

def main():
    parser = argparse.ArgumentParser(description="Race multiple Jules sessions.")
    parser.add_argument("session_ids", nargs='+', help="List of Jules session IDs to watch")
    parser.add_argument("--interval", type=int, default=30, help="Polling interval in seconds")
    parser.add_argument("--no-apply", action="store_true", help="Disable auto-apply")
    parser.add_argument("--next-cmd", help="Command to run after the winner finishes.")
    parser.add_argument("--no-commit-push", action="store_true", help="Disable auto-commit and push after applying")
    
    args = parser.parse_args()
    
    active_sessions = set(args.session_ids)
    failed_sessions = set()
    winner = None
    
    logger.info(f"Racing sessions: {active_sessions}")

    while not winner and len(failed_sessions) < len(args.session_ids):
        for session_id in list(active_sessions):
            if session_id in failed_sessions:
                continue

            status = get_session_status(session_id)
            
            if status in ["COMPLETE", "SUCCEEDED", "AWAITING"]:
                logger.info(f"🏆 WINNER: Session {session_id} finished with status {status}")
                winner = session_id
                break
            
            elif status in ["FAILED", "ERROR"]:
                logger.warning(f"Session {session_id} failed. Ignoring.")
                failed_sessions.add(session_id)
            
            else:
                pass
        
        if winner:
            break
            
        if len(failed_sessions) == len(args.session_ids):
            logger.error("All sessions failed.")
            sys.exit(1)

        time.sleep(args.interval)

    # Handle Winner
    if winner:
        if not args.no_apply:
            logger.info(f"Auto-applying changes from winner {winner}...")
            run_command(f"jules remote pull --session {winner} --apply", exit_on_error=True)
            logger.info("Changes applied.")

        if not args.no_commit_push:
            logger.info("Committing and pushing changes...")
            commit_message = get_session_description(winner)
            run_command(f"git add .") 
            run_command(f"git commit -m \"{commit_message}\"", exit_on_error=True)
            run_command(f"git push", exit_on_error=True)
            logger.info("Changes committed and pushed.")
        
        if args.next_cmd:
            logger.info(f"Triggering chained command: {args.next_cmd}")
            output = run_command(args.next_cmd, exit_on_error=True)
            logger.info(f"Command output:\n{output}")
            
            new_id = extract_session_id(output)
            if new_id:
                logger.info(f"Next task started with Session ID: {new_id}")
            else:
                logger.info("Next command finished.")
        
        logger.info("Workflow completed successfully.")
        sys.exit(0)
    else:
        logger.error("No winner found.")
        sys.exit(1)

if __name__ == "__main__":
    main()