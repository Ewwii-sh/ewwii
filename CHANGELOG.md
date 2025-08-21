# Changelog

All notable changes to `ewwii` are documented here.

This changelog follows the [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format,
and this project adheres to [Semantic Versioning](https://semver.org/).

## [0.1.0-beta] - [UNRELEASED]

### Added

-   Added `std::json` module to handle all json related tasks in Rhai.
-   Added `api::wifi` module for handling wifi related tasks.
-   Better error handling for `ErrorFunctionNotFound`.
-   Better dynamic system which can handle dyn_id issues and reodering.
-   Added `std::math` module for mathematics related tasks.
-   Added `propagate_natural_height` property to scroll widget.

### Changed

-   Made `x`, `y`, `widget` and `height` properties optional on window definition.

### Fixed

-   Ewwii window not closing when user requests with `WM_DELETE_WINDOW` event.
-   Ewwii window not resizable by default.

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
