// The "testframe" iframe element should always be available as a variable

var apiend = "http://localhost:5000/api";

//fetch("http://localhost:5000/api/user/getregistrationcodebyusername/test940647490084").then(r => r.text()).then(d => console.log(d));


//We set these to make error reporting easier(?)
var currentSuite = "?";
var currentTest = "?";
var currentTestUser = {};

window.onload = function()
{
    var testButton = document.getElementById("teststart");
    if(testButton)
    {
        testButton.onclick = () => runtests();
        testButton.textContent = "Run all tests";
    }
};

window.onerror = function(msg, url, line)
{
    alert(`Test '${currentSuite}:${currentTest}' failed on line ${line}: ${msg}`);
};

function randomUsername()
{
    var rnd = String(Math.random());
    return ("test" + rnd.substring(rnd.indexOf(".") + 1)).substring(0, 16); //Just in case
}

//function chainCallback(cb1, cb2) { return () => { cb1(); cb2(); } }
function nonblockingCallback(callback) { return () => setTimeout(callback, 0); }

//Just load a page with the onload event set to the given callback. Should have NO concept of 
//chained tests or anything
function loadIframe(relativeUrl, callback)
{
    console.log("📑 Loading page " + relativeUrl);
    //Don't let anything block the load event. Don't depend on this though, it may change
    testframe.onload = nonblockingCallback(callback);
    testframe.src = relativeUrl;
}

//Post a form on an ALREADY LOADED iframe.
function postIframe(form, callback)
{
    console.log("📫 Posting form " + form.id);
    //Don't let anything block the load event. Don't depend on this though, it may change
    testframe.onload = nonblockingCallback(callback);
    submit_regular_form(form);
}

//First load, then post a form on the loaded iframe using the given information
function loadAndPostIframe(relativeUrl, formId, formObj, callback)
{
    loadIframe(relativeUrl, () =>
    {
        var form = testframe.contentWindow.document.getElementById(formId);
        if(!form) throw "Couldn't find form with id " + formId;
        apply_to_form(formObj, form);
        postIframe(form, callback);
    });
}

//Perform a test of the given name on the currently loaded iframe
function test(name, assertion)
{
    currentTest = name;
    assertion(); //testframe.contentWindow.document);
    console.log(`✅ ${currentSuite}:${name}`);
}

//Return an xpathresult against the testing iframe for the given xpath
function xpath(xpath)
{
    var idoc = testframe.contentWindow.document;
    return idoc.evaluate(xpath, idoc, null, XPathResult.ANY_TYPE, null);
    //var result = idoc.evaluate(xpath, idoc, namespaceResolver, resultType, result);
}

//Ensure the given xpath does or does not lead to an actual element (look in iframe)
function assertExistsGeneric(path, exists)
{
    var result = false;
    if(path.indexOf("#") == 0) result = testframe.contentWindow.document.getElementById(path.substr(1));
    else result = xpath(`count(${path})`).numberValue > 0;
    if(exists && !result) throw `Expected ${path} to exist; it did not!`;
    else if(!exists && result) throw `Expected ${path} not to exist, it did!`;
}

function assertExists(path) { assertExistsGeneric(path, true); }
function assertNotExists(path)  { assertExistsGeneric(path, false); }

function assertAtPath(path) 
{ 
    var iframePath = testframe.contentWindow.document.location.pathname;
    if(iframePath !== path) throw `Expected iframe to be at ${path} but it was at ${iframePath}`
}

//Because everything is a callback, and we never know what we might be waiting on, this turns a simple array
//of tests to be run in order against the iframe into the proper chained callback, wrapping each callback so
//it calls the next in line at the end of its own execution.
function runChainedTests(testarray)
{
    var nextCb = () => console.log("🎉 All tests complete!");

    //Go backwards, because each callback actually has to call the NEXT callback, so they're
    //all basically getting wrapped. This could instead be some recursive function but whatever.
    for(var i = testarray.length - 1; i >= 0; i--)
    {
        let testfunc = testarray[i][0];
        let testrun = testarray[i][1];
        let thiscb = nextCb;
        nextCb = () => {
            currentSuite = testfunc.name.replaceAll("_tests", "");
            testrun(() =>
            {
                testfunc();     //Run the desired tests first
                thiscb();       //Then run whatever is supposed to come next
            });
        };
    }

    nextCb();
}

function resetCurrentTestUser()
{
    currentTestUser = {
        //token : false,
        username : randomUsername(),
        password : "password"
    };
    currentTestUser.email = currentTestUser.username + "@smilebasicsource.com";
}

