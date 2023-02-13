//The website is designed to have maximum usability without javascript. As such, most of what
//you'll find here are quality of life improvements

//This all happens on "defer" so it's fine to do it out in the open like this
upgrade_forms();
upgrade_times();
upgrade_code();
upgrade_deleteconfirm();

function upgrade_forms()
{
    var forms = document.querySelectorAll('form[method="POST"]');
    for(var i = 0; i < forms.length; i++)
    {
        forms[i].addEventListener("submit", stdform_onsubmit);
    }
}

function upgrade_code()
{
    var codes = document.querySelectorAll(".content .code");
    for(var i = 0; i < codes.length; i++)
    {
        try
        {
            applySyntaxHighlighting(codes[i]);
        }
        catch(ex)
        {
            console.error("Couldn't highlight:", ex);
        }
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
        times[i].setAttribute("datetime", times[i].textContent);
        var date = new Date(times[i].innerHTML);
        times[i].textContent = date.toLocaleDateString();
    }

    //Here we're looking for datetime times and adding a title that's the easy read version.
    //This can't be done by the SSR because of the "locale" thing (or at least, it's super hard)
    var times = document.querySelectorAll('time[datetime]');
    for(var i = 0; i < times.length; i++)
    {
        var date = new Date(times[i].getAttribute("datetime"));
        times[i].setAttribute("title", date.toLocaleString());
    }

}

function upgrade_deleteconfirm()
{
    //We're looking for SPECIFICALLY inputs with the delete confirm, since we'll be interrupting the 
    //submit and then submitting after the confirm
    var deleteconfirms = document.querySelectorAll('input[data-confirmdelete]');
    console.log("Found delete confirms: ", deleteconfirms)
    for(var i = 0; i < deleteconfirms.length; i++)
    {
        let deleteInput = deleteconfirms[i];
        deleteInput.onclick = (e) => {
            e.preventDefault();
            if (confirm("Are you sure you want to delete " + deleteInput.getAttribute("data-confirmdelete") + "?")) 
                deleteInput.parentElement.submit();
        };
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