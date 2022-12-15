function fix_searchform()
{
    var searchType = document.getElementById("search-type");
    var searchCategory = document.getElementById("search-category");

    //This makes sure you can only see categories for the type you selected
    var refresh_categories = function() {
        var options = searchCategory.querySelectorAll("option");
        searchCategory.value = 0;
        for(var i = 0; i < options.length; i++)
        {
            var attr = options[i].getAttribute("data-for");
            if(attr)
            {
                if(attr === searchType.value)
                    options[i].removeAttribute("hidden");
                else
                    options[i].setAttribute("hidden", "");
            }
        }
    }
    refresh_categories();
    searchType.oninput = refresh_categories;
}


fix_searchform();