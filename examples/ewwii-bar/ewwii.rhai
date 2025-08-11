fn bar(music_var, volume, time) {
  return centerbox(#{ orientation: "h" }, [
    workspaces(), 
    music(music_var), 
    sidestuff(volume, time),
  ]);
}

fn sidestuff(volume, time) {
  return box(#{ class: "sidestuff", orientation: "h", space_evenly: false, halign: "end"}, [
    metric(#{
      label: "🔊",
      value: volume,
      onchange: "pamixer --set-volume {}",
      dyn_id: "volume_metric"
    }),
    label(#{ text: time, dyn_id: "current_time" })
  ]);
}

fn workspaces() {
  let buttons = [];

  // looping and creating buttons
  for n in 0..9 {
    buttons.push(
        button(#{ 
            onclick: "i3-msg workspace " + (n + 1), 
            label: (n + 1).to_string() 
        })
    );
  }

  return box(#{
    class: "workspaces",
    orientation: "h",
    space_evenly: true,
    halign: "start",
    spacing: 10,
  }, buttons);
}

fn music(music_var) {
  let label_text = if music_var != "" {
      "🎵" + music_var
  } else {
      ""
  };
  return box(#{
    class: "music",
    orientation: "h",
    space_evenly: false,
    halign: "center"
  }, [
    label(#{ text: label_text, dyn_id: "music_bar" }),
  ]);
}

fn metric(props) {
  let label_prop = props.label;
  let value_prop = props.value;
  let onchange_prop = props.onchange;
  let dyn_id_prop = props.dyn_id;


  return box(#{
    orientation: "h",
    class: "metric",
    space_evenly: false,
  }, [
    box(#{ class: "label" }, [ label(#{ text: label_prop }) ]),
    slider(#{
      min: 0,
      max: 101,
      active: onchange_prop != "",
      value: value_prop,
      onchange: onchange_prop,
      dyn_id: dyn_id_prop
    }),
  ]);
}

enter([
  listen("music_var", #{
    initial: "",
    cmd: "echo Playing Ewwii",
  }),

  listen("volume", #{ 
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
  }, bar(music_var, volume, time)),
]);
