# Fundamentals

Ewwii uses Rhai as its configuration language. But instead of just pure Rhai, ewwii has its own layout that users should follow to create widgets using Rhai. And you may be wondering why ewwii has a "custom" layout instead of allowing users to just use pure Rhai. It's good question and the reason why ewwii has a custom layout is because it tries to remove unnecessary complexity.

The full reasons for this layout wont be explained much more because it goes way deeper than just "decreasing complexity".

For more information about Rhai, you can read [their documentation](https://rhai.rs/book/).

## Widgets and their parameters

Each widget in ewwii is a function (e.g: `button(#{...})` is a function call to create a button). So each one requires its own parameters.

For example, `defwindow` expects a **String**, **Properties**, and a function call that **returns a widget.**

**Example:**

```js
enter([
  // the string here (the text in "") is the name of the window
  // the content in #{} is the properties
  // and the 3rd `root_widget()` call is the function that returns a child.

  // defwindow cant have children in [] directly, but it expects a function returning it for it.
  defwindow("example", #{
      monitor: 0,
      windowtype: "dock",
      stacking: "fg",
      wm_ignore: false,
      geometry: #{
        x: "0%",
        y: "2px",
        width: "90%",
        height: "30px",
        anchor: "top center"
      },
      reserve: #{ distance: "40px" side: "top" }
  }, root_widget())
])
```

This is not that complex once you know the parameters of defwindow as most of the other widgets only take in properties or optinally children. **Poll/Listen** are the only things that is similar to `defwindow` and you will learn about it later in the [variables chapter](./variables.md).

## The root

It is an important concept that users must know to go forward with this documentaiton. Ewwii's Rhai layout is made to be **logical and powerful**, so the user is given access the root of the entire widget system.

The root is defined as `enter()` and it is where you should write `defwindow`.

Here is an example:

```js
enter([
  defwindow("example", #{
      monitor: 0,
      windowtype: "dock",
      stacking: "fg",
      wm_ignore: false,
      geometry: #{
        x: "0%",
        y: "2px",
        width: "90%",
        height: "30px",
        anchor: "top center"
      },
      reserve: #{ distance: "40px" side: "top" }
  }, root_widget())
])
```

Now that you saw this example, you may be wondering why we are doing `enter([])` instead of `enter()`. That is due to another fundamental concept in ewwii which is very important. You will learn about it in the [properties and child definition section](#properties-and-child-definition).

## Semi-colons

Semi-colon is an important character in Rhai. Just like programming languages like JavaScript, Java, Rust etc.

You can use the following link to read about semi-colons in the Rhai book as they have an awesome documentation.

[https://rhai.rs/book/ref/statements.html#statements](https://rhai.rs/book/ref/statements.html#statements)

## Properties and child definition

The part where most people get confused is the use of `[]` and `#{}`. Let's get into what those are and how you can use them.

The `[]` is used for adding **children** to a widget.

**Example:**

```js
fn greeter(foo) {
  return box(#{
    orientation: "horizontal",
    halign: "center"
  }, [
    // the `[]` holds a button which is the child widget of the box widget
    // each element in a `[]` should end in a comma (,) instead of a semi-colon (;).
    button(#{ onclick: `notify-send '${foo}'`, label: "baz" }),
    label(#{ text: "example" }),
  ]);
};
```

The `#{}` works similar to the `[]` but, it is used to add properties into the widget.

**Example:**

```js
fn greeter(foo) {
  // the `#{}` holds the properties of the box widget
  // each element in a `#{}` should end in a comma (,) instead of a semi-colon (;).
  return box(#{
    orientation: "horizontal",
    halign: "center"
  }, [
    // properties are assigned to both button and label using the #{}.
    button(#{ onclick: `notify-send '${foo}'`, label: "baz" }),
    label(#{ text: "example" }),
  ]);
};
```
