# Widget Properties

## widget

These properties apply to all widgets, and can be used anywhere!

**Properties**

-   `class`: `string` css class name
-   `valign`: `string` how to align this vertically. possible values: "fill", "baseline", "center", "start", "end"
-   `halign`: `string` how to align this horizontally. possible values: "fill", "baseline", "center", "start", "end"
-   `vexpand`: `bool` should this container expand vertically. Default: false
-   `hexpand`: `bool` should this widget expand horizontally. Default: false
-   `width`: `int` width of this element
-   `height`: `int` height of this element
-   `active`: `bool` If this widget can be interacted with
-   `tooltip`: `string` tooltip text (on hover)
-   `visible`: `bool` visibility of the widget
-   `style`: `string` inline scss style applied to the widget
-   `css`: `string` scss code applied to the widget

## combo-box-text

**Properties**

-   `items`: `vec` Items displayed in the combo box
-   `timeout`: `duration` timeout of the command. Default: "200ms"
-   `onchange`: `string` runs when an item is selected, replacing `{}` with the item

## expander

**Properties**

-   `name`: `string` name of the expander
-   `expanded`: `bool` sets whether it's expanded

## revealer

**Properties**

-   `transition`: `string` animation name ("slideright", "slideleft", etc.)
-   `reveal`: `bool` whether the child is revealed
-   `duration`: `duration` how long the transition lasts. Default: "500ms"

## checkbox

**Properties**

-   `checked`: `bool` initial checked state
-   `timeout`: `duration` command timeout. Default: "200ms"
-   `onchecked`: `string` command when checked
-   `onunchecked`: `string` command when unchecked

## color-button

**Properties**

-   `use_alpha`: `bool` use alpha channel
-   `onchange`: `string` command on color select
-   `timeout`: `duration` Default: "200ms"

## color-chooser

**Properties**

-   `use_alpha`: `bool` use alpha channel
-   `onchange`: `string` command on color select
-   `timeout`: `duration` Default: "200ms"

## slider

**Properties**

-   `flipped`: `bool` reverse direction
-   `marks`: `string` draw marks
-   `draw_value`: `bool` show value
-   `value_pos`: `string` where to show value ("left", "right", etc.)
-   `round_digits`: `int` number of decimal places
-   `value`: `float` current value
-   `min`: `float` minimum value
-   `max`: `float` maximum value
-   `timeout`: `duration` Default: "200ms"
-   `onchange`: `string` command on change (use `{}` for value)
-   `orientation`: `string` layout direction

## progress

**Properties**

-   `flipped`: `bool` reverse direction
-   `value`: `float` progress (0–100)
-   `orientation`: `string` layout direction

## input

**NOTE:** This widget exposes a special environment variable `INPUT_VAL` to the commands specified in `onchange` and `onaccept`.

**Properties**

-   `value`: `string` current text
-   `onchange`: `string` command on change; `INPUT_VAL` contains the new value
-   `timeout`: `duration` Default: "200ms"
-   `onaccept`: `string` command on Enter; `INPUT_VAL` contains the new value
-   `password`: `bool` obscure input

## button

**Properties**

-   `timeout`: `duration` Default: "200ms"
-   `onclick`: `string` command on activation
-   `onmiddleclick`: `string` command on middle click
-   `onrightclick`: `string` command on right click

## image

**Properties**

-   `path`: `string` image file path
-   `image_width`: `int` image width
-   `image_height`: `int` image height
-   `preserve_aspect_ratio`: `bool` keep aspect ratio
-   `fill_svg`: `string` fill color for SVGs
-   `icon`: `string` theme icon name
-   `icon_size`: `string` size of the icon

## box

**Properties**

-   `spacing`: `int` spacing between children
-   `orientation`: `string` direction of children
-   `space_evenly`: `bool` distribute children evenly

## overlay

**Properties**

_None_

## tooltip

**Properties**

_None listed_

## centerbox

**Properties**

-   `orientation`: `string` direction of layout

## scroll

**Properties**

-   `hscroll`: `bool` allow horizontal scrolling
-   `vscroll`: `bool` allow vertical scrolling
-   `propagate_natural_height`: `bool` use the natural height if true

## eventbox

**Properties**

-   `timeout`: `duration` Default: "200ms"
-   `onscroll`: `string` command on scroll (`{}` becomes direction)
-   `onhover`: `string` command on hover
-   `onhoverlost`: `string` command on hover exit
-   `cursor`: `string` cursor type
-   `ondropped`: `string` command on drop (`{}` is URI)
-   `dragvalue`: `string` URI to drag from this widget
-   `dragtype`: `string` type to drag ("file", "text")
-   `onclick`: `string` command on click
-   `onmiddleclick`: `string` command on middle click
-   `onrightclick`: `string` command on right click

## label

**Properties**

-   `text`: `string` text to display
-   `truncate`: `bool` truncate text
-   `limit_width`: `int` max characters to show
-   `truncate_left`: `bool` truncate beginning
-   `show_truncated`: `bool` show truncation
-   `unindent`: `bool` strip leading spaces
-   `markup`: `string` Pango markup
-   `wrap`: `bool` wrap text
-   `angle`: `float` rotation angle
-   `gravity`: `string` text gravity
-   `xalign`: `float` horizontal alignment
-   `yalign`: `float` vertical alignment
-   `justify`: `string` text justification
-   `wrap_mode`: `string` wrap mode ("word", "char", etc.)
-   `lines`: `int` max lines (−1 = unlimited)

## literal

**Properties**

-   `content`: `string` raw yuck

## calendar

**Properties**

-   `day`: `float` selected day
-   `month`: `float` selected month
-   `year`: `float` selected year
-   `show_details`: `bool` show details
-   `show_heading`: `bool` show heading
-   `show_day_names`: `bool` show day names
-   `show_week_numbers`: `bool` show week numbers
-   `onclick`: `string` command with `{0}`, `{1}`, `{2}` for day/month/year
-   `timeout`: `duration` Default: "200ms"

## stack

**Properties**

-   `selected`: `int` child index
-   `transition`: `string` animation name
-   `same_size`: `bool` equal child size

## transform

**Properties**

-   `rotate`: `float` rotation angle
-   `transform_origin_x`: `string` transform origin x
-   `transform_origin_y`: `string` transform origin y
-   `translate_x`: `string` shift x
-   `translate_y`: `string` shift y
-   `scale-x`: `string` scale x
-   `scale-y`: `string` scale y

## circular-progress

**Properties**

-   `value`: `float` 0–100 progress
-   `start_at`: `float` start percentage
-   `thickness`: `float` line thickness
-   `clockwise`: `bool` direction

## graph

**Properties**

-   `value`: `float` current value
-   `thickness`: `float` line thickness
-   `time_range`: `duration` duration to track
-   `min`: `float` minimum value
-   `max`: `float` maximum value
