# Demo Recordings

This directory stores recorded demo sessions for the 2HAL9 MVP.

## File Format

Recordings are saved as JSON files with the naming pattern:
```
demo_{scenario}_{timestamp}.json
```

Example:
```
demo_task_management_web_application_20240115_143022.json
```

## Usage

### Record a new demo
```bash
../record-demo.sh
```

### Replay existing recording
```bash
../replay-demo.sh
```

### Direct replay
```bash
cargo run --release -p hal9_mvp -- --replay=demo_task_management_web_application_20240115_143022.json
```

## Tips

- Keep your best recordings for presentations
- Use git to version control important demos
- Recordings work across platforms (JSON format)
- Variable speed playback available during replay