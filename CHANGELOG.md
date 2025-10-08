# Changelog

All notable changes to `ewwii` are documented here.

This changelog follows the [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format,
and this project adheres to [Semantic Versioning](https://semver.org/).

## [UNRELEASED]

### Added

- Support for binary level plugins.

### Changed

- The name of `slider` widget to `scale`.

### Fixed

-   `focusable` property not working issue.

## [0.3.0-alpha] - 2025-10-04

### Added

-   GTK4 support.
-   `can_target` boolean property for all widgets.
-   `icon` widget which always preserves a scaling of 1:1.
-   Full native wayland compatibility (x11 compatibility has decreased).

### Changed

-   The way dynamic updates are handled.
-   Window positioning logic.
-   X11 communication and x11 based window handling.
-   Daemon GTK main loop.
-   Application of sticky and stacking propery of window on X11.
-   The full implementation of eventbox widget.
-   GTK controller/signal handling.
-   Initialization logic of window.
-   Overlay widget dynamic post-creation re-creation logic (bugs are expected).

### Removed

-   `show_details` property of calendar.
-   `angle` property from label.
-   `icon_size` property from image widget.
-   `same_size` property from stack widget (box is a replacement).
-   `icon_name` from `image` widget as a result of the introduction of `icon` widget.
-   `transform`, `graph`, and `circular_progress` widget (temporarily).
-   Application of css class on the window.
-   Legacy GTK3 related code.
-   Centerbox widget.

## [0.2.0] - 2025-09-29

### Added

-   `state` command to print the current poll/listen variable state.
-   `m` as another duration unit for minute.
-   `std::regex` library for regex matching.
-   `engine-override` command which can be used to change engine settings of a configuration.
-   `force_normal` property for ewwii windows. It allows user to create normal windows on wayland.
-   Better error support by migrating to `rhai_trace` v0.3.0.
-   File path indicator in rhai errors.

### Fixed

-   The logs going to `eww_{}.log` instead of `ewwii_{}.log`.
-   Logs not truncating if it is over 100MB and not deleting if over 7 days old.
-   Ewwii crashing on invalid duration property.
-   The module resolver throwing error at `import` defenition.
-   Fixed commands sending error with success status.

### Removed

-   Legacy `true`/`false` support for `focusable` window property.
-   `$INPUT_VAL` variable injected in commands ran by input widget.
-   Many dependencies and code for faster build and lesser binary size.
-   `monitor` library as a step towards GTK4.

## [0.1.4] - 2025-09-18

### Added

-   `--preserve` flag to the `update` command which preserves the new updates.

## [0.1.3] - 2025-09-17

### Changed

-   `update` command so that it preserves current widget state.
-   `--inject-vars` argument of update to just `--inject` (or `-i` in short).

### Fixed

-   `image_width` and `image_height` not working for image widget.

## [0.1.2] - 2025-09-13

### Added

-   "Parent-death signal is not supported" warning on macOS.
-   Error logging on parent-death signal fail.

### Fixed

-   Code not compiling for FreeBSD.

## [0.1.1] - 2025-09-07

### Added

-   Better poll handling for performance.

### Changed

-   `update` to not require a window argument.

### Fixed

-   Poll/Listen handlers not working for multiple windows.

## [0.1.0] - 2025-09-06

### Added

-   SIGINT and SIGTERM catching to KILL AND OBLITERATE children cleanly.
-   Proper poll/listen handler setup in `open_window(...)`.
-   Cleaner poll/listen handler shutdown on `ewwii close`.
-   `api::linux` for getting system information like `cpu`, `gpu`, `ram`, `disk`, `kernel version` etc.
-   Support for defining multiple `enter([..])` at top level.

### Changed

-   Changed `std::monitor` return values so that it will work fine with Rhai.
-   Error handling so that it uses [rhai_trace](https://github.com/byson94/rhai_trace) and [codespan-reporting](https://github.com/brendanzab/codespan) for more user-friendly and pretty errors.
-   Internal WidgetNode parsing so that user don't have to return anything (in rhai).

### Fixed

-   Ewwii creating zombie process on opening window.
-   Ewwii shutting down all poll/listen handlers when any window closes.
-   Broken `%` based width/height in window definition.
-   Default poll/listen variables not working in external modules.
-   `call-fns` command not returning anything.
-   `poll/listen` definition extractor not skipping comments.

### Removed

-   Deprecated attribute warning which cluttered the logs.
-   `std::json` (Rhai has built in json support).
-   `std::math` (Rhai already convers everything that it has).
-   The need for `dyn_id` for dynamic system. THIS IS A MAJOR UX change as it massivly reduces errors and the burden on users.
-   Daemon exit on configuration error mechanism in favor of hot-reloading.

## [0.1.0-beta] - 2025-08-27

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
-   Proper error handling for runtime error in external module code.
-   **call-fns** command for calling a Rhai function. Note: The function can only see poll/listen variables as their initial value.
-   **update** command with **--inject-vars** flag to update widget state. Note: All poll/listen variables will reset to their initial values.
-   `std::command` module for running shell commands.
-   `INPUT_VAL` environment variable for Input widget commands (`onchange` and `onaccept`), containing the current text of the input field.
-   Parse error handling for external module code.
-   Killing poll/listen handlers feature in `ewwii kill` command.

### Changed

-   `x`, `y`, `widget`, and `height` properties on window definition are now optional.
-   Internal **Id to WidgetInfo** mapping now borrows values instead of owning them to improve performance.
-   **homogeneous** is no longer set to true if `space_evenly` property on box is not defined.
-   Widget creation and diffing system to borrow WidgetNode instead of owning them to improve performance.

### Fixed

-   Ewwii window not closing when user requests with `WM_DELETE_WINDOW` event.
-   Ewwii window not resizable by default.
-   Ewwii not printing errors from external modules.
-   Prevents early termination caused by kill_on_drop on `listen`.
-   Slider updating value while dragging issue.

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
