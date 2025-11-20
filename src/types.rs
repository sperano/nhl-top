/// Shared types used across the application
///
/// This module contains type definitions that are shared between
/// the library (commands, tui) and the binary (main.rs).

enum TeamNameFormat {
    Abbreviated,
    City,
    Common,
    Full,
}

// impl TeamNameFormat {
//     fn display(&self, )
// }