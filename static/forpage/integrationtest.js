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

function randomTitle()
{
    return randomUsername(); //For now, this is good enough
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

//Post a form on an ALREADY LOADED iframe. The form is expected to be pre-filled
function postIframe(form, callback)
{
    console.log("📫 Posting form " + form.getAttribute("id"));
    //Don't let anything block the load event. Don't depend on this though, it may change
    testframe.onload = nonblockingCallback(callback);
    submit_regular_form(form);
}

//Post a form on an ALREADY LOADED iframe, but go find it and apply some data first. 
//If no data is supplied, the function skips applying but still functions normally
function postIframeData(formId, formObj, callback)
{
    var form = testframe.contentWindow.document.getElementById(formId);
    if(!form) throw "Couldn't find form with id " + formId;
    if(formObj) apply_to_form(formObj, form);
    postIframe(form, callback);
}

//First load, then post a form on the loaded iframe using the given information
function loadAndPostIframe(relativeUrl, formId, formObj, callback)
{
    loadIframe(relativeUrl, () => postIframeData(formId, formObj, callback));
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

//Run a function on each of the results of the given query selector (against the
//currently loaded iframe)
function selectorEach(path, func)
{
    var results = testframe.contentWindow.document.querySelectorAll(path);
    for(var i = 0; i < results.length; i++)
        func(results[i], i);
    return results.length;
}

function selectorSingle(path)
{
    return testframe.contentWindow.document.querySelector(path);
}

function assertTrue(value, message) { if(!value) throw (message || "Expected to be true"); }
function assertFalse(value, message) { if(value) throw (message || "Expected to be false"); }
function assertEqual(a, b) { assertTrue(a === b, `${a} does not equal ${b}`); }

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
    var parsedPath = path.match(/https?:\/\//) ? new URL(path).pathname : path;
    var iframePath = testframe.contentWindow.document.location.pathname;
    if(iframePath !== parsedPath) throw `Expected iframe to be at ${parsedPath} but it was at ${iframePath}`
}

function assertAtPathQuery(path)
{
    if(path.match(/https?:\/\//))
    {
        var url = new URL(path);
        path = url.pathname + url.search;
    }
    var iloc = testframe.contentWindow.document.location;
    var iframePath = iloc.pathname + iloc.search;
    if(iframePath !== path) throw `Expected iframe to be at ${path} but it was at ${iframePath}`
}

function assertLocationRegex(regex)
{
    var iframePath = testframe.contentWindow.location.href; //FULL path!
    if(!iframePath.match(regex)) throw `Expected iframe to match ${regex} but it was at ${iframePath}`;
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
    fetch(`${apiend}/user/debug/getregistrationcodebyusername/${currentTestUser.username}`)
        .then(r => r.text()).then(d => {
            var form = testframe.contentWindow.document.getElementById("complete_form");
            apply_to_form({"key":d}, form);
            postIframe(form, cb);
        });
}

// ---------------------------------
// ** THE REST ARE ALL THE TESTS! **
// ---------------------------------

//NOTE: we have some globals here; they are basically constants but we don't know what they are until
//we inspect the page. They become available after certain tests are run, which makes the whole thing
//very fragile. Consider changing this somehow in the future (I don't care right now)
var sbs_categories = [];    //The HASHES for all sbs categories listed in the main forum page
var base_category = {};     //The category to run thread tests agains (such as newthread/etc) 
var newthread_link = "";    //The link to get to someplace which lets us create a new thread in a safe space
var newthread_edit_link = ""; //The link to edit the new thread
var newthread_post_link = ""; //Link to post new post on new thread
var random_submission_link = "/forum/thread/shadow-men"; //link to some random submission

var newthread_data = {
    title: randomTitle(), //This title should be 1-1 translatable to a hash
    post: "this is just some [b]random[/b] post\ni don't care"
};

//The title may change, but the hash should not
var newthread_hash = newthread_data.title;

var newpost_data = {
    post: "a brand new post! [u]underlines!"
};

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
        [ forum_main_tests, (cb) => loadIframe("/forum", cb)],
        [ forum_category_tests, (cb) => loadIframe(base_category.link, cb)], //NOTE: HAVE to do forummain tests first, as they populate the sbs_category array!
        [ forum_newthread_form_tests, (cb) => loadIframe(newthread_link, cb)], //Just check the form itself
        [ forum_newthread_tests, (cb) => postIframeData("threadedit_form", newthread_data, cb)], //We're already on the right page, so just post and check
        [ forum_newthread_general_tests, (cb) => {
            //Update the thread data to reflect the edited title
            newthread_data.title = newthread_data.title.replace("test", "tesw");
            loadAndPostIframe(newthread_edit_link, "threadedit_form", { title: newthread_data.title }, cb);
        }],
        [ forum_newpost_form_tests, (cb) => loadIframe(newthread_post_link, cb) ],
        [ forum_newpost_tests, (cb) => postIframeData("postedit_form", newpost_data, cb)], //We're already on the right page, so just post and check
        [ random_submission_tests, (cb) => loadIframe(random_submission_link, cb)],
        [ program_editor_general_tests, (cb) => loadIframe("/page/edit?mode=program", cb)],
        [ resource_editor_general_tests, (cb) => loadIframe("/page/edit?mode=resource", cb)],
        [ ptc_editor_general_tests, (cb) => loadIframe("/page/edit?mode=ptc", cb)],
        [ documentation_editor_general_tests, (cb) => loadIframe("/page/edit?mode=documentation", cb)],
        [ documentation_general_tests, (cb) => loadIframe("/documentation", cb)],
        [ documentation_knownpage_tests, (cb) => loadIframe("/forum/thread/docs-sb4-constants", cb)], //fragile, oh well
        //This should normally come WAY later, after you are FULLY done with the 'currentTestUser', so add other tests to do with 
        //the actual currentTestUser above this.
        [ register_confirm_tests, (cb) => {
            resetCurrentTestUser();
            loadAndPostIframe("/register", "register_form", currentUserToForm(), cb);
        }],
        [ register_resend_tests, (cb) => postIframeData("resend_form", null, cb)],
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

// Test the main forum list. This populates the globals "sbs_categories" and "base_category"
function forum_main_tests()
{
    test("at_categories", () => assertAtPath("/forum"));
    sbs_categories = [];
    selectorEach(".categoryinfo h1 a", (e, i) =>
    {
        let href = e.getAttribute("href");
        sbs_categories.push({
            link: href,
            title: e.textContent,
            id: e.getAttribute("title").match(/\d+/)[0],
            hash: href.match(/\/([^\/]*)$/)[1]
        });
    });
    test("found_categories", () => assertTrue(sbs_categories.length > 4, `Not enough categories found! (${sbs_categories.length}, expected > 4)`))
    base_category = sbs_categories[0];
}

// Test against one of the categories. This populates the global "newthread_link"
function forum_category_tests()
{
    test("at_firstcategory", () => assertAtPath(base_category.link));

    //Instead of using the normal test, we go query for the new thread link and perform
    //other kinds of tests on it (because we need the value within)
    var newThread = selectorSingle("#newthread");
    test("has_newthread", () => assertTrue(newThread, "Couldn't find newthread link!"));
    newthread_link = newThread.getAttribute("href");
}

function forum_newthread_form_tests()
{
    test("at_newthreadform", () => assertAtPathQuery(newthread_link));
    test("has_categorytitle", () => assertExists(`//h1[contains(text(), "${base_category.title}")]`));
    test("has_categoryinput", () => assertExists(`//form/input[@name="parent_id" and @value="${base_category.id}"]`));
}

function forum_newthread_general_tests()
{
    test("has_title", () => assertExists(`//section/h1[text()="${newthread_data.title}"]`));
    var piece = newthread_data.post.match(/^([^\[]*)\[/)[1]; 
    test("has_post", () => assertExists(`//div[contains(@class,"post")]//div[contains(@class,"content") and contains(text(),"${piece}")]`));
    //Because we're logged in AND it's our thread, we should be able to edit it. BUT we should NOT be able to delete it
    test("has_editthread", () => assertExists("#editthread"));
    test("has_newpost", () => assertExists("#createpost"));
    test("missing_deletethread", () => assertNotExists("#deletethread"));
    test("first_post_editable", () => assertExists(`/descendant::div[contains(@class,"post")][1]//a[contains(@class,"postedit")]`)); // descendant::bar[1]));
    test("first_post_replyable", () => assertExists(`/descendant::div[contains(@class,"post")][1]//a[contains(@class,"postreply")]`)); // descendant::bar[1]));
    test("first_post_deletable", () => assertExists(`/descendant::div[contains(@class,"post")][1]//form[contains(@class,"postdelete")]`)); // descendant::bar[1]));
}

function forum_newthread_tests()
{
    test("at_newthread", () => assertLocationRegex(new RegExp(`/forum/thread/${newthread_hash}/\\d+#(.*)$`)));
    forum_newthread_general_tests();
    var editThread = selectorSingle("#editthread");
    newthread_edit_link = editThread.getAttribute("href");
    var createPost = selectorSingle("#createpost");
    newthread_post_link = createPost.getAttribute("src").replace("widget", "notwidget"); //createPost.getAttribute("href");
}

function forum_newpost_form_tests()
{
    test("at_newpostform", () => assertAtPathQuery(newthread_post_link));
    test("has_threadtitle", () => assertExists(`//h1[contains(text(), "${newthread_data.title}")]`));
    test("has_threadinput", () => assertExists(`//form/input[@name="content_id"]`));
}

function forum_newpost_tests()
{
    test("at_newthread", () => assertLocationRegex(new RegExp(`/forum/thread/${newthread_hash}/\\d+#(.*)$`)));
    forum_newthread_general_tests();
    var piece = newpost_data.post.match(/^([^\[]*)\[/)[1]; 
    test("has_post", () => assertExists(`//div[contains(@class,"post")]//div[contains(@class,"content") and contains(text(),"${piece}")]`));
}

function random_submission_tests()
{
    test("at_submission", () => assertAtPath(random_submission_link));
    test("no_editthread", () => assertNotExists("#editthread"));
    test("no_deletethread", () => assertNotExists("#deletethread"));
    test("has_newpost", () => assertExists("#createpost"));
    test("first_post_replyable", () => assertExists(`/descendant::div[contains(@class,"post")][1]//a[contains(@class,"postreply")]`)); // descendant::bar[1]));
    test("first_post_noedit", () => assertNotExists(`/descendant::div[contains(@class,"post")][1]//a[contains(@class,"postedit")]`)); // descendant::bar[1]));
    test("first_post_nodelete", () => assertNotExists(`/descendant::div[contains(@class,"post")][1]//form[contains(@class,"postdelete")]`)); // descendant::bar[1]));
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

function editor_general_test_base(type)
{
    test("has_id", () => assertExists("#pageedit_id"));
    test("has_subtype", () => assertExists("#pageedit_subtype"));
    //The subtype for ptc is still program
    test("subtype_equals", () => assertExists(`//input[@id="pageedit_subtype" and @value="${(type === "ptc" ? "program" : type)}"]`));
    test("has_title", () => assertExists("#pageedit_title"));
    test("has_tagline", () => assertExists("#pageedit_tagline"));
    test("has_text", () => assertExists("#pageedit_text"));
    test("has_keywords", () => assertExists("#pageedit_keywords"));

    //Stuff specific to documentation
    if(type === "documentation") 
    {
        test("has_markup", () => assertExists("#pageedit_markup"));
        test("has_docpath", () => assertExists("#pageedit_docpath"));
        test("has_hash", () => assertExists("#pageedit_hash"));
        test("no_images", () => assertNotExists("#pageedit_images"));
        test("no_categories", () => assertNotExists("#pageedit_categories"));
    }
    else 
    {
        test("no_markup", () => assertNotExists("#pageedit_markup"));
        test("no_docpath", () => assertNotExists("#pageedit_docpath"));
        test("no_hash", () => assertNotExists("#pageedit_hash"));
        test("has_images", () => assertExists("#pageedit_images"));
        test("has_categories", () => assertExists("#pageedit_categories"));
    }

    //Stuff specific to arbitrary "program" pages (including ptc)
    if(type === "ptc" || type === "program") {
        test("has_systems", () => assertExists("#pageedit_systems"));
        test("has_version", () => assertExists("#pageedit_version"));
        test("has_size", () => assertExists("#pageedit_size"));
    }
    else {
        test("no_systems", () => assertNotExists("#pageedit_systems"));
        test("no_version", () => assertNotExists("#pageedit_version"));
        test("no_size", () => assertNotExists("#pageedit_size"));
    }

    if(type === "ptc")
    {
        test("system_equals", () => assertExists(`//input[@id="pageedit_systems" and @value="${type}"]`));
        test("has_newfile", () => assertExists("#pageedit_newfile"));
        test("has_filelist", () => assertExists("#ptc_file_list"));
        //This is the important one
        test("has_ptc_files", () => assertExists("#pageedit_ptc_files"));
    }
    else
    {
        test("no_newfile", () => assertNotExists("#pageedit_newfile"));
        test("no_filelist", () => assertNotExists("#ptc_file_list"));
        //This is the important one
        test("no_ptc_files", () => assertNotExists("#pageedit_ptc_files"));
    }
}

function program_editor_general_tests() { editor_general_test_base("program") }
    //test("at_neweditor", () => assertAtPathQuery("/page/edit?mode=program"));
function resource_editor_general_tests() { editor_general_test_base("resource") }
function ptc_editor_general_tests() { editor_general_test_base("ptc") }
function documentation_editor_general_tests() { editor_general_test_base("documentation") }

function documentation_general_tests()
{
    test("no_newdoc", () => assertNotExists("#newdocumentation"));
}

function documentation_knownpage_tests()
{
    test("no_editdoc", () => assertNotExists("#editpage"));
    test("no_deletedoc", () => assertNotExists("#deletepage"));
}