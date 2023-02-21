//Currently checks every 90 seconds, should maybe be fine considering our tiny traffic.
var NEWACTIVITYINTERVAL = 90 * 1000;
var ALERTID = "activityalert";
var originalTitle = document.title;

if(document.getElementById("newlinkplaceholder"))
{
    setInterval(() => check_new_activity(newlinkplaceholder.textContent), NEWACTIVITYINTERVAL);
}

function check_new_activity(link)
{
    fetch(link)
        .then((response) => response.text())
        .then((text) => {
            var parser = new DOMParser();
            var doc = parser.parseFromString(text, 'text/html');
            var newactivity = doc.querySelectorAll("#activitylist .activity");
            if(newactivity.length > 0)
            {
                make_or_update_alert(newactivity.length);
            }
        });
}

function make_or_update_alert(amount)
{
    var existing = document.getElementById(ALERTID);
    if(!existing)
    {
        existing = create_new_activity_alert();
        activitylist.insertBefore(existing, activitylist.firstElementChild);
    }
    existing.textContent = `${amount} new event` + (amount > 1 ? "s!" : "!");
    document.title = `(${amount}) ${originalTitle}`;
}

function create_new_activity_alert()
{
    var result = document.createElement("a");
    result.setAttribute("href", mainactivitylink.href);
    result.setAttribute("id", ALERTID);
    return result;
}
