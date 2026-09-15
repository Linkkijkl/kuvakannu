/*

Developers, please note:

The front-end should be as complete as possible without JS.
Only use JS when nothing else will make the desired functionality happen.

*/

const clickElement = (element_selector) => {
    let elem = document.querySelector(element_selector);
    if (elem) elem.click();
};

// Listen for keyboard shortcuts
document.addEventListener("keydown", (event) => {
    switch (event.key) {
        case "ArrowLeft":
            clickElement(".prev-button");
            break;
        case "ArrowRight":
            clickElement(".next-button");
            break;
        case "Escape":
            clickElement(".back-button");
            break;
    }
});

document.addEventListener("DOMContentLoaded", itemViewInit);

function itemViewInit() {
    fullscreen();
    leftRightRebind();
    backRebind();
}

function fullscreen() {
    const topControls = document.querySelector(".top-controls");
    if (!topControls) return;
    const fullscreenButton = document.createElement("a");
    fullscreenButton.classList.add("icon-button");
    const fullscreenIcon = document.createElement("img");
    fullscreenIcon.src = "/static/material/fullscreen.svg";
    fullscreenIcon.alt = "Toggle fullscreen";
    fullscreenButton.appendChild(fullscreenIcon);
    fullscreenButton.addEventListener("click", () => {
        if (document.fullscreenElement) {
            document.exitFullscreen();
            fullscreenIcon.src = "/static/material/fullscreen.svg";
        } else {
            document.body.requestFullscreen();
            fullscreenIcon.src = "/static/material/fullscreen-exit.svg";
        }
    });
    const backButton = topControls.querySelector(".back-button");
    topControls.insertBefore(fullscreenButton, backButton);
}

function leftRightRebind() {
    const parser = new DOMParser;
    const arrowClick = async (elem) => {
        const response = await fetch(elem.href);
        const result = await response.text();
        const parsed = parser.parseFromString(result, "text/html");
        document.querySelector(".view-container").replaceWith(parsed.querySelector(".view-container"));
        itemViewInit();
        window.history.replaceState(null, parsed.head.title, elem.href);
    };
    const prev = document.querySelector(".prev-button");
    if (prev) {
        prev.addEventListener("click", (e) => {
            e.preventDefault();
            arrowClick(prev);
        });
    }
    const next = document.querySelector(".next-button");
    if (next) {
        next.addEventListener("click", (e) => {
            e.preventDefault();
            arrowClick(next);
        });
    }
}

function backRebind() {
    const backButton = document.querySelector(".back-button");
    if (backButton) backButton.addEventListener("click", (e) => {
        if (window.history.length > 1) {
            e.preventDefault();
            window.history.back();
        }
    });
}
