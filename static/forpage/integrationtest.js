// The "testframe" iframe element should always be available as a variable

// This probably isn't necessary tbh
//window.addEventListener("load", function()
//{
//    var testStartButton = document.getElementsById("teststart");
//    testStartButton.addEventListener("click", function(e) { 
//        e.preventDefault(); 
//        runAllTests(); 
//    })
//    testStartButton.textContent = "Run tests";
//});

var pendingTestOnload = false;

function testonload()
{
    console.log("test iframe loaded");

    if(pendingTestOnload)
    {
        console.log("Calling callback");
        pendingTestOnload = false;
    }
}

function runtests()
{

}

function runAllTests()
{
    console.log(testframe);
}