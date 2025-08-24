# Changelog

All notable changes to `ewwii` are documented here.

This changelog follows the [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format,
and this project adheres to [Semantic Versioning](https://semver.org/).

## [0.1.0-beta] - [UNRELEASED]

### Added

-   `std::json` module to handle all json related tasks in Rhai.
-   `api::wifi` module for handling wifi related tasks.
-   Better error handling for `ErrorFunctionNotFound`.
-   Better dynamic system which can handle dyn_id issues and reodering.
-   `std::math` module for mathematics related tasks.
-   `propagate_natural_height` property to scroll widget.
-   Faster re-evaluation of configuration by reusing compiled configuration.
-   Improved runtime error handling of WidgetNode casting.
-   Caching for ParseConfig in re-evaluation system.
-   Proper error handling for invalid external module code.
-   **call-fns** command for calling a Rhai function. Note: The function can only see poll/listen variables as their initial value.
-   **update** command with **--inject-vars** flag to update widget state. Note: All poll/listen variables will reset to their initial values.
-   `std::command` module for running shell commands.
-   `INPUT_VAL` environment variable for Input widget commands (`onchange` and `onaccept`), containing the current text of the input field.

### Changed

-   `x`, `y`, `widget`, and `height` properties on window definition are now optional.
-   Internal **Id to WidgetInfo** mapping now borrows values instead of owning them to improve performance.
-   **homogeneous** is no longer set to true if `space_evenly` property on box is not defined.

### Fixed

-   Ewwii window not closing when user requests with `WM_DELETE_WINDOW` event.
-   Ewwii window not resizable by default.
-   Ewwii not printing errors from external modules.

## [0.1.0-alpha] - 2025-08-18

### Added

-   Introduced new programmable configuration system based on Rhai, replacing the Yuck syntax.
-   New widget tree system using Rhai functions like `box`, `centerbox`, `defwidget`, etc.
-   Diffing system which is the backbone of dynamic updates.

### Changed

-   Complete rewrite of the internal widget compiler to support declarative Rhai input.
-   GTK widget construction is redesigned to work with the new tree.
-   Replaced `Simplexpr` and Yuck AST with Rhai's built in expression system and widget trees.
-   Removed dependency on Yuck parser.
-   Full rewrite of Documentation.

### Removed

-   Entire Yuck and Simplexpr code from the parsing and rendering codebase.
