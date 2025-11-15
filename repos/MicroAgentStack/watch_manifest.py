import time
from watchdog.observers import Observer
from watchdog.events import FileSystemEventHandler
import subprocess

# List all the files you want to watch here:
WATCH_FILES = ["updated_agent_manifest.json", "another_config.json"]  # Add as many as needed
SCRIPT_TO_RUN = "full_auto_agents.py"   # The automation script to run

class ManifestChangeHandler(FileSystemEventHandler):
    def on_modified(self, event):
        if any(event.src_path.endswith(f) for f in WATCH_FILES):
            print(f"\n{event.src_path} changed. Running automation script...\n")
            subprocess.run(["python3", SCRIPT_TO_RUN])

if __name__ == "__main__":
    event_handler = ManifestChangeHandler()
    observer = Observer()
    observer.schedule(event_handler, path=".", recursive=False)
    print(f"Watching these files for changes: {WATCH_FILES}. Press Ctrl+C to stop.")
    observer.start()
    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        observer.stop()
    observer.join()
