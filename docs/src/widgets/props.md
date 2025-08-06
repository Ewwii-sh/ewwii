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

-   `use-alpha`: `bool` use alpha channel
-   `onchange`: `string` command on color select
-   `timeout`: `duration` Default: "200ms"

## color-chooser

**Properties**

-   `use-alpha`: `bool` use alpha channel
-   `onchange`: `string` command on color select
-   `timeout`: `duration` Default: "200ms"

## slider

**Properties**

-   `flipped`: `bool` reverse direction
-   `marks`: `string` draw marks
-   `draw-value`: `bool` show value
-   `value-pos`: `string` where to show value ("left", "right", etc.)
-   `round-digits`: `int` number of decimal places
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

**Properties**

-   `value`: `string` current text
-   `onchange`: `string` command on change
-   `timeout`: `duration` Default: "200ms"
-   `onaccept`: `string` command on Enter
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
-   `image-width`: `int` image width
-   `image-height`: `int` image height
-   `preserve-aspect-ratio`: `bool` keep aspect ratio
-   `fill-svg`: `string` fill color for SVGs
-   `icon`: `string` theme icon name
-   `icon-size`: `string` size of the icon

## box

**Properties**

-   `spacing`: `int` spacing between children
-   `orientation`: `string` direction of children
-   `space-evenly`: `bool` distribute children evenly

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
-   `limit-width`: `int` max characters to show
-   `truncate-left`: `bool` truncate beginning
-   `show-truncated`: `bool` show truncation
-   `unindent`: `bool` strip leading spaces
-   `markup`: `string` Pango markup
-   `wrap`: `bool` wrap text
-   `angle`: `float` rotation angle
-   `gravity`: `string` text gravity
-   `xalign`: `float` horizontal alignment
-   `yalign`: `float` vertical alignment
-   `justify`: `string` text justification
-   `wrap-mode`: `string` wrap mode ("word", "char", etc.)
-   `lines`: `int` max lines (−1 = unlimited)

## literal

**Properties**

-   `content`: `string` raw yuck

## calendar

**Properties**

-   `day`: `float` selected day
-   `month`: `float` selected month
-   `year`: `float` selected year
-   `show-details`: `bool` show details
-   `show-heading`: `bool` show heading
-   `show-day-names`: `bool` show day names
-   `show-week-numbers`: `bool` show week numbers
-   `onclick`: `string` command with `{0}`, `{1}`, `{2}` for day/month/year
-   `timeout`: `duration` Default: "200ms"

## stack

**Properties**

-   `selected`: `int` child index
-   `transition`: `string` animation name
-   `same-size`: `bool` equal child size

## transform

**Properties**

-   `rotate`: `float` rotation angle
-   `transform-origin-x`: `string` transform origin x
-   `transform-origin-y`: `string` transform origin y
-   `translate-x`: `string` shift x
-   `translate-y`: `string` shift y
-   `scale-x`: `string` scale x
-   `scale-y`: `string` scale y

## circular-progress

**Properties**

-   `value`: `float` 0–100 progress
-   `start-at`: `float` start percentage
-   `thickness`: `float` line thickness
-   `clockwise`: `bool` direction

## graph

**Properties**

-   `value`: `float` current value
-   `thickness`: `float` line thickness
-   `time-range`: `duration` duration to track
-   `min`: `float` minimum value
-   `max`: `float` maximum value
