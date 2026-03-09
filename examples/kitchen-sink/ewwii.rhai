fn widgets() {
    scroll(#{}, [
        box(#{ orientation: "v", space_evenly: false }, [
            combo_box_text(#{
                items: ["Item1", "Item2", "ewwii?", time]
            }),
            expander(#{}, [
                label(#{ text: "Surprise!" })
            ]),
            revealer(#{}, [
                label(#{ text: "Hey! I am the revealed child!" })
            ]),
            checkbox(#{}),
            color_button(#{}),
            color_chooser(#{}),
            scale(#{}),
            progress(#{ value: 50, text: "Progress...", show_text: true }),
            circular_progress(#{ value: 50 }),
            input(#{ placeholder: "Text goes here..." }),
            button(#{ label: "A simple button" }),
            image(#{ path: "./EwwiiLogo.png", content_fit: "contain", image_height: 16, image_width: 16 }),
            overlay(#{}, [
                label(#{ text: "A little overlay :D" })
            ]),
            tooltip(#{}, [
                label(#{ text: "Child 1" }),
                label(#{ text: "Child 2" }),
            ]),
            eventbox(#{ drag_type: "text", dragvalue: "Hello :D" }, [
                label(#{ text: "Some event is gonna happen" })
            ]),
            label(#{ text: "Hello, World", truncate: false, show_truncated: false, limit_width: 5 }),
            calendar(#{}),
            stack(#{}, [
                label(#{ text: "Page 1" }),
                label(#{ text: "Page 2" })
            ]),
            flowbox(#{}, [])
        ])
    ])
}

tree([
    poll("time", #{
        cmd: "date +%H:%M:%S",
        initial: "Initial Time",
        interval: "1s"
    }),

    defwindow("sink", #{
        monitor: 0,
        windowtype: "normal",
        force_normal: true,
    }, widgets()),
])