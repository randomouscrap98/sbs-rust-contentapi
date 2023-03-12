//The website is designed to have maximum usability without javascript. As such, most of what
//you'll find here are quality of life improvements

//This all happens on "defer" so it's fine to do it out in the open like this
upgrade_forms();
upgrade_markupeditors();
upgrade_times();
upgrade_markup();
upgrade_code();
upgrade_deleteconfirm();

function upgrade_forms()
{
    var forms = document.querySelectorAll('form[method="POST"]:not([data-noupgrade])');
    for(var i = 0; i < forms.length; i++)
    {
        forms[i].addEventListener("submit", stdform_onsubmit);
    }
}

function upgrade_markupeditors()
{
    var editors = document.querySelectorAll('.markupeditor');
    for(var i = 0; i < editors.length; i++)
    {
        let editor = editors[i];
        let showpreview = editor.querySelector("[data-showpreview]");
        let clearpreview = editor.querySelector("[data-clearpreview]");
        let preview = editor.querySelector("[data-preview]");
        let rawtext = editor.querySelector("[data-text]");
        let markup = editor.querySelector("[data-markup]");

        if(!(showpreview && clearpreview && preview && rawtext)) {
            console.error("Found markupeditor without required preview skeleton!");
            continue;
        }

        showpreview.onclick = (e) =>
        {
            e.preventDefault();

            var formData = new FormData();
            formData.append("text", rawtext.textContent);
            if(markup) formData.append("markup", markup);

            fetch(SBSBASEURL + "/widget/contentpreview", {
                method: "POST",
                body: formData
            })
                .then((response) => response.text())
                .then((text) => {
                    var parser = new DOMParser();
                    var doc = parser.parseFromString(text, 'text/html');
                    var result = doc.body.firstElementChild;
                    preview.appendChild(result);
                });

            preview.style = "";
            clearpreview.style = "";
        };

        clearpreview.onclick = (e) =>
        {
            e.preventDefault();
            preview.style.display = "none";
            clearpreview.style.display = "none";
        };

        //Now that we're setup, unhide the showpreview button
        showpreview.style = "";
    }
}

function upgrade_markup(element)
{
    element = element || document;
    var markups = element.querySelectorAll(".content[data-markup]:not([data-prerendered]");
    for(var i = 0; i < markups.length; i++)
    {
        Markup.convert_lang(markups[i].textContent, markups[i].getAttribute("data-markup") || "plaintext", markups[i]);
    }
}

function upgrade_code(element)
{
    element = element || document;
    var codes = element.querySelectorAll(".content .code");
    upgrade_code_general(codes);
    codes = element.querySelectorAll(".Markup pre"); //This is what 12y considers code (ugh don't use just pre!!)
    upgrade_code_general(codes);
}

function upgrade_code_general(codes)
{
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
    for(var i = 0; i < deleteconfirms.length; i++)
    {
        let deleteInput = deleteconfirms[i];
        let confirmText = "Are you sure you want to delete " + deleteInput.getAttribute("data-confirmdelete") + "?";
        deleteInput.removeAttribute("data-confirmdelete");
        deleteInput.onclick = (e) => {
            e.preventDefault();
            if (confirm(confirmText))
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