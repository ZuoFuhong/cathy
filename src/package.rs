pub type PackageType = u16;

pub struct Package {
    code: u16,        // 消息类型
    content: Vec<u8>, // 消息体
    already: bool,
}

impl Package {
    pub fn new(code: u16, content: Vec<u8>) -> Package {
        Package {
            code,
            content,
            already: true,
        }
    }

    pub fn empty() -> Package {
        Package {
            code: 0,
            content: vec![0; 0],
            already: false,
        }
    }

    pub fn get_code(&self) -> u16 {
        self.code
    }

    pub fn get_content(&self) -> &Vec<u8> {
        &self.content
    }

    pub fn ready(&self) -> bool {
        self.already
    }

    pub fn len(&self) -> u16 {
        self.content.len() as u16
    }
}
