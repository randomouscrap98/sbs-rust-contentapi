var mode = pageedit_form.getAttribute("data-mode");
var SYSTEMCHECKLISTID = "systems_checklist";
var CATEGORYCHECKLISTID = "categories_checklist";

pageedit_form.onsubmit = editor_onsubmit;

//These are safe to call, they don't do anything if the respective elements don't exist
fix_systems(); 
fix_categories(); 

if (mode === "ptc") {
    console.log("Setting up PTC controls");
    //script is defer, put all function calls right here in the script
    pageedit_newfile.addEventListener("change", added_file);
    ptc_files_refresh.onclick = refresh_raw_ptc_list;

    //Need to parse whatever was originally in the raw data and create
    //elements from it. We do this last just in case it fails? Should just 
    //add try catch
    preparse_ptc_list();
}
else {
    console.log("Not setting up PTC controls (not in ptc mode)");
}

function editor_onsubmit()
{
    var input = pageedit_form.querySelector('input[type="submit"]');
    input.setAttribute("disabled", "");
    var result = editor_onsubmit_check();
    if(!result) input.removeAttribute("disabled");
    return result;
}

function editor_onsubmit_check()
{
    if (mode === "ptc") {
        if(!refresh_raw_ptc_list()) {
            alert("You must upload at least one PTC file!")
            return false;
        }
    }
    var systems_checklist = document.getElementById(SYSTEMCHECKLISTID);
    if(systems_checklist) pageedit_systems.value = systems_checklist.to_list();
    if(!pageedit_systems.value) {
        alert("You must select at least one system!");
        return false;
    }
    var categories_checklist = document.getElementById(CATEGORYCHECKLISTID);
    if(categories_checklist) pageedit_categories.value = categories_checklist.to_list();
    return true;
}

function fix_systems()
{
    if(!document.getElementById("systems_table")) return;
    pageedit_systems.setAttribute("type", "hidden");
    //if(document.getElementById("systems_instructions")) 
    systems_instructions.style.display = "none";
    var systems = JSON.parse(systems_table.getAttribute("data-raw"));
    var checklist = make_checklist(systems, pageedit_systems.value, SYSTEMCHECKLISTID);
    pageedit_systems.parentNode.insertBefore(checklist, pageedit_systems);
    //if(document.getElementById("ptc_editor_aside")) 
    checklist.parentNode.insertBefore(ptc_editor_aside, checklist.nextElementSibling);
}

function fix_categories()
{
    if(!document.getElementById("categories_table")) return;
    pageedit_categories.setAttribute("type", "hidden");
    categories_instructions.style.display = "none";
    var categories = JSON.parse(categories_table.getAttribute("data-raw"));
    pageedit_categories.parentNode.insertBefore(make_checklist(categories, pageedit_categories.value, CATEGORYCHECKLISTID), pageedit_categories);
}

//Data should be an array of arrays, unfortunately?
function make_checklist(data, original, id)
{
    var container = document.createElement("div");
    container.className = "checklist";
    container.setAttribute("id", id);

    var values = original.split(" ").filter(x => x);

    for(var i = 0; i < data.length; i++)
    {
        var checkContainer = document.createElement("div");
        checkContainer.className = "checkitem";
        var label = document.createElement("label");
        var input = document.createElement("input");
        input.setAttribute("type", "checkbox");
        input.setAttribute("value", data[i][0]);
        input.checked = values.includes(data[i][0]);
        var span = document.createElement("span");
        span.className = "checkname";
        span.textContent = data[i][1];
        label.appendChild(input);
        label.appendChild(span);
        checkContainer.appendChild(label);
        container.appendChild(checkContainer);
    }

    container.to_list = function()
    {
        var result = "";
        var all_checks = container.querySelectorAll("input:checked");
        for(var i = 0; i < all_checks.length; i++)
        {
            result += all_checks[i].value + " ";
        }
        return result;
    };

    return container;
}

function refresh_raw_ptc_list()
{
    //need to get all the ptcfiles and pull the data out
    var elements = ptc_file_list.querySelectorAll(".ptcfile") 
    var result = [];
    for(var i = 0; i < elements.length; i++)
        result.push(elements[i].getData());
    pageedit_ptc_files.textContent = JSON.stringify(result);
    return result.length;
}

function preparse_ptc_list() 
{
    var original = pageedit_ptc_files.value ? JSON.parse(pageedit_ptc_files.value) : [];
    for(var i = 0; i < original.length; i++)
    {
        //data should be enough to naturally create elements
        ptc_file_list.appendChild(create_ptc_element(original[i]));
    }
}

function added_file() 
{
    console.log("File(s) added, processing");

    for(var i = 0 ; i < pageedit_newfile.files.length; i++) 
    {
        let file = pageedit_newfile.files[i];
        let reader = new FileReader();
        reader.addEventListener('load', process_newfile);
        reader.readAsArrayBuffer(file);
    }

    pageedit_newfile.value = null;
}

function parse_sdfile(arrbuf)
{
    var result = { name : "", raw: "" };
    var byteview = new Uint8Array(arrbuf);

    //4 bytes header 
    //4 bytes filesize
    //4 bytes unknown
    //8 bytes name
    //16 bytes md5
    for(var i = 12; i < 20; i++)
    {
        if (byteview[i] == 0) 
            break;
        else
            result.name += String.fromCharCode(byteview[i]);
    }

    for(var i = 36; i < byteview.byteLength; i++)
    {
        result.raw += String.fromCharCode(byteview[i]);
    }

    result.base64 = window.btoa(result.raw);

    return result;
}

function process_newfile(event)
{
    console.log("Got something: ", event.target.result);
    var parse = parse_sdfile(event.target.result);
    console.log(`Parsed file: ${parse.name}`);
    ptc_file_list.appendChild(create_ptc_element(parse));
}

function create_ptc_element(parsed_data)
{
    var container = document.createElement("div");
    container.className = "ptcfile";
    container.setAttribute("data-data", parsed_data.base64);

    var name = document.createElement("input");
    name.className = "ptcname";
    name.placeholder = "Filename";
    name.value = parsed_data.name;

    var description = document.createElement("textarea");
    description.className = "ptcdescription";
    description.placeholder = "Description of file (optional)";
    if(parsed_data.description)
        description.textContent = parsed_data.description;

    var upbutton = document.createElement("button");
    upbutton.setAttribute("type", "button");
    upbutton.textContent = "▲";
    upbutton.title = "Move up";
    upbutton.onclick = function () {
        var previous = container.previousElementSibling;
        if(previous) { container.parentElement.insertBefore(container, previous); }
    };

    var downbutton = document.createElement("button");
    downbutton.setAttribute("type", "button");
    downbutton.textContent = "▼";
    downbutton.title = "Move down";
    downbutton.onclick = function () {
        var next = container.nextElementSibling;
        if(next) { container.parentElement.insertBefore(container, next.nextElementSibling); }
    };

    var deletebutton = document.createElement("button");
    deletebutton.setAttribute("type", "button");
    deletebutton.textContent = "✖";
    deletebutton.textContent = "Delete";
    deletebutton.onclick = function () {
        container.parentNode.removeChild(container);
    };

    var topcontainer = document.createElement("div");
    topcontainer.className = "topline";
    topcontainer.appendChild(name);
    topcontainer.appendChild(upbutton);
    topcontainer.appendChild(downbutton);
    topcontainer.appendChild(deletebutton);

    container.appendChild(topcontainer);
    container.appendChild(description);

    container.getData = function () {
        return {
            base64 : container.getAttribute('data-data'),
            name : name.value,
            description : description.value
        };
    };

    return container;
}
