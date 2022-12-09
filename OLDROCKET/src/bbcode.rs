use regex::{Regex, Captures};

// Carlos Sanchez - 2022-12-05
// - For SBS

//So:
//- early closures close all tags in the previous scope
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
    pub valparse: TagValueParse,
    pub blankconsume : BlankConsume
}

impl TagInfo {
    //Constructors for basic tags. Anything else, you're better off just constructing it normally
    pub fn simple(tag: &'static str) -> TagInfo {
        TagInfo { tag, outtag: tag, tag_type: TagType::Simple, rawextra: None, valparse: TagValueParse::Normal, blankconsume: BlankConsume::None }
    }
    /*pub fn normal(tag: &'static str, outtag: &'static str, tag_type: TagType, rawextra: Option<&'static str>) -> TagInfo {
        TagInfo { tag, outtag, tag_type, rawextra, valparse
    }*/
    fn start() -> TagInfo {
        TagInfo { tag: "", outtag: "", tag_type: TagType::Start, rawextra: None, valparse: TagValueParse::Normal, blankconsume: BlankConsume::None }
    }

    pub fn is_verbatim(&self) -> bool {
        if let TagValueParse::ForceVerbatim = self.valparse {
            true
        }
        else {
            match self.tag_type {
                TagType::Start => false,
                TagType::Simple => false,
                TagType::DefinedArg(_) => false,
                TagType::DefinedTag(_, _) => false,
                TagType::SelfClosing(_) => true,
                TagType::DefaultArg(_) => true
            }
        }
    }
}

//Property of value parsing
#[derive(Debug, Clone)]
pub enum TagValueParse {
    Normal,
    ForceVerbatim,
    DoubleCloses
}

