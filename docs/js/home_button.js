document.addEventListener("DOMContentLoaded", function () {
  const homeBtn = document.createElement("a");
  homeBtn.href = "https://ewwii-sh.github.io/";
  homeBtn.innerText = "🏠︎";
  homeBtn.className = "home-button";

  homeBtn.style.padding = "6px";
  homeBtn.style.backgroundColor = "transparent";
  homeBtn.style.color = "grey";
  homeBtn.style.borderRadius = "4px";
  homeBtn.style.textDecoration = "none";
  homeBtn.style.display = "inline-flex";
  homeBtn.style.alignItems = "center";
  homeBtn.style.justifyContent = "center";
  homeBtn.style.transition = "all 0.2s ease";

  // Hover effect
  homeBtn.addEventListener("mouseover", () => {
    homeBtn.style.color = "#bbbbbb";
    homeBtn.style.transform = "scale(1.01)";
  });

  homeBtn.addEventListener("mouseout", () => {
    homeBtn.style.backgroundColor = "transparent";
    homeBtn.style.color = "grey";
    homeBtn.style.transform = "scale(1)";
  });

  const header = document.querySelector(".right-buttons");
  if (header) header.appendChild(homeBtn);
});
