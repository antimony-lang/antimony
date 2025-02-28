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
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded "><a href="introduction/SUMMARY.html"><strong aria-hidden="true">1.</strong> Introduction</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="introduction/installation.html"><strong aria-hidden="true">1.1.</strong> Installation</a></li><li class="chapter-item expanded "><a href="introduction/hello-world.html"><strong aria-hidden="true">1.2.</strong> Hello World!</a></li></ol></li><li class="chapter-item expanded "><a href="concepts/SUMMARY.html"><strong aria-hidden="true">2.</strong> Common language concepts</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="concepts/variables.html"><strong aria-hidden="true">2.1.</strong> Variables</a></li><li class="chapter-item expanded "><a href="concepts/datatypes.html"><strong aria-hidden="true">2.2.</strong> Data Types</a></li><li class="chapter-item expanded "><a href="concepts/functions.html"><strong aria-hidden="true">2.3.</strong> Functions</a></li><li class="chapter-item expanded "><a href="concepts/comments.html"><strong aria-hidden="true">2.4.</strong> Comments</a></li><li class="chapter-item expanded "><a href="concepts/control-flow.html"><strong aria-hidden="true">2.5.</strong> Control Flow</a></li><li class="chapter-item expanded "><a href="concepts/structured-data.html"><strong aria-hidden="true">2.6.</strong> Structured Data</a></li></ol></li><li class="chapter-item expanded "><a href="modules/SUMMARY.html"><strong aria-hidden="true">3.</strong> Modules and Imports</a></li><li class="chapter-item expanded "><a href="developers/SUMMARY.html"><strong aria-hidden="true">4.</strong> Developer Resources</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="developers/specification.html"><strong aria-hidden="true">4.1.</strong> Specification</a></li><li class="chapter-item expanded "><a href="developers/backends.html"><strong aria-hidden="true">4.2.</strong> Compiler Backends</a></li><li class="chapter-item expanded "><a href="developers/debugging.html"><strong aria-hidden="true">4.3.</strong> Debugging the compiler</a></li><li class="chapter-item expanded "><a href="developers/releasing.html"><strong aria-hidden="true">4.4.</strong> Release Workflow</a></li></ol></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0];
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
