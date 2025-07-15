# MCP Logseq Server (Rust)

A Rust implementation of an MCP (Model Context Protocol) server for Logseq, enabling AI assistants like Claude to interact with your Logseq knowledge graph through Logseq's Local HTTP API.

## Features

### Query Operations
- **list_graphs**: List available Logseq graphs
- **list_pages**: List all pages in the current graph
- **get_page**: Retrieve content of a specific page by name
- **get_block**: Get a specific block by its UUID
- **search**: Search across all pages in the graph

### Write Operations
- **create_page**: Create a new page with optional content
- **update_block**: Update the content of an existing block
- **insert_block**: Insert a new block as child or sibling
- **delete_block**: Delete a block by its UUID
- **append_to_page**: Append content to the end of a page

## Prerequisites

1. **Logseq** with HTTP API server enabled:
   - Open Logseq
   - Go to Settings → Features
   - Toggle on "HTTP APIs server"
   - Click the API option in the top menu
   - Set up an authorization token
   - Enable "Auto start server when Logseq launches"

2. **Rust** (latest stable version)

## Installation

1. Clone this repository:
   ```bash
   git clone <repository-url>
   cd mcp-logseq-rust
   ```

2. Copy the example environment file:
   ```bash
   cp .env.example .env
   ```

3. Edit `.env` and add your Logseq API token:
   ```env
   LOGSEQ_API_TOKEN=your-actual-token-here
   LOGSEQ_API_URL=http://localhost:12315
   ```

4. Build the project:
   ```bash
   cargo build --release
   ```

## Usage

### Running the Server

```bash
cargo run --release
```

Or use the compiled binary:
```bash
./target/release/mcp-logseq-rust
```

### Configuring with Claude Desktop

Add the following to your Claude Desktop configuration file:

**On macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
**On Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "logseq": {
      "command": "/path/to/mcp-logseq-rust/target/release/mcp-logseq-rust",
      "env": {
        "LOGSEQ_API_TOKEN": "your-logseq-api-token",
        "LOGSEQ_API_URL": "http://localhost:12315"
      }
    }
  }
}
```

Replace `/path/to/mcp-logseq-rust` with the actual path to your project directory.

## Example Usage in Claude

Once configured, you can interact with your Logseq graph through Claude:

```
"Can you show me all pages in my Logseq graph?"
"Search for notes about 'project planning'"
"Create a new page called 'Meeting Notes 2024-01-15'"
"Add a task to my Daily Notes page"
```

## Development

### Project Structure

```
src/
├── main.rs           # Server entry point
├── config.rs         # Configuration handling
├── logseq_client.rs  # HTTP client for Logseq API
├── models.rs         # Data structures
└── tools/            # MCP tool implementations
    ├── mod.rs        # Tool definitions
    ├── query.rs      # Read operations
    └── mutate.rs     # Write operations
```

### Running Tests

```bash
cargo test
```

### Debugging

Set the `RUST_LOG` environment variable for more detailed logging:

```bash
RUST_LOG=debug cargo run
```

## Security Notes

- Keep your Logseq API token secure and never commit it to version control
- The `.env` file is gitignored by default
- Consider using environment-specific tokens for different environments

## Troubleshooting

### "LOGSEQ_API_TOKEN not set" error
Make sure you've created a `.env` file with your token or set the environment variable directly.

### Connection refused errors
1. Ensure Logseq is running
2. Verify the HTTP API server is enabled in Logseq settings
3. Check that the API URL matches your Logseq configuration (default: http://localhost:12315)

### Authentication errors
Double-check that your API token in the `.env` file matches the one configured in Logseq.

## License

[Your chosen license]

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.