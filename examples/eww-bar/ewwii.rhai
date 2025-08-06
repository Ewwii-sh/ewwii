fn bar(music, time, volume) {
  return centerbox(#{ orientation: "h" }, [workspaces(), music(music), sidestuff(volume, time)]);
}

fn sidestuff(volume, time) {
  return box(#{ class: "sidestuff", orientation: "h", space_evenly: false, halign: "end"}, [
    metric(#{
      label: "🔊",
      value: volume,
      onchange: "amixer -D pulse sset Master {}%",
    }),
    label(#{ text: time })
  ]);
}

fn workspaces() {
  return box(#{
    class: "workspaces",
    orientation: "h",
    space_evenly: true,
    halign: "start",
    spacing: 10,
  }, [
    button(#{ onclick: "wmctrl -s 0", label: "1" }),
    button(#{ onclick: "wmctrl -s 1", label: "2" }),
    button(#{ onclick: "wmctrl -s 1", label: "3" }),
    button(#{ onclick: "wmctrl -s 3", label: "4" }),
    button(#{ onclick: "wmctrl -s 4", label: "5" }),
    button(#{ onclick: "wmctrl -s 5", label: "6" }),
    button(#{ onclick: "wmctrl -s 6", label: "7" }),
    button(#{ onclick: "wmctrl -s 7", label: "8" }),
    button(#{ onclick: "wmctrl -s 8", label: "9" }),
  ]);
}

fn music(music) {
  let label_text = if music != "" {
      "🎵" + music
  } else {
      ""
  };
  return box(#{
    class: "music",
    orientation: "h",
    space_evenly: false,
    halign: "center"
  }, [
    label(#{ text: label_text }),
  ]);
}

fn metric(props) {
  let label_prop = props.label;
  let value_prop = props.value;
  let onchange_prop = props.onchange;

  return box(#{
    orientation: "h",
    class: "metric",
    space_evenly: false,
  }, [
    slider(#{
      min: 0,
      max: 101,
      active: onchange_prop != "",
      value: value_prop,
      onchange: onchange_prop,
    }),
  ]);
}

enter([
  listen("music", #{
    initial: "",
    cmd: "playerctl --follow metadata --format '{{ artist }} - {{ title }}' || true"
  }),

  poll("volume", #{ 
      interval: "1s", 
      cmd: "scripts/getvol",
      initial: "" 
  }),

  poll("time", #{ 
      interval: "10s", 
      cmd: "date '+%H:%M %b %d, %Y'",
      initial: "" 
  }),

  defwindow("bar", #{
      monitor: 0,
      windowtype: "dock",
      geometry: #{
          x: "0%",
          y: "0%",
          width: "90%",
          height: "10px",
          anchor: "top center",
      },
      reserve: #{ side: "top", distance: "4%" }
  }, bar(music, time, volume)),
])