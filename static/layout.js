//The website is designed to have maximum usability without javascript. As such, most of what
//you'll find here are quality of life improvements

window.onload = function(wole) {
    upgrade_forms();
    upgrade_times();
};

function upgrade_forms()
{
    var forms = document.querySelectorAll('form[method="POST"]');
    for(var i = 0; i < forms.length; i++)
    {
        forms[i].addEventListener("submit", stdform_onsubmit);
    }
}

function upgrade_times()
{
    //Note that we're only looking for time that hasn't been setup yet.
    //There may be template generated times that ARE perfectly fine and 
    //should NOT be modified!
    var times = document.querySelectorAll('time:not([datetime])');
    for(var i = 0; i < times.length; i++)
    {
        times[i].setAttribute("datetime", times[i].innerHTML);
        var date = new Date(times[i].innerHTML);
        times[i].textContent = date.toLocaleDateString();
    }
}

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