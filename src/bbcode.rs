use regex::{Regex, Captures};

// Carlos Sanchez - 2022-12-05
// - For SBS

//So:
//- early closures close all tags in the previous scope
//- dupes auto close previous scope (just one level)
//- ignore unmatched closing tags
//- close all unclosed tags at the end
//- don't modify whitespace for version 1

//The user provides this
#[derive(Debug, Clone)]
pub struct TagInfo {
    //The tag identity, such as "b", "youtube", etc
    pub tag : &'static str,
    //The tag to put out as html
    pub outtag: &'static str,
    pub tag_type: TagType,
    pub rawextra: Option<&'static str>, //Just dump this directly into the tag at the end. No checks performed
}

impl TagInfo {
    //Constructors for basic tags. Anything else, you're better off just constructing it normally
    fn simple(tag: &'static str) -> TagInfo {
        TagInfo { tag, outtag: tag, tag_type: TagType::Simple, rawextra: None }
    }
    fn start() -> TagInfo {
        TagInfo { tag: "", outtag: "", tag_type: TagType::Start, rawextra: None }
    }
}

//This is the 'silly' part of the parser. Rather than making some actually generic system, I identified some
//standard ways tags are used and just made code around those ways. Probably bad but oh well.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum TagType {
    Start,          //Should ONLY have one of these! It's like S in a grammar!
    Simple,         //Stuff like [b][/b], no args, normal translation (can change tag name still)
    DefinedArg(&'static str),   //CAN have argument defined, attribute name is given
    SelfClosing(&'static str),  //No closing tag, value is 
    DefaultArg(&'static str),   //The tag enclosed value provides a default for the given attribute, or not if defined
}

impl TagType {
    //Another bad thing: rather than defining what tags are allowed in or out of a tag, it's all or nothing.
    //This works for most situations, and it's fairly easy to add some custom list of tags later
    fn is_verbatim(&self) -> bool {
        match self {
            Self::Start => false,
            Self::Simple => false,
            Self::DefinedArg(_) => false,
            Self::SelfClosing(_) => true,
            Self::DefaultArg(_) => true
        }
    }
}

//While "TagType" determines how the tag functions at a lower level (such as how it handles arguments), 
//this determines how the whole block functions on a greater level. They define how scopes and whole blocks
//of text move into the output
#[derive(Debug, Clone)]
pub enum MatchType { 
    Passthrough,    //Pass this junk right out as-is
    Open(TagInfo),  //this is so small, it's fine to dupe in open/close
    Close(TagInfo),
    DirectReplace(&'static str)
}

//Definition for a block level matcher. Analogous to "TypeInfo" but for the greater scope. Should always be
//readonly, it is just a definition. Not necessary a tag element, could define eating garbage, escape chars, etc.
#[derive(Debug, Clone)]
pub struct MatchInfo {
    pub regex : Regex,
    pub match_type: MatchType,
}

//For the following section, it is assumed the entire scoper and all elements within will have the same lifetime
//as the calling function, which also houses the input string.

//A scope for bbcode tags. Scopes increase and decrease as tags are opened and closed. Scopes are placed on a stack
//to aid with auto-closing tags
struct BBScope<'a> {
    info: &'a TagInfo,
    inner_start: usize, //TERRIBLE! MAYBE?!
    has_arg: bool, //May need to change if more configuration needed
}

//A container with functions to help manage scopes. It doesn't understand what bbcode is or how the tags should
//be formatted, it just handles pushing and popping scopes on the stack
struct BBScoper<'a> {
    scopes : Vec<BBScope<'a>>
}

//Everything inside BBScoper expects to live as long as the object itself. So everything is 'a
impl<'a> BBScoper<'a> 
{
    fn new() -> Self { BBScoper { scopes: Vec::new() }}

    //Add a scope, which may close some existing scopes. The closed scopes are returned in display order.
    //NOTE: the added infos must live as long as the scope container!
    fn add_scope(&mut self, scope: BBScope<'a>) -> (&BBScope, Vec<BBScope<'a>>) {
        //here we assume all taginfos have unique tags because why wouldn't they
        let mut result = Vec::new();
        if let Some(topinfo) = self.scopes.last() {
            if topinfo.info.tag == scope.info.tag {
                //It's the same, close the last one
                if let Some(scope) = self.scopes.pop() {
                    result.push(scope);
                }
                else {
                    println!("BBScoper::add_scope HOW DID THIS HAPPEN? There were scopes from .last but pop returned none!");
                }
            }
        }

        self.scopes.push(scope);
        (self.scopes.last().unwrap(), result) //Kind of a silly return type, might change it later
    }
    
    //Close the given scope, which should return the scopes that got closed (including the self).
    //If no scope could be found, the vector is empty
    fn close_scope(&mut self, info: &'a TagInfo) -> Vec<BBScope<'a>> {
        let mut scope_count = 0;
        let mut tag_found = false;

        //Scan backwards, counting. Stop when you find a matching tag. This lets us know the open child scopes
        //that were not closed
        for scope in self.scopes.iter().rev() {
            scope_count += 1;
            if info.tag == scope.info.tag {
                tag_found = true;
                break;
            }
        }

        //Return all the scopes from the end to the found closed scope. Oh and also remove them
        if tag_found {
            let mut result = Vec::with_capacity(scope_count + 1);
            for _i in 0..scope_count {
                if let Some(scope) = self.scopes.pop() {
                    result.push(scope);
                }
                else {
                    println!("BBScope::close_scope LOGIC ERROR: SCANNED PAST END OF SCOPELIST");
                }
            }
            result
        }
        else {
            Vec::new()
        }
    }

    //Consume the scope system while dumping the rest of the scopes in the right order for display
    fn dump_remaining(self) -> Vec<BBScope<'a>> {
        self.scopes.into_iter().rev().collect()
    }
}


// ------------------------------
// *     MAIN FUNCTIONALITY    *
// ------------------------------

//The main bbcode system. You create this to parse bbcode!
pub struct BBCode {
    //These are SOMETIMES processed (based on context)
    pub tags : Vec<MatchInfo>
}

impl BBCode {
    //Maybe get rid of anyhow if you want to separate this, kind of a big thing to include.
    //Anyway, this build function precompiles all the regex for you. Try to reuse this item
    //as much as possible if you don't want to incur the compile times. Also, you need to pass
    //ALL the tags you want to handle. Use BBCode::basics() to get the list of default bbcode
    //tags. This might be enough for your needs!
    pub fn build(taginfos: Vec<TagInfo>) -> Result<Self, anyhow::Error> {
        //First, get the default direct replacements
        let mut tags = Self::html_escapes().iter().map(|e| {
            //Unfortunately, have to allocate string
            let regstring = format!(r"^{}", e.0);
            Ok(MatchInfo { 
                regex: Regex::new(&regstring)?,
                match_type : MatchType::DirectReplace(e.1)
            })
        }).collect::<Result<Vec<MatchInfo>,anyhow::Error>>()?;

        //This is an optimization: any large block of characters that has no meaning in bbcode can go straight through.
        //Put it FIRST because this is the common case
        tags.insert(0, MatchInfo {
            regex: Regex::new(r#"^[^\[\]<>'"&/]+"#)?,
            match_type : MatchType::Passthrough
        });

        //Next, convert the taginfos to even more "do".
        for tag in taginfos.iter() {
            //The existing system on SBS doesn't allow spaces in tags at ALL. I don't know if this 
            //much leniency on the = value is present in the old system though...
            let open_tag = format!(r#"^\[{}(=[^\]]*)?\]"#, tag.tag);
            tags.push(MatchInfo {
                regex: Regex::new(&open_tag)?,
                match_type : MatchType::Open(tag.clone())
            });
            let close_tag = format!(r#"^\[/{}\]"#, tag.tag);
            tags.push(MatchInfo {
                regex: Regex::new(&close_tag)?,
                match_type : MatchType::Close(tag.clone())
            });
        }

        Ok(BBCode { tags })
    }

    //The basic direct replacement escapes for HTML
    pub fn html_escapes() -> Vec<(&'static str, &'static str)> {
        vec![
            ("<", "&lt;"),
            (">", "&gt;"),
            ("&", "&amp;"),
            ("\"", "&quot;"),
            ("'", "&#39;")
        ]
    }

    //Get a vector of the basic taginfos of bbcode
    pub fn basics() -> Vec<TagInfo> {
        vec![
            TagInfo::simple("b"),
            TagInfo::simple("i"),
            TagInfo::simple("sup"),
            TagInfo::simple("sub"),
            TagInfo::simple("u"),
            TagInfo::simple("s"),
            //There's a [list=1] thing, wonder how to do that. It's nonstandard, our list format is entirely nonstandard
            TagInfo { tag: "list", outtag: "ul", tag_type: TagType::Simple, rawextra: None },
            TagInfo { tag: r"\*", outtag: "li", tag_type: TagType::Simple, rawextra: None },
            TagInfo { tag: "url", outtag: "a", tag_type: TagType::DefaultArg("href"), rawextra: Some(r#"target="_blank""#) },
            TagInfo { tag: "img", outtag: "img", tag_type: TagType::SelfClosing("src"), rawextra: None },
        ]
    }

    //Push an argument="value" part onto the result. Will omit the last " if argval is None
    fn push_tagarg(mut result: String, argname: &str, argval: Option<&str>) -> String {
        result.push_str(" ");
        result.push_str(argname);
        result.push_str("=\"");
        //Now we need an html escaper
        if let Some(argval) = argval {
            //NOTE: our matcher grabs the = (for now), that's why the 1
            result.push_str(&html_escape::encode_quoted_attribute(&argval[1..]));
            result.push_str("\"");
        }
        result
    }

    //Write the "open" tag to the given result for the given new scope. 
    fn push_open_tag(mut result: String, scope: &BBScope, captures: &Captures) -> String {
        result.push_str("<");
        result.push_str(scope.info.outtag);
        //Put the raw stuff first (maybe class, other)
        if let Some(rawextra) = scope.info.rawextra {
            result.push_str(" ");
            result.push_str(rawextra);
        }
        //Now output different stuff depending on the type
        match scope.info.tag_type {
            TagType::Start => {}, //Do nothing
            TagType::Simple => { result.push_str(">"); }, //Just close it, all done!
            TagType::DefinedArg(argname) => {
                if let Some(capture) = captures.get(1) { //Push the argument first
                    result = Self::push_tagarg(result, argname, Some(capture.as_str()));
                }
                result.push_str(">"); //THEN close it!
            },
            //For the opening tag, 'DefaultArg' and 'SelfClosing' act the same. They could either have the value
            //in the arg, or in the body. Difference is on completion, where self closing just closes (or quits),
            //but DefaultArg may have to copy the value into the body, since we only scanned the arg
            TagType::SelfClosing(argname) | TagType::DefaultArg(argname) => {
                if let Some(capture) = captures.get(1) { //If an argument exists, push it
                    result = Self::push_tagarg(result, argname, Some(capture.as_str()));
                    result.push_str(">"); //THEN close it!
                }
                else {  
                    //But if it doesn't, output like it's a SelfClosing, meaning the inner value
                    //in bbcode becomes the 'default arg'. This requires a special handler in the
                    //closing tag
                    result = Self::push_tagarg(result, argname, None);
                }
            },
        }

        result
    }

    fn push_just_close_tag(mut result: String, info: &TagInfo) -> String {
        result.push_str("</");
        result.push_str(info.outtag);
        result.push_str(">");
        result
    }

    //Emit the closing tag for the given scope. This also needs the full input string and the position
    //of the end of this scope, because certain complicated closing tags need it.
    fn push_close_tag(mut result: String, scope: &BBScope, input: &str, end: usize) -> String {
        match scope.info.tag_type {
            TagType::SelfClosing(_) => {
                if !scope.has_arg { 
                    //If this was the standard style of selfclosing, need output the end of the tag.
                    //If it was in the arguments (nonstandard), we already closed it
                    result.push_str(r#"">"#);
                }
            },
            TagType::DefaultArg(_) => {
                //This one is complicated. If there were arguments, we simply output the closing 
                //tag, same as a normal tag. But if there was NOT an argument, we're still in the original
                //tag, AND we still have to output the value we captured in this scope
                if scope.has_arg {
                    result = Self::push_just_close_tag(result, scope.info);
                }
                else {
                    result.push_str(r#"">"#);
                    result.push_str(&input[scope.inner_start..end]);
                    result = Self::push_just_close_tag(result, scope.info);
                }
            }
            TagType::Start => {
                //Do nothing
            },
            _ => {
                result = Self::push_just_close_tag(result, scope.info);
            }
        }

        result
    }

    //Main function! You call this to parse your raw bbcode! It also escapes html stuff so it can
    //be used raw!  Current version keeps newlines as-is and it's expected you use pre-wrap, later
    //there may be modes for more standard implementations
    pub fn parse(&self, input: &str) -> String 
    {
        //We know it will be at LEAST as big, and that strings usually double in size
        //when they grow anyway, so just start at 2X by default
        let mut result = String::with_capacity(input.len() * 2);

        //Because of utf-8, it's better to just use regex directly all the time?
        let mut slice = &input[0..]; //Not necessary to be this explicit ofc

        //Only 'Taginfo' can create scope, so don't worry about "DirectReplace" types
        let mut scoper = BBScoper::new();
        let start_info = TagInfo::start();
        scoper.add_scope(BBScope { info: &start_info, inner_start: 0, has_arg: false });

        //To determine how far into the string we are
        let input_ptr = input.as_ptr();

        //While there is string left, keep checking against all the regex. Remove some regex
        //if the current scope is a meanie
        while slice.len() > 0
        {
            //Slow? IDK, fix it later
            let current_scope = match scoper.scopes.last() {
                Some(cs) => cs,
                None => {
                    println!("BBCode::parse, ran out of scopes somehow!");
                    break;
                }
            };

            let mut matched_do : Option<&MatchInfo> = None;

            //figure out which next element matches (if any). This is the terrible optimization part, but
            //these are such small state machines with nothing too crazy that I think it's fine.... maybe.
            //Especially since they all start at the start of the string
            for tagdo in &self.tags {
                if current_scope.info.tag_type.is_verbatim() {
                    match &tagdo.match_type {
                        //If the thing to match is open or close and we're inside a verbatim string, skip the matching if
                        //it's not the same tag as the current scope. So, [url]what[url] is fine, [url] will be found
                        MatchType::Open(doinfo) | MatchType::Close(doinfo) => { if doinfo.tag != current_scope.info.tag { continue; } }
                        //MatchType::Close(doinfo) => { if doinfo.tag != current_scope.info.tag { continue; } }
                        _ => {} //Do nothing
                    }
                }
                if tagdo.regex.is_match(slice) {
                    matched_do = Some(tagdo);
                    break;
                }
            }

            //SOMETHING matched, which means we do something special to consume the output
            if let Some(tagdo) = matched_do 
            {
                //There should only be one but whatever
                for captures in tagdo.regex.captures_iter(slice) {
                    //do this pre-emptively so we're AT the start of the inside of the tag
                    let scope_end : usize = slice.as_ptr() as usize - input.as_ptr() as usize;
                    slice = &slice[captures[0].len()..];
                    match &tagdo.match_type {
                        MatchType::Passthrough => {
                            //The entire matched portion can go straight through. This gets us quickly
                            //through chunks of non-bbcode 
                            result.push_str(&captures[0]);
                        },
                        MatchType::DirectReplace(new_text) => {
                            //The matched chunk has a simple replacement with no rules
                            result.push_str(new_text);
                        },
                        MatchType::Open(info) => {
                            //Need to enter a scope. Remember where the beginning of this scope is just in case we need it
                            let new_scope = BBScope {
                                info, 
                                inner_start : (slice.as_ptr() as usize) - (input_ptr as usize),
                                has_arg: captures.get(1).is_some()
                            };
                            //By starting a scope, we may close many scopes. Also it'll tell us what it thinks the 
                            //starting scope looks like (it may change? probably not though)
                            let scope_result = scoper.add_scope(new_scope);
                            for cscope in scope_result.1 {
                                result = Self::push_close_tag(result, &cscope, input, scope_end);
                            }
                            //The add_scope function only gives us the close scopes, so we
                            //still need to emit the open tag
                            result = Self::push_open_tag(result, scope_result.0, &captures);
                        },
                        MatchType::Close(info) => {
                            //Attempt to close the given scope. The scoper will return all the actual scopes
                            //that were closed, which we can dump
                            for cscope in scoper.close_scope(info) {
                                result = Self::push_close_tag(result, &cscope, input, scope_end);
                            }
                            //The close_scope function gives us the scopes to close
                        }
                    }
                }
            }
            else  //Nothing matched, so we just consume the next character. This should be very rare
            {
                //just move forward and emit the char. Note that the slice is in bytes, but the char
                //is a unicode scalar that could be up to 4 bytes, so we need to know how many 'bytes'
                //we just popped off
                if let Some(ch) = slice.chars().next() {
                    result.push(ch);
                    slice = &slice[ch.len_utf8()..];
                }
                else {
                    println!("In BBCode::parse, there were no more characters but there were leftover bytes!");
                    break;
                }
            }
        }

        //At the end, we should close any unclosed scopes
        for cscope in scoper.dump_remaining() {
            result = Self::push_close_tag(result, &cscope, input, input.len() as usize);
        }

        result
    }
}



// ----------------------------
// *         TESTS           
// ----------------------------

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! bbtest_basics {
        ($($name:ident: $value:expr;)*) => {
        $(
            #[test]
            fn $name() {
                let bbcode = BBCode::build(BBCode::basics()).unwrap();
                let (input, expected) = $value;
                assert_eq!(bbcode.parse(input), expected);
            }
        )*
        }
    }

    #[test]
    fn build_init() {
        //This shouldn't fail?
        let _bbcode = BBCode::build(BBCode::basics()).unwrap();
    }

    #[test]
    fn build_add_lt() {
        //This shouldn't fail?
        let bbcode = BBCode::build(BBCode::basics()).unwrap();
        let found = bbcode.tags.iter().find(|x| matches!(x.match_type, MatchType::DirectReplace(_))).unwrap();
        assert_eq!(found.regex.as_str(), "^<");
        if let MatchType::DirectReplace(repl) = found.match_type {
            assert_eq!(repl, "&lt;")
        }
        else {
            panic!("TEST LOGIC ERROR, NOT DIRECTREPLACE TYPE");
        }
    }

    bbtest_basics! {
        no_alter: ("hello", "hello");
        lt_single: ("h<ello", "h&lt;ello");
        gt_single: ("h>ello", "h&gt;ello");
        amp_single: ("h&ello", "h&amp;ello");
        quote_single: ("h'ello", "h&#39;ello");
        doublequote_single: ("h\"ello", "h&quot;ello");
        complex_escape: (
            "it's a %CRAZY% <world> 💙=\"yeah\" 👨‍👨‍👧‍👦>>done", 
            "it&#39;s a %CRAZY% &lt;world&gt; 💙=&quot;yeah&quot; 👨‍👨‍👧‍👦&gt;&gt;done"
        );
        //"Simple" means there are no complicated tag structures, or only a single tag (most common)
        simple_bold: ("[b]hello[/b]", "<b>hello</b>");
        simple_sup: ("[sup]hello[/sup]", "<sup>hello</sup>");
        simple_sub: ("[sub]hello[/sub]", "<sub>hello</sub>");
        simple_strikethrough: ("[s]hello[/s]", "<s>hello</s>");
        simple_underline: ("[u]hello[/u]", "<u>hello</u>");
        simple_italic: ("[i]hello[/i]", "<i>hello</i>");
        simple_nospaces: ("[b ]hello[/ b]", "[b ]hello[/ b]");
        simple_bolditalic: ("[b][i]hello[/i][/b]", "<b><i>hello</i></b>");
        simple_url_default: ("[url]https://google.com[/url]", r#"<a target="_blank" href="https://google.com">https://google.com</a>"#);
        simple_url_witharg: ("[url=http://haloopdy.com]furries lol[/url]", r#"<a target="_blank" href="http://haloopdy.com">furries lol</a>"#);
        simple_img: ("[img]https://old.smilebasicsource.com/user_uploads/avatars/t1647374379.png[/img]", r#"<img src="https://old.smilebasicsource.com/user_uploads/avatars/t1647374379.png">"#);
        simple_img_nonstd: ("[img=https://old.smilebasicsource.com/user_uploads/avatars/t1647374379.png][/img]", r#"<img src="https://old.smilebasicsource.com/user_uploads/avatars/t1647374379.png">"#);
        //NOTE: this one, it's just how I want it to work. IDK how the real bbcode handles this weirdness
        simple_img_nonstd_inner: ("[img=https://old.smilebasicsource.com/user_uploads/avatars/t1647374379.png]abc 123[/img]", r#"<img src="https://old.smilebasicsource.com/user_uploads/avatars/t1647374379.png">abc 123"#);
        //This also tests auto-closed tags, albeit a simple form
        list_basic:  ("[list]\n[*]item 1\n[*]item 2\n[*]list\n[/list]", "<ul>\n<li>item 1\n</li><li>item 2\n</li><li>list\n</li></ul>");
        unclosed_basic: ("[b] this is bold [i]also italic[/b] oops close all[/i]", "<b> this is bold <i>also italic</i></b> oops close all");
        verbatim_url: ("[url]this[b]is[/b]a no-no[i][/url]", r#"<a target="_blank" href="this[b]is[/b]a no-no[i]">this[b]is[/b]a no-no[i]</a>"#);
        inner_hack: ("[[b][/b]b]love[/[b][/b]b]", "[<b></b>b]love[/<b></b>b]");
        random_brackets: ("[][[][6][a[ab]c[i]italic[but][][* not] 8[]]][", "[][[][6][a[ab]c<i>italic[but][][* not] 8[]]][</i>");
    }
}