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
fn animalButton(emoji) {
    let class = bound([emoji, selected], |vars| {
        if vars[0] == vars[1] {
            return "animal selected"
        } else {
            return "animal"
        }
    });

    return box(#{ class: "animalLayout" }, [
        eventbox(#{ 
            class: class,
            cursor: "pointer",
            onclick: "echo " + emoji + " >> /tmp/selected_emoji.txt",
        }, [
            label(#{ text: emoji })  // unique per emoji
        ])
    ]);
}

// Widget: the row of animal buttons
fn animalRow(stringArray) {
    let buttons = [];
    for emoji in stringArray {
        buttons.push(animalButton(emoji));
    }
    return box(#{ 
        class: "animals",
        orientation: "h",
        halign: "center"
    }, buttons);
}

// Widget: currently selected animal display
fn currentAnimal(object) {
    return box(#{}, [
        label(#{ text: selected })
    ]);
}

// Layout widget
fn layout(stringArray, object) {
    return box(#{ 
        class: "layout",
        orientation: "v",
        halign: "center"
    }, [
        animalRow(stringArray),
        currentAnimal(object)
    ]);
}

tree([
    // Track the selected emoji
    listen("selected", #{
        cmd: "scripts/readlast_and_truncate.sh",
        initial: "🦝",
    }),

    defwindow("data-structures", #{
        monitor: 0,
        exclusive: false,
        focusable: "none",
        windowtype: "normal",
        geometry: #{ anchor: "center", x: "0px", y: "0px", width: "100px", height: "10px" },
    }, layout(stringArray, object))
])