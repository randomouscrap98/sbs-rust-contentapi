
const COPYVISUALTIMEOUT = 2500;

//E is a click event, this way it's just one function. This is a function
//you can attach as a click event to ANY element where you want text to be
//copyable. It expects [data-copytext] and a member function 'copyVisual'
function copy_text_generic_onclick(e)
{
    e.preventDefault();
    var element = e.target;
    if(!element.hasAttribute("data-copytext"))
        console.error("Did not set data-copytext on ", element);
    if(!element.copyVisual)
        console.error("Did not set copyVisual on ", element);
    if(element.timer)
    {
        console.warn("Can't copy just yet, still showing the copy")
        return; 
    }
    var text = element.getAttribute("data-copytext");
    navigator.clipboard.writeText(text).then(
        () => {
            element.timer = setTimeout(() => 
            {
                element.copyVisual(element, false);
                element.timer = false;
            }, COPYVISUALTIMEOUT);
            element.copyVisual(element, true);
        },
        () => { 
            console.error("Could not copy to clipboard: " + text);
            element.className += " error";
        }
    );
    element.timer
}

//copyVisual functionality for general inputs (where the value displayed is the copy)
function input_copy_visual(input, showCopy)
{
    if(showCopy)
    {
        input.value = "Copied!";
        input.className += " success";
    }
    else
    {
        input.value = input.getAttribute("data-copytext");
        input.className = input.className.replace(" success", "");
    }
}

//Setup the basic input copy on the given input
function setup_input_copy(input)
{
    input.setAttribute("data-copytext", input.value);
    input.copyVisual = input_copy_visual;
    input.onclick = copy_text_generic_onclick;
}

// For any iframes that are within that haven't been loaded yet, set the src
function load_inner_iframe(e) {
    var iframe = e.target.querySelector("iframe[data-src]");
    if (iframe) {
        iframe.setAttribute("src", iframe.getAttribute("data-src"));
        iframe.removeAttribute("data-src");
    }
}

//Apply the given key/value pairs to their respective inputs, do NOT submit the form
function apply_to_form(formObj, form)
{
    Object.keys(formObj).forEach(k =>
    {
        var input = form.querySelector(`[name="${k}"]`);

        if(!input) throw "Form field not found: " + k;

        if(input.tagName === "INPUT")
        {
            var inputType = input.getAttribute("type");

            if(inputType === "checkbox")
                input.checked = formObj[k];
            else
                input.value = formObj[k];
        }
        else if(input.tagName === "TEXTAREA")
        {
            input.textContent = formObj[k];
        }
        else
        {
            throw "Unsupported input type for field " + k;
        }
    });
}

//Assuming a regular SBS form, just... submit it? No callbacks or anything.
function submit_regular_form(form)
{
    var submit = form.querySelector(`[type="submit"]`);
    submit.click();
}
