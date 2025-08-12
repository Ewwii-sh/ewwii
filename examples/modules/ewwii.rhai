import "std::monitor" as std_monitor;
import "std::text" as std_text;

fn text_mod() {
    return box(#{
        orientation: "v",
    }, [
        label(#{ text: "Text heading", class: "heading", halign: "start"  }),
        label(#{ 
            text: "'Hello World!' to slug: " + std_text::to_slug("Hello World!"), 
            halign: "start" 
        }),
        label(#{ 
            text: "'Hello World!' to camel case: " + std_text::to_camel_case("Hello World!"), 
            halign: "start" 
        }),
        label(#{ 
            text: "'Hello World!' truncated to 10: " + 
            std_text::truncate_chars("Hello World!", 10), 
            halign: "start" 
        }),
        label(#{ 
            text: "'Hello World!' to lower case: " + std_text::to_lower("Hello World!"), 
            halign: "start" 
        }),
        label(#{ 
            text: "'Hello World!' to upper case: " + std_text::to_upper("Hello World!"), 
            halign: "start" 
        }),
        seperator()
    ]);
}

fn monitor_mod() {
    return box(#{
        orientation: "v",
    }, [
        label(#{ text: "Monitor module", class: "heading",  halign: "start" }),
        label(#{ text: "Number of Monitors: " + std_monitor::count(), halign: "start" }),
        // also has primary_resolution variant which returns a (i64, i64)
        label(#{ text: "Primary monitor resolution: " + std_monitor::primary_resolution_str(), halign: "start" }),
        // also has all_resolutions variant which returns a Vec<(i64, i64)>
        label(#{ text: "All monitor resolutions: " + std_monitor::all_resolutions_str(), halign: "start" }),
        // also has dimensions variant which returns a (i64, i64, i64, i64)
        label(#{ text: "Dimensions of monitor 0: " + std_monitor::dimensions_str(0), halign: "start" }),
        label(#{ text: "DPI of monitor 0: " + std_monitor::dpi_str(0), halign: "start" }),
    ]);
}

fn seperator() {
    return box(#{ margin: "0 4px", class: "separator" }, []);
}

fn window() {
    return box(#{}, [
        scroll(#{
            hscroll: false,
        }, [
            box(#{
                orientation: "v"
            }, [
                text_mod(),
                monitor_mod(),
            ])
        ]),
    ]);
}

enter([
    defwindow("modules-example", #{
        monitor: 0,
        windowtype: "normal",
        geometry: #{
            x: "0%",
            y: "0%",
            width: "400px",
            height: "400px",
            anchor: "center",
        },
    }, window())
])