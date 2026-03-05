# Fix `style_context().add_provider()` warning

Use CSS classes as scoping mechanism

```rs
let unique_class = format!("ewwii-widget-{}", some_unique_id);
gtk_widget.add_css_class(&unique_class);
let scss = format!(".{} {{ {} }}", unique_class, style_str);
// then load display-wide
```

# Find replacement for deprecated functions

Currently, widgets like "ComboBoxText" which are deprecated in gtk4 are marked `#[allow(deprecated)]`.
As this widget is likely to get replaced in the future, a replacement should be made for these widgets.

# Overhaul dynamic rendering system

This is quite important for v0.5.0 as the current dynamic rendering model is just
reevaluating the user's ewwii configuration when a change is detected (in global variable). 
This works most of the time, but will end up costing more resources if the user meets 
one of the following conditions:

1. Having a large configuration
2. Having lot of global variables that go off frequently

In the recent releases, a new variable called the `localsignal` has been released to
fix the second issue, but its a feature that is likely to get ignored.

The planned fix for this hoverhaul is removing the entire reevaluation system
and moving to a different model where the user has full control over their widgets.
One way to achieve this is introducing a way to control all the widgets and their properties 
through one place. A special file named `function.rhai` could be used as an interface
to control all the widgets.