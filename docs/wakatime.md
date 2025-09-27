# WakaTime Integration

Helix supports integration with [WakaTime](https://wakatime.com/), a time tracking service for programmers. This allows you to automatically track the time you spend coding in different files, projects, and languages.

## Configuration

To enable WakaTime tracking, add a `[editor.wakatime]` section to your `config.toml` file:

```toml
[editor.wakatime]
enabled = true
api-key = "your-wakatime-api-key-here"
```

### Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | boolean | `false` | Enable or disable WakaTime tracking |
| `api-key` | string | `None` | Your WakaTime API key for authentication |
| `api-url` | string | `"https://api.wakatime.com/api/v1/users/current/heartbeats"` | Custom WakaTime API URL |
| `project` | string | `None` | Override project name for all files |
| `hide-file-names` | boolean | `false` | Hide file names from WakaTime (sends "HIDDEN" instead) |
| `hide-project-names` | boolean | `false` | Hide project names from WakaTime |
| `timeout` | integer | `30` | Timeout for WakaTime API requests in seconds |

### Complete Configuration Example

```toml
[editor.wakatime]
enabled = true
api-key = "waka_12345678-1234-1234-1234-123456789012"
api-url = "https://api.wakatime.com/api/v1/users/current/heartbeats"
project = "my-custom-project-name"
hide-file-names = false
hide-project-names = false
timeout = 30
```

## Getting Your API Key

1. Sign up for a free account at [wakatime.com](https://wakatime.com/)
2. Go to your [Settings page](https://wakatime.com/settings/account)
3. Copy your API key from the "API Key" section
4. Paste it into your Helix config file

## Project Detection

Helix automatically detects projects by looking for common project root indicators:

- `.git` directory (Git repository)
- `.hg` directory (Mercurial repository)  
- `.svn` directory (Subversion repository)
- `Cargo.toml` (Rust project)
- `package.json` (Node.js project)
- `pyproject.toml` (Python project)

If you want to override the detected project name, use the `project` configuration option.

## Tracked Events

WakaTime will track the following activities in Helix:

- **File opens**: When you open a file
- **File edits**: When you make changes to a file
- **Cursor movement**: When you move the cursor or change selections

## Privacy

If you're concerned about privacy, you can use the following options:

- `hide_file_names = true`: Sends "HIDDEN" instead of actual file names
- `hide_project_names = true`: Doesn't send project names to WakaTime

## Troubleshooting

### WakaTime not working

1. Check that `enabled = true` in your config
2. Verify your API key is correct
3. Check the Helix logs for WakaTime-related errors:
   ```bash
   hx --log-file helix.log
   # Then check helix.log for WakaTime messages
   ```

### High network usage

If you're experiencing high network usage, consider:

- Increasing the `timeout` value to reduce failed requests
- Using WakaTime's offline mode (if supported)

### Self-hosted WakaTime

For self-hosted WakaTime instances, change the `api_url` to point to your server:

```toml
[editor.wakatime]
enabled = true
api-key = "your-api-key"
api-url = "https://your-wakatime-server.com/api/v1/users/current/heartbeats"
```