# Writing your ewwii configuration

(For a list of all built-in widgets (i.e. `box`, `label`, `button`), see [Widget Documentation](widgets.md).)\
Ewwii is configured using its own language called `rhai`.
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

```rust,ignore
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
      reserve: #{ distance: "40px" side: "top" }
  }, root_widget())
])
```

Here, we are defining a window named `example`, which we then define a set of properties for. Additionally, we set the content of the window to be the text `"example content"`.

You can now open your first window by running `eww open example`! Glorious!

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

```rust,ignore
fn greeter(name) {
  return box(#{
    orientation: "horizontal",
    halign: "center"
  }, [
    button(#{ onclick: "notify-send 'Hello' 'Hello, ${name}'", text: "Greet" })
  ]);
};
```

To show this, let's replace the text in our window definition with a call to this new widget:

```rust,ignore
enter([
  defwindow("example", #{
    // ... properties omitted
  }, greeter("Bob"))
])
```

There is a lot going on here, so let's step through this.

We are creating a function named `greeter` and a function is equal to a component that returns a child (widget). So function has two uses: one to return a component, and the other to do a set of functions.
And this function takes one parameters, called `name`. The `name` parameter _must_ be provided or else, you should emit it. Rhai does allow adding optional parameters, but we will talk about it later for the sake of beginners who are in-experienced with imprative programming languages.

Now inside the function, we declare the body of our widget that we are returning. We make use of a `box`, which we set a couple properties of.

We need this `box`, as a function can only ever contain a single widget - otherwise,
ewwii would not know if it should align them vertically or horizontally, how it should space them, and so on.
Thus, we wrap multiple children in a `box`.
This box then contains a button.
In that button's `onclick` property, we refer to the provided `name` using string-interpolation syntax: `"${name}"`.
This allows us to easily refer to any variables within strings.

<!-- TODO -->
<!-- In fact, there is a lot more you can do within `${...}` - more on that in the chapter about the [expression language](expression_language.md). -->

To then use our widget, we call the function that provides the widget with the necessary parameters passed.

As you may have noticed, we are using a couple predefined widgets here. These are all listed and explained in the [widgets chapter](widgets.md).

## The root

It is an important concept that users must know to go forward with this documentaiton. Ewwii's Rhai layout is made to be logical, so the user is given access the root of the entire widget system.

The root is defined as `enter()` and it is where you should write `defwindow`.

Here is an example:

```rust,ignore
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

Now that you saw this example, you may be wondering why we are doing `enter([])` instead of `enter()`. We need to add that `[]` in because we have to pass multiple references inside the `enter()` (a.k.a the root). So, we add `[]` inside the `enter()` before defining anything.

Confusing? yes. It is confusing because ewwii's rhai layout is customized for the most logic and power without sacrificing power. In a direct comparison with yuck (eww's configuration system), rhai may be a bit more verbose and complex. But once you understand the logical flow, it will be easy.

The biggest part where people get confused are the `[]` and `#{}`. Let's get into what those are and how you can use them.

As we discussed, `[]` holds children of a compontent. But `#{}` is used to add properties into the component. In both `[]` and `#{}` you have to add `,` at the end of each statement.

Here is an example with comments to let you understand:

```rust,ignore
fn widget1() {
  // this is where the logic starts.
  // we use return because defwindow expects widget1() to return a widget
  return box(#{
    orientation: "horizontal",
    halign: "center"
  }, [ // in this [], we add the children of the box
    // just like we discussed, #{} contains the properties.
    button(#{ onclick: "notify-send 'example'", text: "foo" })
  ]);
}

enter([
  // it has extra things like #{}, [], "string" etc. so we will get to it later
  defwindow(..., widget1())
  // with the `widget1()` function call,
  // we essentially ask widget1 component to give something which we can feed to root
  // to render something
])
```

## Widgets and their parameters

Each widget in ewwii is a function (e.g: `button(#{...})` is a function call to create a button). So each one requires its own parameters.

For example, defwindow needs a String, Properties, and the function that returns the root widget.

Example:

