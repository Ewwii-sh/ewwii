// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="introduction.html">Introduction</a></li><li class="chapter-item expanded "><a href="getting_started.html"><strong aria-hidden="true">1.</strong> Getting Started</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="installation.html"><strong aria-hidden="true">1.1.</strong> Installation</a></li><li class="chapter-item expanded "><a href="statictranspl.html"><strong aria-hidden="true">1.2.</strong> Optional: Statictranspl</a></li></ol></li><li class="chapter-item expanded "><a href="config/config_and_syntax.html"><strong aria-hidden="true">2.</strong> Configuration &amp; Syntax</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="config/configuration.html"><strong aria-hidden="true">2.1.</strong> Configuration</a></li><li class="chapter-item expanded "><a href="config/rendering_and_best_practices.html"><strong aria-hidden="true">2.2.</strong> Rendering and Best Practices</a></li><li class="chapter-item expanded "><a href="config/config_fundamentals.html"><strong aria-hidden="true">2.3.</strong> Fundamentals</a></li><li class="chapter-item expanded "><a href="config/variables.html"><strong aria-hidden="true">2.4.</strong> Variables</a></li><li class="chapter-item expanded "><a href="config/expression_language.html"><strong aria-hidden="true">2.5.</strong> Expression Language</a></li></ol></li><li class="chapter-item expanded "><a href="theming/theming_and_ui.html"><strong aria-hidden="true">3.</strong> Theming &amp; UI</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="theming/working_with_gtk.html"><strong aria-hidden="true">3.1.</strong> Working With GTK</a></li><li class="chapter-item expanded "><a href="theming/styling_widgets.html"><strong aria-hidden="true">3.2.</strong> Styling Widgets</a></li></ol></li><li class="chapter-item expanded "><a href="modules/modules.html"><strong aria-hidden="true">4.</strong> Modules</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="modules/stdlib.html"><strong aria-hidden="true">4.1.</strong> Std Library</a></li><li class="chapter-item expanded "><a href="modules/apilib.html"><strong aria-hidden="true">4.2.</strong> API Library</a></li></ol></li><li class="chapter-item expanded "><a href="widgets/widgets.html"><strong aria-hidden="true">5.</strong> Widgets</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="widgets/widgets_and_params.html"><strong aria-hidden="true">5.1.</strong> Widgets &amp; Parameters</a></li><li class="chapter-item expanded "><a href="widgets/props.html"><strong aria-hidden="true">5.2.</strong> Widget Properties</a></li></ol></li><li class="chapter-item expanded "><a href="examples/examples.html"><strong aria-hidden="true">6.</strong> Examples</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="examples/starter_bar.html"><strong aria-hidden="true">6.1.</strong> Starter Bar</a></li><li class="chapter-item expanded "><a href="examples/interactive.html"><strong aria-hidden="true">6.2.</strong> Interactive Widgets</a></li><li class="chapter-item expanded "><a href="examples/theming.html"><strong aria-hidden="true">6.3.</strong> Theming Tricks</a></li></ol></li><li class="chapter-item expanded "><a href="troubleshooting.html"><strong aria-hidden="true">7.</strong> Troubleshooting</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0].split("?")[0];
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
