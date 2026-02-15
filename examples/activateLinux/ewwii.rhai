tree([
  defwindow(
    "activate linux",
    #{
      monitor: 0,
      focusable: "none",
      stacking: "overlay",
      wm_ignore: false,
      geometry: #{
        x: "50px",
        y: "20px",
        width: "50px",
        height: "30px",
        anchor: "bottom right",
      },
      reserve: #{ distance: "40px", side: "top" },
    },
    label(#{ markup: "<big>Activate linux</big>\nGo to Settings to activate Linux", justify: "left",class: "activate"})
  ),
]);
