# Styling Widgets

Ewwii also allows writing inline styles to widgets using the `style` property. These styles are then applied at runtime using GTK's CSS system.

**Example:**

```js
fn foo() {
    return box(#{
        style: "color: black; background-color: #fff;",
    }, [ label(#{ text: "baz" }) ]);
}
```

> This example makes the text color of all child widgets of box to black and sets the background color of the box to white (`#fff` is the hexadecimal for white).
