//Modify all the image list inputs so users just have to click to get the hash
function set_copy_on_click() {
    if (navigator && navigator.clipboard) {
        var inputs = [...document.querySelectorAll(".imagelist input")];
        inputs.forEach((x) =>
        {
            setup_input_copy(x);
            x.parentNode.className += " copyable";
        });
    }
    else {
        console.warn("Could not setup copy on click for image hashes; no window.navigator");
    }
}

//Everything is on "defer" so it's fine to just do this out in the open
set_copy_on_click();
