use regex::Regex;

//So:
//- dupes auto close previous scope.
//- ignore unmatched closing tags

//The user provides this
#[derive(Debug, Clone)]
pub struct TagInfo {
    //The tag identity, such as "b", "youtube", etc
    pub tag : &'static str,
    //The tag to put out as html
    pub outtag: &'static str,
    pub tag_type: TagType,
    //pub argout : Option<&'static str>,
    //pub valout: Option<&'static str>, //Put the value in this attribute, if defined
    //pub verbatim : bool, //whether tags are allowed inside, basically
    pub rawextra: Option<&'static str>, //Just dump this directly into the tag at the end. No checks performed
}

impl TagInfo {
    fn simple(tag: &'static str) -> TagInfo {
        TagInfo { tag, outtag: tag, tag_type: TagType::Simple, rawextra: None }
        //argout: None, valout: None, verbatim: false, rawextra: None }
    }
}

#[derive(Debug, Clone)]
enum TagType {
    Simple,
    DefinedArg(&'static str),   //CAN have argument defined, attribute name is given
    SelfClosing(&'static str),  //No closing tag, value is 
    DefaultArg(&'static str),   //The tag enclosed value provides a default for the given attribute, or not if defined
}

impl TagType {
    fn is_verbatim(&self) -> bool {
        match self {
            Self::Simple => false,
            Self::DefinedArg(_) => false,
            Self::SelfClosing(_) => true,
            Self::DefaultArg(_) => true
        }
    }
}

//Bbcode tags come in... different flavors maybe (or this is bad?)
#[derive(Debug, Clone)]
pub enum MatchType { 
    Open(TagInfo), //this is so small, it's fine to dupe in open/close
    Close(TagInfo),
    DirectReplace(&'static str)
}

//The system uses this 
#[derive(Debug, Clone)]
pub struct TagDo {
    pub regex : Regex,
    pub match_type: MatchType,
    //pub info : TagInfo 
}

struct BBScope<'a> {
    scopes : Vec<&'a TagInfo>
}

impl<'a> BBScope<'a> {
    fn new() -> Self { BBScope { scopes: Vec::new() }}

    //Add a scope, which may close some existing scopes. The closed scopes are returned in display order.
    //NOTE: the added infos must live as long as the scope container!
    fn add_scope(&mut self, info: &'a TagInfo) -> Vec<&TagInfo> {
        //here we assume all taginfos have unique tags because why wouldn't they
        if let Some(topinfo) = self.scopes.last() {
            //Duplicate scopes close the preceding one.
            //Do nothing, but tell the caller that we "closed" the top scope
            vec![*topinfo]
        }
        else {
            self.scopes.push(info);
            Vec::new()
        }
    }

    //Close the given scope, which should return the scopes that got closed (including the self).
    //If no scope could be found, the vector is empty
    fn close_scope(&mut self, info: &'a TagInfo) -> Vec<&TagInfo> {
        let mut scope_count = 0;
        let mut tag_found = false;

        for scope in self.scopes.iter().rev() {
            scope_count += 1;
            if info.tag == scope.tag {
                tag_found = true;
                break;
            }
        }

        if tag_found {
            let mut result = Vec::with_capacity(scope_count + 1);
            for _ in [0..result.len()] {
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
    fn dump_remaining(self) -> Vec<&'a TagInfo> {
        self.scopes.into_iter().rev().collect()
    }
}

pub struct BBCode {
    //These are ALWAYS processed
    //pub directs : Vec<(&'static str, &'static str)>,
    //These are SOMETIMES processed (based on context)
    pub tags : Vec<TagDo>
}

impl BBCode {
    //Maybe get rid of anyhow if you want to separate this, kind of a big thing to include.
    //Anyway, this build function precompiles all the regex for you. Try to reuse this item
    //as much as possible, it doesn't require mutation and so can be part of the global rocket state
    pub fn build(taginfos: Vec<TagInfo>) -> Result<Self, anyhow::Error> {
        //First, get the default direct replacements
        let mut tags = Self::html_escapes().iter().map(|e| {
            //Unfortunately, have to allocate string
            let regstring = format!(r"^{}", e.0);
            Ok(TagDo { 
                regex: Regex::new(&regstring)?,
                match_type : MatchType::DirectReplace(e.1)
            })
        }).collect::<Result<Vec<TagDo>,anyhow::Error>>()?;

        //Next, convert the taginfos to even more "do".
        for tag in taginfos.iter() {
            //The existing system on SBS doesn't allow spaces in tags at ALL. I don't know if this 
            //much leniency on the = value is present in the old system though...
            let open_tag = format!(r#"^\[{}(=[^\]]*)?\]"#, tag.tag);
            tags.push(TagDo {
                regex: Regex::new(&open_tag)?,
                match_type : MatchType::Open(tag.clone())
            });
            let close_tag = format!(r#"^\[/{}\]"#, tag.tag);
            tags.push(TagDo {
                regex: Regex::new(&close_tag)?,
                match_type : MatchType::Close(tag.clone())
            });
        }

        Ok(BBCode { tags })
    }

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
            TagInfo { tag: "url", outtag: "a", tag_type: TagType::DefaultArg("href"), rawextra: Some(r#"target="_blank""#) },
            TagInfo { tag: "img", outtag: "img", tag_type: TagType::SelfClosing("src"), rawextra: None },
        ]
    }

    fn push_tagarg(mut result: String, argname: &str, argval: Option<&str>) -> String {
        result.push_str(" ");
        result.push_str(argname);
        result.push_str("=\"");
        //Now we need an html escaper
        if let Some(argval) = argval {
            result.push_str(&html_escape::encode_quoted_attribute(argval));
            result.push_str("\"");
        }
        result
    }

    fn push_open_tag(mut result: String, info: &TagInfo, capture: &regex::Captures) -> String {
        result.push_str("<");
        result.push_str(info.outtag);
        result.push_str(" ");
        //Put the raw stuff first (maybe class, other)
        if let Some(rawextra) = info.rawextra {
            result.push_str(rawextra);
            result.push_str(" ");
        }
        //Now output different stuff depending on the type
        match info.tag_type {
            TagType::Simple => {
                result.push_str(">"); //Just close it, all done!
            },
            TagType::DefinedArg(argname) => {
                if capture.len() > 1 { //Push the argument first
                    result = Self::push_tagarg(result, argname, Some(&capture[1]));
                }
                result.push_str(">"); //THEN close it!
            },
            TagType::DefaultArg(argname) => {
                if capture.len() > 1 { //If an argument exists, push it
                    result = Self::push_tagarg(result, argname, Some(&capture[1]));
                }
                else { //But if it doesn't, use the... value... wait a sec... how will this work

                }
                result.push_str(">"); //THEN close it!
            }
        }

        //You can have both an argout AND a valout. Your implementation of valout might be dangerous!
        if let Some(argout) = info.argout {
            if capture.len() > 1 {
                result = Self::push_tagarg(result, argout, Some(&capture[1]));
            }
        }
        if let Some(valout) = info.valout {
            result = Self::push_tagarg(result, valout, None); //Leave the arg open
        }
        else {
            result.push_str(">");
        }
        result
    }

    fn push_close_tag(mut result: String, info: &TagInfo) -> String {
        if let TagType::SelfClosing(_) = info.tag_type {
            //Need to finish closing the attribute, self closing tags have
            //no... well, closing tag
            result.push_str(r#"">"#);
        }
        else {
            result.push_str("</");
            result.push_str(info.outtag);
            result.push_str(">");
        }
        result
    }

    pub fn parse(&self, input: &str) -> String 
    {
        //We know it will be at LEAST as big, and that strings usually double in size
        //when they grow anyway, so just start at 2X by default
        let mut result = String::with_capacity(input.len() * 2);

        //Because of utf-8, it's better to just use regex directly all the time?
        let mut slice = &input[0..]; //Not necessary to be this explicit ofc

        //Only 'Taginfo' can create scope, so don't worry about "DirectReplace" types
        let mut scopes = BBScope::new();

        //While there is string left, keep checking against all the regex. Remove some regex
        //if the current scope is a meanie
        while slice.len() > 0
        {
            //Slow? IDK
            let verbatim_scope = scopes.scopes.last().and_then(|i| Some(TagType::is_verbatim(&i.tag_type))).unwrap_or(false);

            let mut matched_do : Option<&TagDo> = None;

            for tagdo in &self.tags {
                if verbatim_scope && !matches!(tagdo.match_type, MatchType::DirectReplace(_)) {
                    continue;
                }
                if tagdo.regex.is_match(slice) {
                    matched_do = Some(tagdo);
                    break;
                }
            }

            if let Some(tagdo) = matched_do {
                //There should only be one but whatever
                for capture in tagdo.regex.captures_iter(slice) {
                    match &tagdo.match_type {
                        MatchType::DirectReplace(new_text) => {
                            result.push_str(new_text);
                        },
                        MatchType::Open(info) => {
                            for cscope in scopes.add_scope(info) {
                                result = Self::push_close_tag(result, cscope);
                            }
                            //The add_scope function only gives us the close scopes, so we
                            //still need to emit the open tag
                            result = Self::push_open_tag(result, info, &capture);
                        },
                        MatchType::Close(info) => {
                            for cscope in scopes.close_scope(info) {
                                result = Self::push_close_tag(result, cscope);
                            }
                            //The close_scope function gives us the scopes to close
                        }
                    }
                    slice = &slice[capture[0].len()..];
                }
            }
            else {
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

            //each regex until one matches. skip regexes that aren't direct replacement if 
            //current scope is "verbatim". if direct replace, just dump direct replacement to
            //result. if open tag, dump it to stream too, add to scope. if closing tag, scan backwards
            //until a matching scope is found, pop all scopes and write them in pop order to the result,
            //and finally the last scope too. if none are found, fully ignore close and continue.
        }

        //At the end, we should close any unclosed scopes
        for cscope in scopes.dump_remaining() {
            result = Self::push_close_tag(result, cscope);
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

    #[test]
    fn no_alter() {
        let bbcode = BBCode::build(BBCode::basics()).unwrap();
        assert_eq!(bbcode.parse("hello"), "hello");
    }

    #[test]
    fn lt_single() {
        let bbcode = BBCode::build(BBCode::basics()).unwrap();
        assert_eq!(bbcode.parse("h<ello"), "h&lt;ello");
    }

    #[test]
    fn gt_single() {
        let bbcode = BBCode::build(BBCode::basics()).unwrap();
        assert_eq!(bbcode.parse("h>ello"), "h&gt;ello");
    }

    #[test]
    fn amp_single() {
        let bbcode = BBCode::build(BBCode::basics()).unwrap();
        assert_eq!(bbcode.parse("h&ello"), "h&amp;ello");
    }

    #[test]
    fn quote_single() {
        let bbcode = BBCode::build(BBCode::basics()).unwrap();
        assert_eq!(bbcode.parse("h'ello"), "h&#39;ello");
    }

    #[test]
    fn doublequote_single() {
        let bbcode = BBCode::build(BBCode::basics()).unwrap();
        assert_eq!(bbcode.parse("h\"ello"), "h&quot;ello");
    }

    #[test]
    fn complex_escape() {
        let bbcode = BBCode::build(BBCode::basics()).unwrap();
        assert_eq!(bbcode.parse(
            //this has both 4 byte single scalar unicde and a big 25 byte combo one
            "it's a %CRAZY% <world> 💙=\"yeah\" 👨‍👨‍👧‍👦>>done"), 
            "it&#39;s a %CRAZY% &lt;world&gt; 💙=&quot;yeah&quot; 👨‍👨‍👧‍👦&gt;&gt;done"
        );
    }

    #[test]
    fn simple_bold() {
        let bbcode = BBCode::build(BBCode::basics()).unwrap();
        assert_eq!(bbcode.parse("[b]hello[/b]"), "<b>hello</b>");
    }

    #[test]
    fn simple_nospaces() {
        let bbcode = BBCode::build(BBCode::basics()).unwrap();
        assert_eq!(bbcode.parse("[b ]hello[/ b]"), "[b ]hello[/ b]");
    }

    #[test]
    fn simple_bolditalic() {
        let bbcode = BBCode::build(BBCode::basics()).unwrap();
        assert_eq!(bbcode.parse("[b][i]hello[/i][/b]"), "<b><i>hello</i></b>");
    }
}