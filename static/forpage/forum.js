stop_iframes();

function stop_iframes()
{
    var replychains = document.querySelectorAll('.repliesview');
    for(var i = 0; i < replychains.length; i++)
    {
        var el = replychains[i];
        el.style = "";
        el.addEventListener("toggle", load_inner_iframe);
        //iframes[i].setAttribute("data-src", iframes[i].src);
        //iframes[i].src = ""; //[i].addEventListener("submit", stdform_onsubmit);
    }
}
