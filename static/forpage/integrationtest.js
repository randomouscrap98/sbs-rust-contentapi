// The "testframe" iframe element should always be available as a variable

// Set this to have something run on completion of iframe load
//var pendingTestOnload = false; //next onload
//var skipPendingLoad = false;
//var pendingTestLoad = []; //all pending tests
var currentSuite = "?";
var currentTest = "?";
var currentTestUser = {};

window.onload = function()
{
    teststart.onclick = () => runtests();
    teststart.textContent = "Run all tests";
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

////Add a pending page load with callback, generally these will be your main test blocks
//function stageLoad(relativeUrl, suite, tests)
//{
//    pendingTestLoad.push({
//        callback : tests,
//        suite: suite,
//        url: relativeUrl
//    });
//}

////Add a pending page load with form submit after, then run the tests on the newly loaded page
//function stagePost(relativeUrl, suite, formObj, formId, tests)
//{
//    stageLoad(relativeUrl, suite, () => { instagePost(formObj, formId, tests); });
//}

//While within a staged callback, post a form with a callback which has additional tests.
//NOTE: you should NOT perform ANYTHING else after an instagePost!
//function instagePost(formObj, formId, callback)
//{
//    var form = testframe.contentWindow.document.getElementById(formId);
//    if(!form) throw "Couldn't find form with id " + formId;
//    applyForm(formObj, form);
//    skipPendingLoad = true;
//    pendingTestOnload = () => { callback(); skipPendingLoad = false; };
//    submitForm(form);
//}

//The function called when the iframe finishes loading. It calls whatever the currently pending
//completion function is (usually the next set of assertions)
//function testonload()
//{
//    console.log("test iframe loaded");
//
//    if(pendingTestOnload)
//    {
//        console.log("Calling callback");
//        pendingTestOnload();
//        pendingTestOnload = false;
//    }
//
//    //If there are still leftovers, load the next one
//    if(pendingTestLoad.length && !skipPendingLoad)
//    {
//        var next = pendingTestLoad.shift();
//        console.log("Loading next pending page " + next.url);
//        currentSuite = next.suite;
//        pendingTestOnload = next.callback;
//        testframe.src = next.url;
//    }
//}

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

function assertExists(path) { return assertExistsGeneric(path, true); }
function assertNotExists(path)  { return assertExistsGeneric(path, false); }

function runChainedTests(testarray)
{
    var nextCb = null;

    //Go backwards, because each callback actually has to call the NEXT callback, so they're
    //all basically getting wrapped. This could instead be some recursive function but whatever.
    for(var i = testarray.length - 1; i >= 0; i--)
    {
        let testfunc = testarray[i][0];
        let testrun = testarray[i][1];
        let thiscb = nextCb;
        nextCb = () => {
            currentSuite = testfunc.name;
            testrun(() =>
            {
                testfunc();             //Run the desired tests first
                if(thiscb) thiscb();    //Then run whatever is supposed to come next
            });
        };
    }

    nextCb();
}

// ---------------------------------
// ** THE REST ARE ALL THE TESTS! **
// ---------------------------------

function runtests()
{
    currentSuite = "prepping";
    currentTest = "none";
    currentTestUser = {
        token : false,
        username : randomUsername()
    };

    runChainedTests([
        [ root_tests, (cb) => loadIframe("/", cb) ],
        [ login_tests, (cb) => loadIframe("/login", cb) ],
        [ register_step1_tests, (cb) => loadAndPostIframe("/register", "register_form", {
            "username" : currentTestUser.username,
            "email" : currentTestUser.username + "@smilebasicsource.com",
            "password" : "password"
        }, cb)]
    ]);

    ////Will make this better later; later loads must happen after previous for now
    //stageLoad("/", "root_loaded", root_tests);
    //stageLoad("/login", "login_loaded", login_tests);
    //stagePost("/register", "register_step1", {
    //    "username" : currentTestUser.username,
    //    "email" : currentTestUser.username + "@smilebasicsource.com",
    //    "password" : "password"
    //}, "register_form", register_step1_tests)
    //testonload(); //Initiate the tests by calling the recursive iframe onload callback
}

function root_tests()
{
    test("header_by_id", () => assertExists("#header-user"));
    test("header_by_xpath", () => assertExists("/html/body/header/nav"));
    test("user_by_xpath", () => assertExists('//div[@id="header-user"]/a'));
    test("user_not_logged_in", () => assertNotExists('//div[@id="header-user"]/a/img'));
    test("footer_about", () => assertExists("#api_about"));
}

function login_tests()
{
    test("login_selected", () => assertExists('//a[contains(@href,"/login") and contains(@class,"current")]'));
}

function register_step1_tests()
{
    test("username_shown", () => assertExists(`//section/p[contains(text(),"${currentTestUser.username}")]`));
    console.log("Completed the register?");
}