//This is the 'silly' part of the parser. Rather than making some actually generic system, I identified some
//standard ways tags are used and just made code around those ways. Probably bad but oh well.
#[derive(Debug, Clone)]
pub enum TagType {
    Start,          //Should ONLY have one of these! It's like S in a grammar!
    Simple,         //Stuff like [b][/b], no args, normal translation (can change tag name still)
    DefinedArg(&'static str),   //CAN have argument defined, attribute name is given
    DefinedTag(&'static str, Option<&'static str>),   //Some arguments turn into tags! Crazy...
    SelfClosing(&'static str),  //No closing tag, value is 
    DefaultArg(&'static str),   //The tag enclosed value provides a default for the given attribute, or not if defined
}

#[derive(Debug, Clone)]
pub enum BlankConsume {
    None,
    Start(i32),
    End(i32)
}

//While "TagType" determines how the tag functions at a lower level (such as how it handles arguments), 
//this determines how the whole block functions on a greater level. They define how scopes and whole blocks
//of text move into the output
#[derive(Debug, Clone)]
pub enum MatchType { 
    Passthrough,    //Pass this junk right out as-is
    Open(TagInfo),  //this is so small, it's fine to dupe in open/close
    Close(TagInfo),
    //Note: you can use BlockTransform to craft many kinds of generic matching, if it can use regex! It just won't
    //be part of the scoping rules! IE it should be an entire block! Also, the match will ALWAYS be html escaped first!
    BlockTransform(&'static str),  //Take the full match and transform it into something entirely different
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
            //oh the thing on top is the same, if we don't want that, close it.
            if topinfo.info.tag == scope.info.tag && matches!(scope.info.valparse, TagValueParse::DoubleCloses){
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
    //Supply this!
    pub matchers: Vec<MatchInfo> //These are SOMETIMES processed (based on context)
}

impl BBCode 
{
    //Get a default bbcode parser. Should hopefully have reasonable defaults!
    #[allow(dead_code)]
    pub fn default() -> Result<Self, anyhow::Error> {
        Ok(Self { matchers: Self::basics()? })
    }

    //The basic direct replacement escapes for HTML. You don't need these if you're using 'basics()'
    pub fn html_escapes() -> Vec<(&'static str, &'static str)> {
        vec![
            ("<", "&lt;"),
            (">", "&gt;"),
            ("&", "&amp;"),
            ("\"", "&quot;"),
            ("'", "&#39;"),
            ("\r", "") //why are these even here??
        ]
    }

    //Get a vector of the basic taginfos of bbcode. You don't need this if you're using 'basics()'
    pub fn basic_tags() -> Vec<TagInfo> {
        vec![
            TagInfo::simple("b"),
            TagInfo::simple("i"),
            TagInfo::simple("sup"),
            TagInfo::simple("sub"),
            TagInfo::simple("u"),
            TagInfo::simple("s"),
            //There's a [list=1] thing, wonder how to do that. It's nonstandard, our list format is entirely nonstandard
            TagInfo { tag: "list", outtag: "ul", tag_type: TagType::Simple, rawextra: None, valparse: TagValueParse::Normal, blankconsume: BlankConsume::End(1)},
            TagInfo { tag: r"\*", outtag: "li", tag_type: TagType::Simple, rawextra: None, valparse: TagValueParse::DoubleCloses, blankconsume: BlankConsume::Start(1)},
            TagInfo { tag: "url", outtag: "a", tag_type: TagType::DefaultArg("href"), rawextra: Some(r#"target="_blank""#), valparse: TagValueParse::ForceVerbatim, blankconsume: BlankConsume::None },
            TagInfo { tag: "img", outtag: "img", tag_type: TagType::SelfClosing("src"), rawextra: None, valparse: TagValueParse::ForceVerbatim, blankconsume: BlankConsume::None } //Not required to be forced
        ]
    }
    
    //If you have extra tags you want to add, use this function to turn the basic definitions into
    //a vector of real MatchInfo for use in the bbcode system
    pub fn tags_to_matches(taginfos: Vec<TagInfo>) -> Result<Vec<MatchInfo>, anyhow::Error> {
        let mut matches = Vec::new();
        //Next, convert the taginfos to even more "do".
        for tag in taginfos.iter() {
            let mut openchomp = String::from("");
            let mut closechomp = String::from("");
            match tag.blankconsume {
                BlankConsume::None => {}, //do nothing
                BlankConsume::Start(amount) => { openchomp = format!("(\r?\n){{0,{}}}", amount); }
                BlankConsume::End(amount) => { closechomp = format!("(\r?\n){{0,{}}}", amount); }
            }
            //The existing system on SBS doesn't allow spaces in tags at ALL. I don't know if this 
            //much leniency on the = value is present in the old system though...
            let open_tag = format!(r#"^(?i){}\[{}(=[^\]]*)?\]{}"#, openchomp, tag.tag, closechomp);
            matches.push(MatchInfo {
                regex: Regex::new(&open_tag)?,
                match_type : MatchType::Open(tag.clone())
            });
            let close_tag = format!(r#"^(?i){}\[/{}\]{}"#, openchomp, tag.tag, closechomp);
            matches.push(MatchInfo {
                regex: Regex::new(&close_tag)?,
                match_type : MatchType::Close(tag.clone())
            });
        }
        Ok(matches)
    }


    //Get a vector of ALL basic matchers! This is the function you want to call to get a vector for the bbcode
    //generator!
    pub fn basics() -> Result<Vec<MatchInfo>, anyhow::Error> {
        //First, get the default direct replacements
        let mut matches = Self::html_escapes().iter().map(|e| {
            //Unfortunately, have to allocate string
            let regstring = format!(r"^{}", e.0);
            Ok(MatchInfo { 
                regex: Regex::new(&regstring)?,
                match_type : MatchType::DirectReplace(e.1)
            })
        }).collect::<Result<Vec<MatchInfo>,anyhow::Error>>()?;

        //This is an optimization: any large block of characters that has no meaning in bbcode can go straight through.
        //Put it FIRST because this is the common case
        matches.insert(0, MatchInfo {
            //We use h to catch ourselves on https. this unfortunately breaks up large sections of text into much
            //smaller ones, but it should be ok... I don't know. My parser is stupid lol
            regex: Regex::new(r#"^[^\[\]<>'"&\r\n/h]+"#)?, 
            match_type : MatchType::Passthrough
        });

        //Don't forget about autolinking! This is a crappy autolinker and it doesn't matter too much!
        matches.push(MatchInfo { 
            //characters taken from google's page https://developers.google.com/maps/url-encoding
            regex: Regex::new(r#"^(https?://[a-zA-Z0-9\-_.~!*()';:@&=+$,/?%#\[\]]+)"#)?, 
            match_type: MatchType::BlockTransform(r#"<a target="_blank" href="$0">$0</a>"#) 
        });

        let mut tag_matches = Self::tags_to_matches(Self::basic_tags())?;
        matches.append(&mut tag_matches);

        Ok(matches)
    }

    //Some fancy extra bbcode. You have to append it yourself to basic! These are nonstandard, you don't have to use them!
    pub fn extras() -> Result<Vec<MatchInfo>, anyhow::Error> {
        BBCode::tags_to_matches(vec![
            TagInfo { tag: "quote", outtag: "blockquote", tag_type : TagType::DefinedArg("cite"), rawextra : None, valparse: TagValueParse::Normal, blankconsume: BlankConsume::End(1) },
            TagInfo { tag: "anchor", outtag: "a", tag_type : TagType::DefinedArg("name"), rawextra : None, valparse: TagValueParse::ForceVerbatim, blankconsume: BlankConsume::None },
            TagInfo { tag: "icode", outtag: "span", tag_type : TagType::Simple, rawextra : Some(r#"class="icode""#), valparse: TagValueParse::ForceVerbatim, blankconsume: BlankConsume::None },
            TagInfo { tag: "code", outtag: "pre", tag_type : TagType::DefinedArg("data-code"), rawextra : Some(r#"class="code""#), valparse: TagValueParse::ForceVerbatim, blankconsume: BlankConsume::End(1) },
            TagInfo { tag: "youtube", outtag: "a", tag_type : TagType::DefaultArg("href"), rawextra : Some(r#"class="youtube" data-youtube"#), valparse: TagValueParse::ForceVerbatim, blankconsume: BlankConsume::End(1) },
            TagInfo { tag: "spoiler", outtag: "details", tag_type : TagType::DefinedTag("summary", Some("Spoiler")), rawextra : Some(r#"class="spoiler""#), valparse: TagValueParse::Normal, blankconsume: BlankConsume::None },
            TagInfo::simple("h1"),
            TagInfo::simple("h2"),
            TagInfo::simple("h3"),
        ])
    }

    //Push an argument="value" part onto the result. Will omit the last " if argval is None
    fn push_tagarg(mut result: String, argname: &str, argval: Option<&str>) -> String {
        result.push_str(" ");
        result.push_str(argname);
        result.push_str("=\"");
        //Now we need an html escaper
        if let Some(argval) = argval {
            //NOTE: our matcher grabs the = (for now), that's why the 1
            result.push_str(&html_escape::encode_quoted_attribute(argval));
            result.push_str("\"");
        }
        result
    }

    fn push_newtag(mut result: String, tagname: &str, argval: &str) -> String {
        //Close the old tag, open a new one
        result.push_str("><");
        result.push_str(tagname);
        result.push_str(">");
        //NOTE: our matcher grabs the = (for now), that's why the 1
        result.push_str(&html_escape::encode_quoted_attribute(argval));
        //And close the whole thing off
        result.push_str("</");
        result.push_str(tagname);
        result.push_str(">");
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
                    result = Self::push_tagarg(result, argname, Some(&capture.as_str()[1..]));
                }
                result.push_str(">"); //THEN close it!
            },
            TagType::DefinedTag(tagname, default) => { //These make the argument a new tag 
                if let Some(capture) = captures.get(1) { //Push the argument first
                    result = Self::push_newtag(result, tagname, &capture.as_str()[1..]); //+1 here because it's not some?
                }
                else if let Some(default) = default {
                    result = Self::push_newtag(result, tagname, default);
                }
                else {
                    result.push_str(">"); //If we didn't push a new arg, gotta close the existing tag
                }
            },
            //For the opening tag, 'DefaultArg' and 'SelfClosing' act the same. They could either have the value
            //in the arg, or in the body. Difference is on completion, where self closing just closes (or quits),
            //but DefaultArg may have to copy the value into the body, since we only scanned the arg
            TagType::SelfClosing(argname) | TagType::DefaultArg(argname) => {
                if let Some(capture) = captures.get(1) { //If an argument exists, push it
                    result = Self::push_tagarg(result, argname, Some(&capture.as_str()[1..]));
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
            for matchinfo in &self.matchers {
                if current_scope.info.is_verbatim() {
                    match &matchinfo.match_type {
                        //If the thing to match is open or close and we're inside a verbatim string, skip the matching if
                        //it's not the same tag as the current scope. So, [url]what[url] is fine, [url] will be found
                        MatchType::Open(doinfo) | MatchType::Close(doinfo) => { if doinfo.tag != current_scope.info.tag { continue; } }
                        MatchType::BlockTransform(_) => continue, //No block transforms inside verbatim no matter what!
                        //MatchType::Close(doinfo) => { if doinfo.tag != current_scope.info.tag { continue; } }
                        _ => {} //Do nothing
                    }
                }
                if matchinfo.regex.is_match(slice) {
                    matched_do = Some(matchinfo);
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
                        MatchType::BlockTransform(replacement) => {
                            //need to take the captures and transform it to whatever you wanted. But always be safe! If you don't
                            //want this, hmmmm gotta think about that
                            //result.push_str(&html_escape::encode_quoted_attribute(&argval[1..]));
                            result.push_str(&tagdo.regex.replace(&html_escape::encode_quoted_attribute(&captures[0]), *replacement));
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
                let bbcode = BBCode { matchers: BBCode::basics().unwrap() };
                let (input, expected) = $value;
                assert_eq!(bbcode.parse(input), expected);
            }
        )*
        }
    }

    macro_rules! bbtest_extras {
        ($($name:ident: $value:expr;)*) => {
        $(
            #[test]
            fn $name() {
                let mut matchers = BBCode::basics().unwrap();
                let mut extras = BBCode::extras().unwrap();
                matchers.append(&mut extras);
                let bbcode = BBCode { matchers };
                let (input, expected) = $value;
                assert_eq!(bbcode.parse(input), expected);
            }
        )*
        }
    }

    #[test]
    fn build_init() {
        //This shouldn't fail?
        let _bbcode = BBCode { matchers: BBCode::basics().unwrap() };
    }

    #[test]
    fn build_add_lt() {
        //This shouldn't fail?
        let bbcode = BBCode { matchers: BBCode::basics().unwrap() };
        let found = bbcode.matchers.iter().find(|x| matches!(x.match_type, MatchType::DirectReplace(_))).unwrap();
        assert_eq!(found.regex.as_str(), "^<");
        if let MatchType::DirectReplace(repl) = found.match_type {
            assert_eq!(repl, "&lt;")
        }
        else {
            panic!("TEST LOGIC ERROR, NOT DIRECTREPLACE TYPE");
        }
    }

    //#[test] //Not really a unit test but whatever
    //fn benchmark_10000() {
    //    let mut matchers = BBCode::basics().unwrap();
    //    let mut extras = BBCode::extras().unwrap();
    //    matchers.append(&mut extras);
    //    let bbcode = BBCode { matchers };
    //    let parselem = vec![
    //        ("it's a %CRAZY% <world> 💙=\"yeah\" 👨‍👨‍👧‍👦>>done", 
    //         "it&#39;s a %CRAZY% &lt;world&gt; 💙=&quot;yeah&quot; 👨‍👨‍👧‍👦&gt;&gt;done"),
    //        ("[][[][6][a[ab]c[i]italic[but][][* not] 8[]]][", "[][[][6][a[ab]c<i>italic[but][][* not] 8[]]][</i>"),
    //        ("[url]this[b]is[/b]a no-no[i][/url]", r#"<a target="_blank" href="this[b]is[/b]a no-no[i]">this[b]is[/b]a no-no[i]</a>"#),
    //        ("[img=https://old.smilebasicsource.com/user_uploads/avatars/t1647374379.png]abc 123[/img]", r#"<img src="https://old.smilebasicsource.com/user_uploads/avatars/t1647374379.png">abc 123"#),
    //        ("[spoiler]this[b]is empty[/spoiler]", r#"<details class="spoiler"><summary>Spoiler</summary>this<b>is empty</b></details>"#)
    //    ];

    //    for i in 0..10000 {
    //        if let Some((input, output)) = parselem.get(i % parselem.len()) {
    //            if bbcode.parse(*input) != *output {
    //                panic!("Hang on, bbcode isn't working!");
    //            }
    //        }
    //        else {
    //            panic!("WHAT? INDEX OUT OF BOUNDS??");
    //        }
    //    }
    //}

    bbtest_basics! {
        no_alter: ("hello", "hello");
        lt_single: ("h<ello", "h&lt;ello");
        gt_single: ("h>ello", "h&gt;ello");
        amp_single: ("h&ello", "h&amp;ello");
        quote_single: ("h'ello", "h&#39;ello");
        doublequote_single: ("h\"ello", "h&quot;ello");
        return_byebye: ("h\rello", "hello");
        //Because inserting tags without knowing the scope is bad (in our system for now), don't generate
        //<br>, just figure the whitespace is pre-wrap or something
        newline_br: ("h\nello", "h\nello");
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
        //The matches are returned lowercase from regex when insensitive
        simple_insensitive: ("[sUp]hello[/SuP]", "<sup>hello</sup>");
        simple_sensitivevalue: ("[sUp]OK but The CAPITALS[/SuP]YEA", "<sup>OK but The CAPITALS</sup>YEA");
        simple_bolditalic: ("[b][i]hello[/i][/b]", "<b><i>hello</i></b>");
        nested_bold: ("[b]hey[b]extra bold[/b] less bold again[/b]", "<b>hey<b>extra bold</b> less bold again</b>");
        simple_url_default: ("[url]https://google.com[/url]", r#"<a target="_blank" href="https://google.com">https://google.com</a>"#);
        simple_url_witharg: ("[url=http://haloopdy.com]furries lol[/url]", r#"<a target="_blank" href="http://haloopdy.com">furries lol</a>"#);
        simple_img: ("[img]https://old.smilebasicsource.com/user_uploads/avatars/t1647374379.png[/img]", r#"<img src="https://old.smilebasicsource.com/user_uploads/avatars/t1647374379.png">"#);
        simple_img_nonstd: ("[img=https://old.smilebasicsource.com/user_uploads/avatars/t1647374379.png][/img]", r#"<img src="https://old.smilebasicsource.com/user_uploads/avatars/t1647374379.png">"#);
        //NOTE: this one, it's just how I want it to work. IDK how the real bbcode handles this weirdness
        simple_img_nonstd_inner: ("[img=https://old.smilebasicsource.com/user_uploads/avatars/t1647374379.png]abc 123[/img]", r#"<img src="https://old.smilebasicsource.com/user_uploads/avatars/t1647374379.png">abc 123"#);
        //This also tests auto-closed tags, albeit a simple form
        list_basic:  ("[list][*]item 1[/*][*]item 2[/*][*]list[/*][/list]", "<ul><li>item 1</li><li>item 2</li><li>list</li></ul>");
        unclosed_basic: ("[b] this is bold [i]also italic[/b] oops close all[/i]", "<b> this is bold <i>also italic</i></b> oops close all");
        verbatim_url: ("[url]this[b]is[/b]a no-no[i][/url]", r#"<a target="_blank" href="this[b]is[/b]a no-no[i]">this[b]is[/b]a no-no[i]</a>"#);
        inner_hack: ("[[b][/b]b]love[/[b][/b]b]", "[<b></b>b]love[/<b></b>b]");
        random_brackets: ("[][[][6][a[ab]c[i]italic[but][][* not] 8[]]][", "[][[][6][a[ab]c<i>italic[but][][* not] 8[]]][</i>");
        autolink_basic: ("this is https://google.com ok?", r#"this is <a target="_blank" href="https://google.com">https://google.com</a> ok?"#);

        newline_list1: ("[list]\n[*]item", "<ul><li>item</li></ul>");
        newline_list2: ("[list]\r\n[*]item", "<ul><li>item</li></ul>");
        newline_listmega: ("\n[list]\r\n[*]item\r\n[*]item2 yeah[\r\n\r\n[*]three", "\n<ul><li>item</li><li>item2 yeah[\n</li><li>three</li></ul>");
        //Bold, italic, etc should not remove newlines anywhere
        newline_bold: ("\n[b]\nhellow\n[/b]\n", "\n<b>\nhellow\n</b>\n");
        newline_italic: ("\n[i]\nhellow\n[/i]\n", "\n<i>\nhellow\n</i>\n");
        newline_underline: ("\n[u]\nhellow\n[/u]\n", "\n<u>\nhellow\n</u>\n");
        newline_strikethrough: ("\n[s]\nhellow\n[/s]\n", "\n<s>\nhellow\n</s>\n");
        newline_sup: ("\n[sup]\nhellow\n[/sup]\n", "\n<sup>\nhellow\n</sup>\n");
        newline_sub: ("\n[sub]\nhellow\n[/sub]\n", "\n<sub>\nhellow\n</sub>\n");

        //Nicole's bbcode edge cases
        e_dangling: ("[b]foo", "<b>foo</b>");
        e_normal: ("[b]foo[/b]", "<b>foo</b>");
        e_nested: ("[b]foo[b]bar[/b][/b]", "<b>foo<b>bar</b></b>");
        e_empty: ("[b]foo[b][/b]bar[/b]", "<b>foo<b></b>bar</b>");
        e_closemulti: ("[b]foo[i]bar[u]baz[/b]quux", "<b>foo<i>bar<u>baz</u></i></b>quux");
        e_faketag: ("[b]foo[i]bar[u]baz[/fake]quux", "<b>foo<i>bar<u>baz[/fake]quux</u></i></b>");
        e_reallyfake: ("[fake][b]foo[i]bar[u]baz[/fake]quux", "[fake]<b>foo<i>bar<u>baz[/fake]quux</u></i></b>");
        e_ignoreclose: ("[b]foo[/b]bar[/b][/b][/b]", "<b>foo</b>bar");
        e_weirdignoreclose: ("[b]foo[/b]bar[/fake][/b][/fake]", "<b>foo</b>bar[/fake][/fake]");
        e_fancytag: ("[[i]b[/i]]", "[<i>b</i>]");
        e_escapemadness: ("&[&]<[<]>[>]", "&amp;[&amp;]&lt;[&lt;]&gt;[&gt;]");
    }

    bbtest_extras! {
        e_emptyquote: ("[quote]...[/quote]", "<blockquote>...</blockquote>");
        e_normalquote: ("[quote=foo]...[/quote]", r#"<blockquote cite="foo">...</blockquote>"#);
        simple_spoiler: ("[spoiler=wow]amazing[/spoiler]", r#"<details class="spoiler"><summary>wow</summary>amazing</details>"#);
        simple_emptyspoiler: ("[spoiler]this[b]is empty[/spoiler]", r#"<details class="spoiler"><summary>Spoiler</summary>this<b>is empty</b></details>"#);
    }

/* These tests are limitations of the old parser, I don't want to include them
[quote=foo=bar]...[/quote]
<blockquote>...</blockquote>

[quote=[foo]]...[/quote]
[quote=[foo]]...
*/

}