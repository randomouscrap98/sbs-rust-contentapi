lazy_iframes();
make_all_galleries();

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
