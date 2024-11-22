use alloc::string::String;
use alloc::vec::Vec;
use crate::renderer::html::attribute::{self, Attribute};

#[derive(Debug, Clone, PartialEq, Eq)]
// トークンの列挙型
pub enum HtmlToken {
    // 開始タグ
    StartTag {
        tag: String,
        self_closing: bool,
        attributes: Vec<Attribute>,
    },
    // 終了タグ
    EndTag {
        tag: String,
    },
    // 文字
    Char(char),
    //ファイルの終了
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
// Tokenizerがとる状態の列挙型
pub enum State {
    Data,
    TagOpen,
    EndTagOpen,
    TagName,
    BeforeAttributeName,
    AttributeName,
    AfterAttributeName,
    BeforeAttributeValue,
    AttributeValueDoubleQuoted,
    AttributeValueSingleQuoted,
    AttributeValueUnquoted,
    AfterAttributeValueQuoted,
    SelfClosingStartTag,
    ScriptData,
    ScriptDataLessThanSign,
    ScriptDataEndTagOpen,
    ScriptDataEndTagName,
    TemporaryBuffer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HtmlTokenizer {
    state: State,
    pos: usize,
    reconsume: bool,
    latest_token: Option<HtmlToken>,
    input: Vec<char>,
    buf: String,
}

impl HtmlTokenizer {
    pub fn new(html: String) -> Self {
        Self {
            state: State::Data,
            pos: 0,
            reconsume: false,
            latest_token: None,
            input: html.chars().collect(),
            buf: String::new(),
        }
    }
    // Eof判定
    fn is_eof(&self) -> bool {
        return self.pos > self.input.len();
    }
    // 現posの文字を読み取り、posを1進める
    fn consume_next_input(&mut self) -> char {
        let c = self.input[self.pos];
        self.pos += 1;
        return c;
    }
    // 現posの1つ前の文字を読み取る
    fn reconsume_input(&mut self) -> char {
        self.reconsume = false;
        return self.input[self.pos - 1];
    }
    // StartTagもしくはEndTagトークンを作成し、latest_tokenにセットする
    fn create_tag(&mut self, start_tag_token: bool) {
        if start_tag_token {
            self.latest_token = Some(HtmlToken::StartTag { 
                tag: String::new(),
                self_closing: false,
                attributes:Vec::new()
            });
        } else {
            self.latest_token = Some(HtmlToken::EndTag { 
                tag: String::new()
            });
        }
    }
    // latest_tokenの末尾に文字を追加する
    fn append_tag_name(&mut self, c: char) {
        assert!(self.latest_token.is_some());
        if let Some(t) = self.latest_token.as_mut() {
            match t {
                HtmlToken::StartTag { tag: ref mut tag, self_closing: _, attributes: _,}
                | HtmlToken::EndTag { tag: ref mut tag } 
                    => tag.push(c),
                _ => panic!("`latest_token` should be either StartTag or EndTag"),
            }
        }
    }
    // 最新のlatest_tokenを返し、リセットする
    fn take_latest_token(&mut self) -> Option<HtmlToken> {
        assert!(self.latest_token.is_some());

        let t = self.latest_token.as_ref().cloned();
        self.latest_token = None;
        assert!(self.latest_token.is_none());
    }
    // latest_tokenにAttributeを追加する
    fn start_new_attribute(&mut self) {
        assert!(self.latest_token.is_some());

        if let Some(t) = self.latest_token.as_mut() {
            match t {
                HtmlToken::StartTag { tag: _, self_closing: _, attributes: ref mut attributes} 
                    => {
                        attributes.push(Attribute::new());
                    }
                _ => panic!("`latest_token` should be either StartTag"),
            }
        }
    }
    // latest_tokenに属性文字を追加する
    fn append_attribute(&mut self, c:char, is_name:bool) {
        assert!(self.latest_token.is_some());

        if let Some(t) = self.latest_token.as_mut() {
            match t {
                HtmlToken::StartTag { tag: _, self_closing: _, attributes: ref mut attributes }
                    => {
                        let len = attributes.len();
                        assert!(len > 0);
                        attributes[len-1].add_char(c, is_name);
                    }
                _ => panic!("`latest_token` should be either StartTag"),
            }
        }
    }
    // latest_tokenが開始タグの場合、self_closingフラグをtrueにする
    fn set_self_closing_flag(&mut self) {
        assert!(self.latest_token.is_some());

        if let Some(t) = self.latest_token.as_mut() {
            match t {
                HtmlToken::StartTag { tag: _, self_closing: ref mut self_closing, attributes: _ }
                    => *self_closing = true,
                _ => panic!("`latest_token` should be either StartTag"),
            }
        }
    }
}

impl Iterator for HtmlTokenizer {
    type Item = HtmlToken;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.input.len() {
            return None;
        }

