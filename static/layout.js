//The website is designed to have maximum usability without javascript. As such, most of what
//you'll find here are quality of life improvements

window.onload = function(wole) {
    var forms = document.querySelectorAll('form[method="POST"]');
    for(var i = 0; i < forms.length; i++)
    {
        forms[i].addEventListener("submit", stdform_onsubmit);
    }
};

//Onsubmit for standard (non-javascript) forms. This isn't required to
//submit these forms (the website is d)
function stdform_onsubmit(event)
{
    var input = event.target.querySelector('input[type="submit"]');
    if(!input) 
        console.warn("No submit found for POST form: ", form);
    else 
        input.setAttribute("disabled", "");
}