function currentUserToForm()
{
    return {
        "username" : currentTestUser.username,
        "email" : currentTestUser.email,
        "password" : currentTestUser.password
    };
}

function completeRegistration(cb)
{
    fetch(`${apiend}/user/getregistrationcodebyusername/${currentTestUser.username}`)
        .then(r => r.text()).then(d => {
            var form = testframe.contentWindow.document.getElementById("complete_form");
            apply_to_form({"key":d}, form);
            postIframe(form, cb);
        });
}

// ---------------------------------
// ** THE REST ARE ALL THE TESTS! **
// ---------------------------------

function runtests()
{
    currentSuite = "prepping";
    currentTest = "none";
    resetCurrentTestUser();

    runChainedTests([
        [ root_tests, (cb) => loadIframe("/", cb) ],
        [ login_tests, (cb) => loadIframe("/login", cb) ],
        [ register_confirm_tests, (cb) => loadAndPostIframe("/register", "register_form", currentUserToForm(), cb)],
        [ userhome_tests, completeRegistration ],
        //This should normally come WAY later, after you are FULLY done with the 'currentTestUser', so add other tests to do with 
        //the actual currentTestUser above this.
        [ register_confirm_tests, (cb) => {
            resetCurrentTestUser();
            loadAndPostIframe("/register", "register_form", currentUserToForm(), cb);
        }],
        [ register_resend_tests, (cb) => {
            var form = testframe.contentWindow.document.getElementById("resend_form");
            postIframe(form, cb);
        }],
        [ userhome_tests, completeRegistration ],
        [ root_tests, (cb) => loadIframe("/logout", cb) ], //And then back to the start; root tests already test for not-logged-in
    ]);
}

function root_tests()
{
    test("at_root", () => assertAtPath("/"));
    test("header_by_id", () => assertExists("#header-user"));
    test("header_by_xpath", () => assertExists("/html/body/header/nav"));
    test("user_by_xpath", () => assertExists('//div[@id="header-user"]/a'));
    test("user_not_logged_in", () => assertNotExists('//div[@id="header-user"]/a/img'));
    test("footer_about", () => assertExists("#api_about"));
}

function login_tests()
{
    test("at_login", () => assertAtPath("/login"));
    test("login_selected", () => assertExists('//a[contains(@href,"/login") and contains(@class,"current")]'));
    test("confirm_relink", () => assertExists('//a[contains(@href,"/register/confirm")]'));
}

function register_confirm_tests()
{
    //WARN: Registering does NOT place us at the confirmation page!!
    //test("at_confirm", () => assertAtPath("/register/confirm"));
    test("username_shown", () => assertExists(`//section/p[contains(text(),"${currentTestUser.username}")]`));
    test("email_filled", () => assertExists(`//input[@id="complete_email" and @value="${currentTestUser.email}"]`));
    test("resend_email_filled", () => assertExists(`//input[@id="resend_email" and @value="${currentTestUser.email}"]`));
}

//This may change to "userhome" tests
function userhome_tests()
{
    test("at_userhome", () => assertAtPath("/userhome"));
    test("auto_login", () => assertExists(`//div[@id="header-user"]//span[text()="${currentTestUser.username}"]`));
    //Even at userhome, make sure login is selected
    test("userhome_selected", () => assertExists('//a[contains(@href,"/userhome") and contains(@class,"current")]'));
    //username exist, email exist, logout link exist, userpage exist
    //do NOT update bio! go to userpage first, make sure it shows up ok
    //make sure to upload a file too! maybe...?
}

function register_resend_tests()
{
    //We should still be at the confirmation page after resend
    test("at_confirm", () => assertAtPath("/register/confirm"));
    //Make sure the two email fields STILL have their data!
    test("email_filled", () => assertExists(`//input[@id="complete_email" and @value="${currentTestUser.email}"]`));
    test("resend_email_filled", () => assertExists(`//input[@id="resend_email" and @value="${currentTestUser.email}"]`));
    //But the username is gone. We specifically test for this to ensure the page actually reloaded and data isn't leaking
    test("username_not_shown", () => assertNotExists(`//section/p[contains(text(),"${currentTestUser.username}")]`));
    //And there should be a success message! This may not be in the resend form in the future but...
    test("success_shown", () => assertExists('//form[@id="resend_form"]/p[contains(text(),"resent") and contains(@class,"success")]'));
}
