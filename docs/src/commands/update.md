# `ewwii update`

The `update` command refreshes the widgets rendered in a specified window. It allows you to immediately reflect changes in your widget configuration.

**Example:**

```bash
# Update a single window
ewwii update --window "my_awesome_widget"

# Short form
ewwii update -w "my_awesome_widget"
```

## Limitation: `poll` and `listen` handlers

A known limitation of `ewwii update` is that it does **not** preserve variables created by `poll` or `listen` handlers. When you run `ewwii update`, these variables are reset to their initial values.

## Solution: Injecting variables with `--inject-vars`

To prevent certain variables from being reset, you can manually inject values into the configuration using the `--inject-vars` argument. This allows you to explicitly set variable values during an update.

```bash
# Long form
ewwii update --window "my_awesome_widget" --inject-vars "VAR1=bar,VAR2=foo2"

# Short form
ewwii update -w "my_awesome_widget" --inject-vars "VAR1=ewwii,VAR2=baz1"
```

> Note: `--inject-vars` does **not** automatically capture the current state of `poll` or `listen` variables. You must explicitly provide the values you want.

## Best Practices

-   Consider using `--inject-vars` when your widgets rely on `poll` or `listen` to avoid default resets.
-   Update multiple widgets by running `ewwii update` for each window separately.
-   Be explicit and consistent with variable values to avoid unexpected behavior.