        loop {
            let c = match self.reconsume {
                true => self.reconsume_input(),
                false => self.consume_next_input(),
            };

            match self.state {
                State::Data => {
                    // < 記号の場合、TagOpen状態に遷移
                    if c == '<' {
                        self.state = State::TagOpen;
                        continue;
                    }
                    // 最後の文字の場合、Eofトークンを返す
                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }
                    // それ以外の場合、文字トークンを返す 
                    return Some(HtmlToken::Char(c));
                }
                State::TagOpen => {
                    // / 記号の場合、EndTagOpen状態に遷移
                    if c == '/' {
                        self.state = State::EndTagOpen;
                        continue;
                    }
                    // 文字がアルファベットで場合、TagName状態に遷移
                    if self.is_ascii_alphabetic() {
                        self.reconsume = true;
                        self.state = State::TagName;
                        self.create_tag(true);
                        continue;
                    }
                    // 最後の文字の場合、Eofトークンを返す
                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }
                    // 上記以外の場合、Data状態に遷移しもう一度判定する
                    self.reconsume = true;
                    self.state = state::Data;
                }
                State::EndTagOpen => {
                    // 最後の文字の場合、Eofトークンを返す
                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }
                    // 文字がアルファベットの場合、TagName状態に遷移
                    if c.is_ascii_alphabetic() {
                        self.reconsume = true;
                        self.state = State::TagName;
                        self.create_tag(false);
                        continue;
                    }
                    // 上記以外の場合、Data状態に遷移しもう一度判定する
                    self.reconsume = true;
                    self.state = State::Data;
                }
                State::TagName => {
                    // スペースの場合、BeforeAttributeName状態に遷移
                    if c == ' ' {
                        self.state = State::BeforeAttributeName;
                        continue;
                    }
                    // / 記号の場合、SelfClosingStartTag状態に遷移
                    if c == '/' {
                        self.state = State::SelfClosingStartTag;
                        continue;
                    }
                    // > 記号の場合、Data状態に移行しlatest_tokenを返す
                    if c == '>' {
                        self.state = State::Data;
                        return self.take_latest_token();
                    }
                    // 大文字の場合、小文字に変換し現在のタグに追加する
                    if c.is_ascii_uppercase() {
                        self.append_tag_name(c.to_ascii_lowercase());
                        continue;
                    }
                    // 最後の文字の場合、Eofトークンを返す
                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }
                    // それ以外の場合、現在のタグに追加する
                    self.append_tag_name(c);
                }
                State::BeforeAttributeName => {
                    // AfterAttributeName状態に遷移
                    if c == '/' || c == '>' || self.is_eof() {
                        self.reconsume = true;
                        self.state = State::AfterAttributeName;
                        continue;;
                    }
                    // それ以外の場合、AfterAttributeName状態に遷移し、start_new_attributeメソッドを呼びだす
                    self.reconsume = true;
                    self.state = State::AttributeName;
                    self.start_new_attribute();
                }
                State::AttributeName => {
                    // AfterAttributeName状態に遷移
                    if c == ' ' || c == '/' || c == '>' || self.is_eof() {
                        self.reconsume = true;
                        self.state = State::AfterAttributeName;
                        continue;
                    }
                    // = 記号の場合、BeforeAttributeValueに遷移
                    if c == '=' {
                        self.state = State::BeforeAttributeValue;
                        continue;
                    }
                    // 大文字の場合、小文字に変換してlatest_tokenに属性文字を追加する
                    if c.is_ascii_uppercase() {
                        self.append_attribute(c.to_ascii_lowercase(), true);
                        continue;
                    }
                    // それ以外の場合、latest_tokenに属性文字を追加する
                    self.append_attribute(c, true);
                }
                State::AfterAttributeName => {
                    // 空文字は無視する
                    if c == ' ' {
                        continue;
                    }
                    // SelfClosingStartTag状態に遷移する
                    if c == '/' {
                        self.state = State::SelfClosingStartTag;
                        continue;
                    }
                    // BeforeAttributeValue状態に遷移
                    if c == '=' {
                        self.state = State::BeforeAttributeValue;
                        continue;
                    }
                    // Data状態に遷移
                    if c == '>' {
                        self.state = State::Data;
                        return self.take_latest_token();
                    }
                    // 最後の文字の場合、Eofトークンを返す
                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }
                    // それ以外の場合、AttributeName状態に遷移
                    self.reconsume = true;
                    self.state = State::AttributeName;
                    self.start_new_attribute();
                }
                State::BeforeAttributeValue => {
                    // 空白は無視
                    if c ==' ' {
                        continue;
                    }
                    // " 記号の場合、AttributeValueDoubleQuoted状態に遷移
                    if c == '"' {
                        self.state = State::AttributeValueDoubleQuoted;
                        continue;
                    }
                    // ' 記号の場合、AttributeValueSingleQuoted状態に遷移
                    if c == '\'' {
                        self.state = State::AttributeValueSingleQuoted;
                        continue;
                    }
                    // それ以外の場合、AttributeValueUnquoted状態に遷移
                    self.reconsume = true;
                    self.state = State::AttributeValueUnquoted;
                }
                State::AttributeValueDoubleQuoted => {
                    // " 記号の場合、AfterAttributeValueQuoted状態に遷移
                    if c == '"' {
                        self.state = State::AfterAttributeValueQuoted;
                        continue;
                    }
                    // 最後の文字の場合、Eofトークンを返す
                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }
                    // それ以外の場合、Attributesに文字を追加する
                    self.append_attribute(c, false);
                }
                State::AttributeValueSingleQuoted => {
                    // ' 記号の場合、AfterAttributeValueQuoted状態に遷移
                    if c == '\'' {
                        self.state = State::AfterAttributeValueQuoted;
                        continue;
                    }
                    // 最後の文字の場合、Eofトークンを返す
                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }
                    // それ以外の場合、Attributeに文字を追加する
                    self.append_attribute(c, false);
                }
                State::AttributeValueUnquoted => {
                    if c == ' ' {
                        self.state = State::BeforeAttributeName;
                        continue;
                    }
                    if c == '>' {
                        self.state = State::Data;
                        continue;
                    }
                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }
                    // それ以外の場合、Attributeに文字を追加する
                    self.append_attribute(c, false);
                }
                State::AfterAttributeValueQuoted => {
                    if c == ' ' {
                        self.state = State::BeforeAttributeName;
                        continue;
                    }
                    if c == '/' {
                        self.state = State::SelfClosingStartTag;
                        continue;
                    }
                    if c == '>' {
                        self.state = State::Data;
                        continue;
                    }
                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }
                    self.reconsume = true;
                    self.state = State::BeforeAttributeValue;
                }
                State::SelfClosingStartTag => {
                    if c == '>' {
                        self.set_self_closing_flag();
                        self.state = State::Data;
                        return self.take_latest_token();
                    }
                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }
                }
                State::ScriptData => {
                    if c == '<' {
                        self.state = State::ScriptDataLessThanSign;
                        continue;
                    }
                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }
                    // それ以外の場合
                    return Some(HtmlToken::Char(c));
                }
                State::ScriptDataLessThanSign => {
                    // 一時バッファをリセットする
                    if c == '/' {
                        self.buf = String::new();
                        self.state = State::ScriptDataEndTagOpen;
                        continue;
                    }
                    // それ以外の場合
                    self.reconsume = true;
                    self.state = State::ScriptData;
                    return Some(HtmlToken::Char('<'));
                }
                State::ScriptDataEndTagOpen => {
                    if c.is_ascii_alphabetic() {
                        self.reconsume = true;
                        self.state = State::ScriptDataEndTagName;
                        self.create_tag(false);
                        continue;
                    }
                    // それ以外の場合
                    self.reconsume = true;
                    self.state = State::ScriptData;
                    return Some(HtmlToken::Char('<'));  // 使用では < と / の2つのトークンを返すようになっているが、1トークンしか返せないため < のみを返す
                }
                State::ScriptDataEndTagName => {
                    if c == '>' {
                        self.state = State::Data;
                        return self.take_latest_token();
                    }
                    if c.is_ascii_alphabetic() {
                        self.buf.push(c);
                        self.append_tag_name(c.to_ascii_lowercase());
                        continue;
                    }
                    self.state = State::TemporaryBuffer;
                    self.buf = String::from("</") + &self.buf;
                    self.buf.push(c);
                    continue;
                }
                State::TemporaryBuffer => {
                    self.reconsume = true;
                    if self.buf.chars().count() == 0 {
                        self.state = State::ScriptData;
                        continue;
                    }
                    let c = self.buf.chars().nth(0).expect("self.buf should have at least 1 char");
                    self.buf.remove(0);
                    return Some(HtmlToken::Char(c));
                }
            }
        }
    }
}
