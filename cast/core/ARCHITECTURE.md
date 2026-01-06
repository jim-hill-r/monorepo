# Cast Core Architecture

## Command Pattern

### Current Implementation: Factory Function with Exhaustive Matching

The `command_factory.rs` module uses a factory function with exhaustive pattern matching to create command instances. This approach was chosen over a command registry pattern for the following reasons:

#### Why Not Use a Registry Pattern?

A command registry pattern (where commands register themselves in a HashMap or similar) was considered but rejected because:

1. **Type Safety**: Rust's exhaustive pattern matching ensures all command variants are handled at compile time. A registry would require runtime lookups and lose this guarantee.

2. **Performance**: The current approach uses compile-time optimized dispatch (typically compiled to a jump table). A registry would require HashMap lookups and additional runtime overhead.

3. **Simplicity**: The factory function provides a single, clear location where all commands are instantiated. A registry would scatter this logic across multiple files.

4. **Static Nature**: All commands are known at compile time. There's no need for dynamic command registration.

5. **Idiomatic Rust**: Exhaustive pattern matching on enums is the idiomatic Rust way to handle this scenario. Registries are more common in dynamically-typed languages.

6. **Maintainability**: When adding a new command, the compiler forces you to handle it in the factory function. With a registry, forgotten registrations would be runtime errors.

#### Current Architecture Benefits

- **Single Responsibility**: Each command module (`commands/build.rs`, `commands/test.rs`, etc.) implements only the `Command` trait
- **Clear Separation**: `args.rs` handles CLI parsing, `command_factory.rs` handles instantiation, command modules handle execution
- **Type-Driven Development**: The `Commands` enum drives the entire flow, ensuring consistency
- **Easy Testing**: Each command can be tested independently, and the factory has comprehensive tests

#### When to Consider a Registry

A registry pattern would be appropriate if:
- Commands needed to be loaded dynamically from plugins
- Third-party code needed to add commands without modifying core code
- The number of commands grew to hundreds (making the match statement unwieldy)

None of these conditions apply to the Cast CLI.

### Adding a New Command

To add a new command:

1. Add a variant to the `Commands` enum in `args.rs`
2. Create a new module in `commands/` implementing the `Command` trait
3. Add the module to `commands/mod.rs`
4. Handle the new variant in `command_factory::create_command()`
5. Add tests for the new command and factory function

The compiler will guide you through steps 3-4 with helpful errors if you forget them.