```rust,ignore
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

This is not that important once you know the parameters of defwindow as most of the other widgets only take in properties or optinally children. Poll/Listen are the only things that is similar to defwindow and you will learn about it later.

### Rendering children in your widgets

As your configuration grows, you might want to improve its structure by factoring out pieces into reusable functions.

In Ewwii’s Rhai-based configuration system, you can define wrapper functions that return widgets and accept a `children` parameter, just like built-in widgets such as `box()` or `button()`.

Here's an example of a custom container that adds a label before its children:

```rust,ignore
fn labeled_container(name, children = []) {
  return box(#{ class: "container" }, [label(#{text: name})] + children)
}
```

You can call it like this:

```rust,ignore
labeled_container("foo", [
  button(#{ onclick: "notify-send hey ho" }, [label(#{ text: "Click me" })])
]);
```

Because children are just a list of widgets, you can also write functions that structure them however you'd like. For example, here's a layout that places the first two children side by side:

```rust,ignore
fn two_boxes(children = []) {
  return box(#{}, [
    box(#{ class: "first" }, [children[0]]),
    box(#{ class: "second" }, [children[1]])
  ]);
}
```

And call it like this:

```rust,ignore
two_boxes([
  label(#{ text: "First" }),
  label(#{ text: "Second" })
]);
```

If a child is missing (e.g., children[1] doesn't exist), make sure to handle that gracefully or document the expected number of children.

## Adding dynamic content

Now that you feel sufficiently greeted by your bar, you may realize that showing data like the time and date might be even more useful than having a button that greets you.

To implement dynamic content in your widgets, you make use of _variables_.

All variables are only locally available so you would need to pass it around using function parameters. And whenever the variable changes, the value in the widget will update!

In Rhai, all variables are dynamically typed bindings to values. You can define variables using let, pass them as function parameters.

**Basic variables (`let`)**

```rust,ignore
let foo = "value";
```

This is the simplest type of variable.
Basic variables don't ever change automatically, if you need a dynamic variable, you can use built in functions like `poll()` and `listen()` to register dynamic values which we will talk about in the following section.

**Polling variables (`poll`)**

```rust,ignore
poll("var_name", #{
  // it is recommended to have initial property defined always. You can pass something like "" if you want no initial value.
  initial: "inital value",
  interval: "2s",
  cmd: "date +%H:%M:%S", // command to execute
})
```

A polling variable is a variable which runs a provided shell-script repeatedly, in a given interval.

This may be the most commonly used type of variable.
They are useful to access any quickly retrieved value repeatedly,
and thus are the perfect choice for showing your time, date, as well as other bits of information such as pending package updates, weather, and battery level.
But it is important to note that these variables are locally available only in enter (a.k.a the root) and you need to pass it to other functions with something like `some_fn(foo)` when you want to use a polled variable.

<!-- You can also specify an initial-value. This should prevent ewwii from waiting for the result of a given command during startup, thus
making the startup time faster. -->

To externally update a polling variable, `eww update` can be used like with basic variables to assign a value.

**Listening variables (`listen`)**

```rust,ignore
listen("foo", #{
  initial: "whatever",
  cmd: "tail -F /tmp/some_file",
})
```

Listening variables might be the most confusing of the bunch.
A listening variable runs a script once, and reads its output continously.
Whenever the script outputs a new line, the value will be updated to that new line.
In the example given above, the value of `foo` will start out as `"whatever"`, and will change whenever a new line is appended to `/tmp/some_file`.

These are particularly useful when you want to apply changes instantaneously when an operation happens if you have a script
that can monitor some value on its own. Volume, brightness, workspaces that get added/removed at runtime,
monitoring currently focused desktop/tag, etc. are the most common usecases of this type of variable.
These are particularly efficient and should be preffered if possible.

For example, the command `xprop -spy -root _NET_CURRENT_DESKTOP` writes the currently focused desktop whenever it changes.
Another example usecase is monitoring the currently playing song with playerctl: `playerctl --follow metadata --format {{title}}`.

<!--
**Built-in "magic" variables**

In addition to defining your own variables, ewwii provides some values for you to use out of the box.
These include values such as your CPU and RAM usage.
These mostly contain their data as JSON, which you can then get using the [json access syntax](expression_language.md).
All available magic variables are listed [here](magic-vars.md). -->

## Dynamically generated widgets with `literal`

In some cases, you want to not only change the text,
value, or color of a widget dynamically, but instead want to generate an entire widget structure dynamically.
This is necessary if you want to display lists of things (for example notifications)
where the amount is not necessarily known,
or if you want to change the widget structure in some other, more complex way.

For this, you can make use of one of ewwii's most powerful features: the `literal` widget.

```lisp
(defvar variable_containing_yuck
  "(box (button 'foo') (button 'bar'))")

; Then, inside your widget, use:
(literal :content variable_containing_yuck)
```

Here, you specify the content of your literal by providing it a string (most likely stored in a variable) which contains a single yuck widget tree.
Ewwii then reads the provided value and renders the resulting widget. Whenever it changes, the widget will be rerendered.

Note that this is not all that efficient. Make sure to only use `literal` when necessary!

## Using window arguments and IDs

In some cases you may want to use the same window configuration for multiple widgets, e.g. for multiple windows. This is where arguments and ids come in.

### Window ID

Firstly let us start off with ids. An id can be specified in the `open` command
with `--id`, by default the id will be set to the name of the window
configuration. These ids allow you to spawn multiple of the same windows. So
for example you can do:

```bash
ewwii open my_bar --screen 0 --id primary
ewwii open my_bar --screen 1 --id secondary
```

When using `open-many` you can follow the structure below. Again if no id is
given, the id will default to the name of the window configuration.

```bash
ewwii open-many my_config:primary my_config:secondary
```

You may notice with this we didn't set `screen`, this is set through the
`--arg` system, please see below for more information.

### Window Arguments

However this may not be enough and you want to have slight changes for each of
these bars, e.g. having a different class for 1080p displays vs 4k or having
spawning the window in a different size or location. This is where the
arguments come in.

Please note these arguments are **CONSTANT** and so cannot be update after the
window has been opened.

Defining arguments in a window is the exact same as in a widget so you can
have:

```lisp
(defwindow my_bar [arg1 ?arg2]
          :geometry (geometry
                       :x      "0%"
                       :y      "6px"
                       :width  "100%"
                       :height { arg1 == "small" ? "30px" : "40px" }
                       :anchor "top center")
          :stacking   "bg"
          :windowtype "dock"
          :reserve    (struts :distance "50px" :side "top")
    (my_widget :arg2 arg2))
```

Here we have two arguments, `arg1` and `arg2` (an optional parameter).

Once we have these parameters, when opening a new window, we must specify them
(unless they are optional, like `arg2`), but how? Well, we use the `--arg`
option when running the `open` command:

```bash
ewwii open my_bar --id primary --arg arg1=some_value --arg arg2=another_value
```

With the `open-many` it looks like this:

```bash
# Please note that `--arg` option must be given after all the windows names
ewwii open-many my_bar:primary --arg primary:arg1=some_value --arg primary:arg2=another_value
```

Using this method you can define `screen`, `anchor`, `pos`, `size` inside the
args for each window and it will act like giving `--screen`, `--anchor` etc. in
the `open` command.

So, now you know the basics, I shall introduce you to some of these "special"
parameters, which are set slightly differently. However these can all be
overridden by the `--arg` option.

-   `id` - If `id` is included in the argument list, it will be set to the id
    specified by `--id` or will be set to the name of the config. This can be
    used when closing the current window through ewwii commands.
-   `screen` - If `screen` is specified it will be set to the value given by
    `--screen`, so you can use this in other widgets to access screen specific
    information.

### Further insight into args in `open-many`

Now due to the system behind processing the `open-many` `--arg` option you
don't have to specify an id for each argument. If you do not, that argument
will be applied across all windows e.g.

```bash
ewwii open-many my_bar:primary my_bar:secondary --arg gui_size="small"
```

This will mean the config is the same throughout the bars.

Furthermore if you didn't specify an id for the window, you can still set args
specifically for that window - following the idea that the id will be set to
the window configuration if not given - by just using the name of the window
configuration e.g.

```bash
ewwii open-many my_primary_bar --arg my_primary_bar:screen=0
```

## Generating a list of widgets from JSON using `for`

If you want to display a list of values, you can use the `for`-Element to fill a container with a list of elements generated from a JSON-array.

```lisp
(defvar my-json "[1, 2, 3]")

; Then, inside your widget, you can use
(box
  (for entry in my-json
    (button :onclick "notify-send 'click' 'button ${entry}'"
      entry)))
```

This can be useful in many situations, for example when generating a workspace list from a JSON representation of your workspaces.
In many cases, this can be used instead of `literal`, and should most likely be preferred in those cases.

To see how to declare and use more advanced data structures, check out the [data structures example](/examples/data-structures/ewwii.rhai).

## Splitting up your configuration

As time passes, your configuration might grow larger and larger. Luckily, you can easily split up your configuration into multiple files!

There are two options to achieve this:

### Using `include`

```lisp
(include "./path/to/your/file.yuck")
```

A single yuck file may import the contents of any other yuck file. For this, make use of the `include` directive.

### Using a separate ewwii configuration directory

If you want to separate different widgets even further, you can create a new ewwii config folder anywhere else.
Then, you can tell ewwii to use that configuration directory by passing _every_ command the `--config /path/to/your/config/dir` flag.
Make sure to actually include this in all your `ewwii` calls, including `ewwii kill`, `eww logs`, etc.
This launches a separate instance of the ewwii daemon that has separate logs and state from your main ewwii configuration.

```

```
