// Array of emoji
let stringArray = [
    "🦝", "🐱", "🐵", "🦁", "🐹", "🦊"
];

// Mapping from emoji to names
let object = #{
    "🦝": "racoon",
    "🐱": "cat",
    "🐵": "ape",
    "🦁": "lion",
    "🐹": "hamster",
    "🦊": "fox"
};


// Widget: a single animal button
fn animalButton(emoji, selected) {
    return box(#{ class: "animalLayout" }, [
        eventbox(#{ 
            class: if selected == emoji { "animal selected" } else { "animal" },
            cursor: "pointer",
            onclick: "echo " + emoji + " > /tmp/selected_emoji.txt",
            dyn_id: "dyn_eventbox",
        }, [
            label(#{ text: emoji })
        ])
    ]);
}

// Widget: the row of animal buttons
fn animalRow(stringArray, selected) {
    let buttons = [];
    for emoji in stringArray {
        buttons.push(animalButton(emoji, selected));
    }
    return box(#{ 
        class: "animals",
        orientation: "h",
        halign: "center"
    }, buttons);
}

// Widget: currently selected animal display
fn currentAnimal(object, selected) {
    return box(#{}, [
        label(#{ text: object[selected] + " " + selected })
    ]);
}

// Layout widget
fn layout(stringArray, object, selected) {
    return box(#{ 
        class: "layout",
        orientation: "v",
        halign: "center"
    }, [
        animalRow(stringArray, selected),
        currentAnimal(object, selected)
    ]);
}

enter([
    // Track the selected emoji
    listen("selected", #{
        cmd: "tail -n 1 -f /tmp/selected_emoji.txt",
        initial: "🦝"
    }),


    defwindow("data-structures", #{
        monitor: 0,
        exclusive: false,
        focusable: "none",
        geometry: #{ anchor: "center", x: "0px", y: "0px", width: "100px", height: "10px" },
    }, layout(stringArray, object, selected))
])