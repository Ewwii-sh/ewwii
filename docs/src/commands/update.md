# `ewwii update`

The `update` command refreshes the widgets rendered on the screen. It allows you to immediately reflect changes in your widget configuration.

**Example:**

```bash
ewwii update
```

## Injecting variables with `--inject`

You can manually inject values into the configuration using the `--inject` argument. This allows you to explicitly set variable values during an update.

```bash
ewwii update --inject "VAR1=bar,VAR2=foo2"
# or in short
ewwii update -i "VAR1=baz,VAR2=zoo"
```

## Preserving updates with `--preserve`

When you inject variables, the state is only updated temporarily. That means that if some other poll/listen variable triggered a update, your update will get overridden. So, this is why the `--preserve` flag exists. This flag allows you to preserve the variables in memory until it's overridden.

```bash
ewwii update -i "foo=hello" --preserve
# or in short
ewwii update -i "foo=hello" -p
```
