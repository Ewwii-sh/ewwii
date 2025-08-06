fn separator() {
    return box(#{
        class: "separator",
        width: "1px",
        height: "20px",
        background_color: "#888",
        margin: "0 8px"
    }, []);
}

fn widget1() {
    return box(#{
        class: "widget1",
        orientation: "h",
        space_evenly: false,
        halign: "start",
        spacing: 5
    }, [
        button(#{ onclick: "print('Hello there!')", text: "greet" }),
        separator(),
        button(#{ onclick: "print('Bye bye!')", text: "say bye" })
    ]);
}

fn window_body() {
    return centerbox(#{ orientation: "h" }, [
        widget1(),
        box(#{},[]),
        box(#{},[]),
    ]);
}

enter([
    poll("cpu_usage", #{ 
        interval: "1s", 
        cmd: "./test.sh",  
        initial: "initial" 
    }),
    listen("net_speed", #{ 
        signal: "./test.sh"
    }),

    defwindow("main_window", #{
        monitor: 0,
        windowtype: "dock",
        geometry: #{ x: "0px", y: "0px", width: "30px", height: "30px" },
        reserve: #{ top: "30", distance: "30" }
    }, window_body())
]);