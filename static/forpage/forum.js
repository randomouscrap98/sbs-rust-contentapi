lazy_iframes();
make_all_galleries();

upgrade_edits();
upgrade_replies();

function lazy_iframes()
{
    var replychains = document.querySelectorAll('.repliesview');
    for(var i = 0; i < replychains.length; i++)
    {
        var el = replychains[i];
        el.style = "";
        el.addEventListener("toggle", load_inner_iframe);
    }
}

function make_all_galleries() {
    var galleries = document.querySelectorAll(".gallery");
    for(var i = 0; i < galleries.length; i++)
        make_gallery(galleries[i]);
}

function make_gallery(gallery) {
    var image = gallery.querySelector("img");
    if(image) {
        var images = JSON.parse(gallery.getAttribute("data-images"));
        var index = 0;
        image.style.cursor = "pointer";
        image.addEventListener('click', function() {
            index++;
            image.src = images[index % images.length];
        })
    }
    else {
        console.warn("No image found for gallery; not setting up");
    }
}

function upgrade_edits()
{
    var edit_links = document.querySelectorAll(".postedit");
    for(var i = 0; i < edit_links.length; i++)
    {
        let linkelem = edit_links[i];
        let link = linkelem.href;
        let postId = linkelem.getAttribute("data-postid");
        let origText = linkelem.textContent;
        linkelem.href = "#";
        linkelem.onclick = (e) =>
        {
            e.preventDefault();
            var existing = document.querySelector(`.postwidget[data-postid="${postId}"]`);
            var content = document.querySelector(`.content[data-postid="${postId}"]`);
            var replyelem = document.querySelector(`.postreply[data-postid="${postId}"]`);
            if(existing) {
                //If the editor is already there, redo the whatever
                linkelem.textContent = origText;
                linkelem.className = linkelem.className.replace("coolbutton", "flatlink");
                existing.parentNode.removeChild(existing);
                content.removeAttribute("style");
                replyelem.removeAttribute("style");
            }
            else {
                existing = document.createElement("iframe");
                existing.setAttribute('src', link + "&widget=true");
                existing.setAttribute("data-postid", postId);
                existing.className = "postwidget";
                content.parentNode.insertBefore(existing, content);
                content.style.display = "none";
                replyelem.style.display = "none";
                linkelem.className = linkelem.className.replace("flatlink", "coolbutton");
                linkelem.textContent = "Quit Editing";
            }
        };
    }
}

function upgrade_replies()
{
    var reply_links = document.querySelectorAll(".postreply");
    for(var i = 0; i < reply_links.length; i++)
    {
        let linkelem = reply_links[i];
        let link = linkelem.href;
        let postId = linkelem.getAttribute("data-postid");
        let origText = linkelem.textContent;
        linkelem.href = `#replywidget-${postId}`;
        linkelem.onclick = (e) =>
        {
            //e.preventDefault();
            var existing = document.querySelector(`.replybox[data-postid="${postId}"]`);
            var content = document.querySelector(`.content[data-postid="${postId}"]`);
            var editelem = document.querySelector(`.postedit[data-postid="${postId}"]`);
            if(existing) {
                //If the editor is already there, we need to reset everything
                linkelem.textContent = origText;
                linkelem.className = linkelem.className.replace("coolbutton", "flatlink");
                existing.parentNode.removeChild(existing);
                editelem.removeAttribute("style");
            }
            else {
                existing = document.createElement("div");
                existing.className = "replybox";
                existing.setAttribute("id", `replywidget-${postId}`);
                existing.setAttribute("data-postid", postId);
                var hr = document.createElement("hr");
                hr.className = "smaller";
                var widget = document.createElement("iframe");
                widget.setAttribute('src', link + "&widget=true");
                widget.className = "postwidget";
                existing.appendChild(hr);
                existing.appendChild(widget);

                //Here, we put the reply AFTER the post, then remove the edit element.
                content.parentNode.insertBefore(existing, content.nextElementSibling);
                editelem.style.display = "none";
                linkelem.className = linkelem.className.replace("flatlink", "coolbutton");
                linkelem.textContent = "Quit Replying";
            }
        };
    }
}