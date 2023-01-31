// The "testframe" iframe element should always be available as a variable

// Set this to have something run on completion of iframe load
var pendingTestOnload = false; //next onload
var pendingTestLoad = []; //all pending tests
var currentSuite = "prepping";
var currentTest = "none";

window.onerror = function(msg, url, line)
{
    alert(`Test '${currentSuite}:${currentTest}' failed on line ${line}: ${msg}`);
};

function stageLoad(relativeUrl, suite, tests)
{
    pendingTestLoad.push({
        callback : tests,
        suite: suite,
        url: relativeUrl
    });
}

//The function called when the iframe finishes loading. It calls whatever the currently pending
//completion function is (usually the next set of assertions)
function testonload()
{
    console.log("test iframe loaded");

    if(pendingTestOnload)
    {
        console.log("Calling callback");
        pendingTestOnload();
        pendingTestOnload = false;
    }

    //If there are still leftovers, load the next one
    if(pendingTestLoad.length)
    {
        var next = pendingTestLoad.shift();
        console.log("Loading next pending page " + next.url);
        currentSuite = next.suite;
        pendingTestOnload = next.callback;
        testframe.src = next.url;
    }
}

////Load a page with GET and call the given callback after it's fully loaded (maybe?)
//function getPage(relativeUrl, callback)
//{
//    pendingTestOnload.push({callback:callback,url:relativeUrl});
//    loadNextPage();
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
    if(path.indexOf("#") == 0) result = document.getElementById(path.substr(1));
    else result = xpath(`count(${path})`).numberValue > 0;
    if(exists && !result) throw `Expected ${path} to exist; it did not!`;
    else if(!exists && result) throw `Expected ${path} not to exist, it did!`;
}

function assertExists(path) { return assertExistsGeneric(path, true); }
function assertNotExists(path)  { return assertExistsGeneric(path, false); }

// ---------------------------------
// ** THE REST ARE ALL THE TESTS! **
// ---------------------------------

function runtests()
{
    //Will make this better later; later loads must happen after previous for now
    stageLoad("/", "root_loaded", root_tests);
    stageLoad("/login", "login_loaded", login_tests);
    testonload(); //Initiate the tests by calling the recursive iframe onload callback
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