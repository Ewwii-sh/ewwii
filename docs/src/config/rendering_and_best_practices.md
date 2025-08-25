# Rendering and Best Practices

## Rendering children in your widgets

As your configuration grows, you might want to improve its structure by factoring out pieces into reusable functions.

In Ewwii’s Rhai-based configuration system, you can define wrapper functions that return widgets and accept a `children` parameter (or any parameter that you prefer), just like built-in widgets such as `box()` or `button()`.

Here's an example of a custom container that adds a label before its children:

```js
fn labeled_container(name, children = []) {
  return box(#{ class: "container" }, [label(#{text: name})] + children)
}
```

You can call it like this:

```js
labeled_container("foo", [
    button(#{ onclick: "notify-send hey ho", label: "Click me" }),
]);
```

Because children are just a list of widgets, you can also write functions that structure them however you'd like. For example, here's a layout that places the first two children side by side:

```js
fn two_boxes(children = []) {
  return box(#{}, [
    box(#{ class: "first" }, [children[0]]),
    box(#{ class: "second" }, [children[1]])
  ]);
}
```

And call it like this:

```js
two_boxes([label(#{ text: "First" }), label(#{ text: "Second" })]);
```

If a child is missing (e.g., children[1] doesn't exist), make sure to handle that gracefully or document the expected number of children.

<!-- TODO: add it once literal is implemented -->
<!-- ## Dynamically generated widgets with `literal`

In some cases, you want to not only change the text,
value, or color of a widget dynamically, but instead want to generate an entire widget structure dynamically.
This is necessary if you want to display lists of things (for example notifications)
where the amount is not necessarily known,
or if you want to change the widget structure in some other, more complex way.

For this, you can make use of one of ewwii's most powerful features: the `literal` widget.

```js
let variable_containing_rhai = "(box (button 'foo') (button 'bar'))";

// Then, inside your widget, use:
literal(#{ content: variable_containing_rhai })
```

Here, you specify the content of your literal by providing it a string (most likely stored in a variable) which contains a single yuck widget tree.
Ewwii then reads the provided value and renders the resulting widget. Whenever it changes, the widget will be rerendered.

Note that this is not all that efficient. Make sure to only use `literal` when necessary! -->

## Window ID

In some cases you may want to use the same window configuration for multiple widgets, e.g. for multiple windows. This is where arguments and ids come in.

Firstly let us start off with ids. An id can be specified in the `open` command
with `--id`, by default the id will be set to the name of the window
configuration. These ids allow you to spawn multiple of the same windows. So
for example you can do:

```bash
ewwii open my_bar --screen 0 --id primary
ewwii open my_bar --screen 1 --id secondary
```

## Generating a list of widgets from array using `for`

If you want to display a list of values, you can use the `for`-Element to fill a container with a list of elements generated from a JSON-array.

```js
let my_array = [1, 2, 3];

// Then, inside your widget, you can use
box(#{}, [
  for entry in my_array {
    button(#{ onclick: `notify-send 'click' 'button ${entry}'`, label: entry.to_string() })
  }
])
```

This can be useful in many situations, for example when generating a workspace list from an array representation of your workspaces.
In many cases, this can be used instead of `literal`, and should most likely be preferred in those cases.

<!-- To see how to declare and use more advanced data structures, check out the [data structures example](/examples/data-structures/ewwii.rhai). -->

## Splitting up your configuration

As time passes, your configuration might grow larger and larger. Luckily, you can easily split up your configuration into multiple files!

There are two options to achieve this:

### Using `import/export`

```js
// in ./foo/baz.rhai
/// Note: all functions are automatically exported.
fn greet() { return "Greetings!" }
let PI = 3.14159
export PI; // we need to export variables manually

// in ./ewwii.rhai
import "foo/baz" as example;
print(example::greet()); // Greetings!
print(example::PI); // 3.14159
```

A rhai file may import the contents of any other rhai file that they export. For this, make use of the `import` directive. If you are exporting a variable/function, make use the `export` directive.

### Using a separate ewwii configuration directory

If you want to separate different widgets even further, you can create a new ewwii config folder anywhere else.
Then, you can tell ewwii to use that configuration directory by passing _every_ command the `--config /path/to/your/config/dir` flag.
Make sure to actually include this in all your `ewwii` calls, including `ewwii kill`, `eww logs`, etc.
This launches a separate instance of the ewwii daemon that has separate logs and state from your main ewwii configuration.

```bash
ewwii --config "/path/to/your/config/dir"
```
