# `ewwii update`

The `update` command refreshes the widgets rendered on the screen. It allows you to immediately reflect changes in your widget configuration.

**Example:**

```bash
ewwii update
```

## Injecting variables with `--inject-vars`

You can manually inject values into the configuration using the `--inject-vars` argument. This allows you to explicitly set variable values during an update.

```bash
ewwii update --inject-vars "VAR1=bar,VAR2=foo2"
```
