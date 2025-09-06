# `ewwii call-fns`

The `call-fns` command allows you to call **Rhai functions** directly from the command line. This can be useful for triggering specific functionality in your widget configuration without updating or interacting with the GUI.

> **Note:** All variables created by `poll` or `listen` handlers will default to their initial values when calling functions. They **cannot be preserved**.

**Example:**

```bash
# Call a single Rhai function
ewwii call-fns "my_function(32, 21)"

# Call multiple functions
ewwii call-fns "first_function('string')" "second_function()"
```

This will execute the specified functions in the context of your current configuration.

## Limitation: Main configuration

However, there is one limitation in `call-fns`. It can only see and call functions defined within your main configuration (i.e `ewwii.rhai` file).

## Solution: Creating wrappers

But, there is one solution though! You can call external functions from functions defined in main.

**Example:**

```js
// in external.rhai
fn awesome_fn() {
    print("Awesome fn triggered!");
}
```

```js
// in ewwii.rhai
fn call_awesome_fn() {
    import "external" as external;
    external::awesome_fn();
}
```

Now you can run `ewwii call-fns "call-awesome_fn()"` and get "Awesome fn triggered!".
