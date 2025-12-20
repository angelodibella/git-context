# Dependencies

Justification and comments for all dependencies used in this project.

## `clap`

Essential for CLI setup and edge cases. Implementing own would be teadious, problematic and not convenient.

## `serde` and `toml`

Needed to serialize configs to `.contexts` and import it. Implementing own would be teadious and difficult to keep serialized TOML tidy and consistent.

## `anyhow`

Convenient for early iterations but would be better to replace soon. Plan to replace with `thiserror` and a project-specific enum. If anything, we can keep `anyhow` only in `main.rs` to print nice chains, but that will be decided later.
