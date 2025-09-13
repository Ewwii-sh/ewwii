# Writing your ewwii configuration

(For a list of all built-in widgets (i.e. `box`, `label`, `button`), see [Widget Documentation](../widgets/widgets.md).)

Ewwii is configured using a language called `Rhai`.
Using rhai, you declare the structure and content of your widgets, the geometry, position, and behavior of any windows,
as well as any state and data that will be used in your widgets.
Rhai is based around imparative syntax, which you may know from programming languages like C, Rust etc.
If you're using vim, you can make use of [vim-rhai](https://github.com/rhaiscript/vim-rhai) for editor support.
If you're using VSCode, you can get syntax highlighting and formatting from [vscode-rhai](https://marketplace.visualstudio.com/items?itemName=rhaiscript.vscode-rhai).

Additionally, any styles are defined in CSS or SCSS (which is mostly just slightly improved CSS syntax).
While ewwii supports a significant portion of the CSS you know from the web,
not everything is supported, as ewwii relies on GTK's own CSS engine.
Notably, some animation features are unsupported,
as well as most layout-related CSS properties such as flexbox, `float`, absolute position or `width`/`height`.

To get started, you'll need to create two files: `ewwii.rhai` and `ewwii.scss` (or `ewwii.css`, if you prefer).
These files must be placed under `$XDG_CONFIG_HOME/ewwii` (this is most likely `~/.config/ewwii`).

Now that those files are created, you can start writing your first widget!

## Creating your first window

Firstly, you will need to create a top-level window. Here, you configure things such as the name, position, geometry, and content of your window.

Let's look at an example window definition:

```js,ignore
enter([ // Add all defwindow inside enter. Enter is the root of the config.
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
      reserve: #{ distance: "40px", side: "top" }
  }, label(#{ text: "example content" }))
])
```

Here, we are defining a window named `example`, which we then define a set of properties for. Additionally, we set the content of the window to be the text `"example content"`.

You can now open your first window by running `ewwii open example`! Glorious!

### `defwindow`-properties

|   Property | Description                                                              |
| ---------: | ------------------------------------------------------------------------ |
|  `monitor` | Which monitor this window should be displayed on. See below for details. |
| `geometry` | Geometry of the window.                                                  |

**`monitor`-property**

This field can be:

-   the string `<primary>`, in which case ewwii tries to identify the primary display (which may fail, especially on wayland)
-   an integer, declaring the monitor index
-   the name of the monitor
-   a string containing a JSON-array of monitor matchers, such as: `'["<primary>", "HDMI-A-1", "PHL 345B1C", 0]'`. Ewwii will try to find a match in order, allowing you to specify fallbacks.

**`geometry`-properties**

|          Property | Description                                                                                                             |
| ----------------: | ----------------------------------------------------------------------------------------------------------------------- |
|          `x`, `y` | Position of the window. Values may be provided in `px` or `%`. Will be relative to `anchor`.                            |
| `width`, `height` | Width and height of the window. Values may be provided in `px` or `%`.                                                  |
|          `anchor` | Anchor-point of the window. Either `center` or combinations of `top`, `center`, `bottom` and `left`, `center`, `right`. |
|       `resizable` | Whether to allow resizing the window or not. Eiither `true` or `false`.                                                 |

<br/>
Depending on if you are using X11 or Wayland, some additional properties exist:

#### X11

|     Property | Description                                                                                                                                                                                                                                                    |
| -----------: | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
|   `stacking` | Where the window should appear in the stack. Possible values: `fg`, `bg`.                                                                                                                                                                                      |
|  `wm_ignore` | Whether the window manager should ignore this window. This is useful for dashboard-style widgets that don't need to interact with other windows at all. Note that this makes some of the other properties not have any effect. Either `true` or `false`.       |
|    `reserve` | Specify how the window manager should make space for your window. This is useful for bars, which should not overlap any other windows.                                                                                                                         |
| `windowtype` | Specify what type of window this is. This will be used by your window manager to determine how it should handle your window. Possible values: `normal`, `dock`, `toolbar`, `dialog`, `desktop`. Default: `dock` if `reserve` is specified, `normal` otherwise. |

#### Wayland

|    Property | Description                                                                                                                                                            |
| ----------: | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
|  `stacking` | Where the window should appear in the stack. Possible values: `fg`, `bg`, `overlay`, `bottom`.                                                                         |
| `exclusive` | Whether the compositor should reserve space for the window automatically. Either `true` or `false`. If `true` `:anchor` has to include `center`.                       |
| `focusable` | Whether the window should be able to be focused. This is necessary for any widgets that use the keyboard to work. Possible values: `none`, `exclusive` and `ondemand`. |
| `namespace` | Set the wayland layersurface namespace ewwii uses. Accepts a `string` value.                                                                                           |

## Your first widget

While our bar is already looking great, it's a bit boring. Thus, let's add some actual content!

```js
fn greeter(name) {
  return box(#{
    orientation: "horizontal",
    halign: "center"
  }, [
    button(#{ onclick: `notify-send 'Hello' 'Hello, ${name}'`, label: "Greet" })
  ]);
};
```

To show this, let's replace the text in our window definition with a call to this new widget:

```js
enter([
    defwindow(
        "example",
        #{
            // ... properties omitted
        },
        greeter("Bob")
    ),
]);
```

There is a lot going on here, so let's step through this.

We are creating a function named `greeter` and a function is equal to a component that returns a child (widget). So function has two uses: one to return a component, and the other to do a set of functions.
And this function takes one parameters, called `name`. The `name` parameter _must_ be provided or else, you should emit it. Rhai does allow adding optional parameters, but we will talk about it later for the sake of beginners who are in-experienced with imprative programming languages.

Now inside the function, we declare the body of our widget that we are returning. We make use of a `box`, which we set a couple properties of.

We need this `box`, as a function can only ever contain a single widget - otherwise,
ewwii would not know if it should align them vertically or horizontally, how it should space them, and so on.
Thus, we wrap multiple children in a `box`.
This box then contains a button.
In that button's `onclick` property, we refer to the provided `name` using string-interpolation syntax: `` `${name}` ``. It is not possible to use a variable within a `""` or `''` just like javascript. You can learn more about it [here](https://rhai.rs/book/ref/strings-chars.html?interpolation#string-interpolation).

<!-- TODO -->
<!-- In fact, there is a lot more you can do within `${...}` - more on that in the chapter about the [expression language](expression_language.md). -->

To then use our widget, we call the function that provides the widget with the necessary parameters passed.

As you may have noticed, we are using a couple predefined widgets here. These are all listed and explained in the [widgets chapter](widgets.md).
