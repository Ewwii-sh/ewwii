# The Rhai Expression Engine

Ewwii uses [Rhai](https://rhai.rs/) as its core expression and scripting engine. This means your configuration is more than just static values or simple substitutions—it’s **real code**, and you can use it anywhere dynamic behavior is needed.

Rhai expressions can:

-   Perform logic and branching (`if`, `match`, `? :`)
-   Handle mathematical calculations and string operations
-   Access nested data from arrays, maps, or JSON
-   Run custom functions
-   Be used directly in layout definitions, widget attributes, and style rules

Unlike Yuck, where expressions were embedded in `{ ... }` or `${ ... }`, Rhai treats expressions as **native**. They don’t need to be wrapped in special delimiters. If you can write it in Rhai, it just works.

## Example

```js
let value = 12 + foo * 10;

box(#{
  class: "baz"
  orientation: "h",
}, [
  button(#{
    class: button_active ? "active" : "inactive",
    on_click: "toggle_thing",
    label: `Some math: ${value}`,
  }),
]);
```

## Core Features

Rhai supports a wide range of expression capabilities:

-   **Mathematics**: `+`, `-`, `*`, `/`, `%`
-   **Comparisons**: `==`, `!=`, `<`, `>`, `<=`, `>=`
-   **Boolean logic**: `&&`, `||`, `!`
-   **Conditionals**: `if/else`, ternary (`cond ? a : b`)
-   **Regex matching**: `=~` operator (Rust-style regex)
-   **Optional access**: `?.` and `?.[index]`
-   **Data structures**: maps/arrays (`obj.field`, `arr[3]`, `map["key"]`)
-   **Function calls**: standard and Ewwii-specific built-ins (see below)
-   **String interpolation**: `` `Value is ${value}` `` (standard Rhai feature)

> Note
>
> ---
>
> Rhai only allows string interpolation only for the strings that are quoted in `` similar to JavaScript.
>
> > Learn more about it [here](https://rhai.rs/book/ref/strings-chars.html?interpolation#string-interpolation).

## Common Built-in Functions

💡 _You may recognize some of these from the old expression system—but now they're just Rhai functions._

Examples include:

-   `round()`, `floor()`, `ceil()`, `powi()`, `powf()`
-   `min()`, `max()`, `sin()`, `cos()`, etc.
-   `replace()`, `matches()`, `captures()`
-   `strlength()`, `arraylength()`, `objectlength()`
-   `jq()` – run jaq-compatible filters on JSON data
-   `get_env()` – read environment variables
-   `formattime()` – format UNIX timestamps
-   `formatbytes()` – format file sizes (IEC or SI)

## Dynamic Usage

Because expressions are just Rhai, you can now write real logic inline or break it into reusable functions:

```js
fn status_text(active) {
  return active ? "enabled" : "disabled";
}

label({
  text: `Status: ${status_text(system_active)}`
});
```

---

### TL;DR

> If you’ve used scripting languages like JavaScript or Lua, you’ll feel at home. Rhai gives you real control—not just interpolation tricks.
