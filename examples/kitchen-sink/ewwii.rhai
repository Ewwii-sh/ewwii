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
            circular_progress(#{}),
            input(#{}),
            button(#{}),
            image(#{ path: "./EwwiiLogo.png", content_fit: "contain" }),
            overlay(#{}, [
                label(#{ text: "A little overlay :D" })
            ]),
            tooltip(#{}, [
                label(#{ text: "Child 1" }),
                label(#{ text: "Child 2" }),
            ]),
            eventbox(#{}, [
                label(#{ text: "Some event is gonna happen" })
            ]),
            label(#{ text: "Hello, World" }),
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