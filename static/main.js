/*

Developers, please note:

The front-end should be as complete as possible without JS.
Only use JS when nothing else will make the desired functionality happen.

*/

const click_element = (element_selector) => {
    let elem = document.querySelector(element_selector);
    if (elem) elem.click();
};

// Listen for keyboard shortcuts
document.addEventListener("keydown", (event) => {
    switch (event.key) {
        case "ArrowLeft":
            click_element(".prev-button");
            break;
        case "ArrowRight":
            click_element(".next-button");
            break;
        case "Escape":
            click_element(".back-button");
            break;
    }
});
