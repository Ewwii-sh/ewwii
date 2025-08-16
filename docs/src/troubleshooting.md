# Troubleshooting

Here you will find help if something doesn't work. If the issue isn't listed here, please [open an issue on the GitHub repo.](https://github.com/Ewwii-sh/ewwii/issues)

## Ewwii does not compile

1. Make sure that you are compiling ewwii using a recent version of rust (run `rustup update` to be sure you have the latest version available)
2. Make sure you have all the necessary dependencies. If there are compile-errors, the compiler will tell you what you're missing.

## Ewwii does not work on Wayland

1. Make sure you compiled ewwii with the `--no-default-features --features=wayland` flags.
2. Make sure that you're not trying to use X11-specific features (these are (hopefully) explicitly specified as such in the documentation).

## My configuration is not loaded correctly

1. Make sure the `ewwii.rhai` and `ewwii.(s)css` files are in the correct places.
2. Sometimes, ewwii might fail to load your configuration as a result of a configuration error. Make sure your configuration is valid.

## Something isn't styled correctly!

Check the [GTK-Debugger](working_with_gtk.md#gtk-debugger) to get more insight into what styles GTK is applying to which elements.

## General issues

You should try the following things before opening an issue or doing more specialized troubleshooting:

-   Kill the ewwii daemon by running `ewwii kill` and re-open your window with the `--debug`-flag to get additional log output.
-   Now you can take a look at the logs by running `ewwii logs`.
-   Use `ewwii state` to see the state of all variables.
-   Use `ewwii debug` to see the structure of your widget and other information.
-   Update to the latest ewwii version.
-   Sometimes hot reloading doesn't work. In that case, you can make use of `ewwii reload` manually.

Remember, if your issue isn't listed here, [open an issue on the GitHub repo](https://github.com/Ewwii-sh/ewwii/issues).
