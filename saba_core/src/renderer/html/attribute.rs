use alloc::string::String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute{
    name: String,
    value: String,
}

impl Attribute {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            value: String::new(),
        }
    }

    // Attributeのフィールドの値を追加するためのメソッド
    pub fn add_char(&mut self, c: char, is_name: bool) {
        if is_name {
            self.name.push(c);
        } else {
            self.value.push(c);
        }
    }

    //ゲッタ
    pub fn name(&self) -> String {
        return self.name.clone();
    }
    pub fn value(&self) -> String {
        return self.value.clone();
    }
}
