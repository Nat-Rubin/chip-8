pub(crate) struct Stack {
    addresses: Vec<u16>,
}

impl Stack {
    pub fn new() -> Self {
        Stack {
            addresses: Vec::new(),
        }
    }

    pub fn push(&mut self, addr: u16) {
        self.addresses.push(addr);
    }

    pub fn pop(&mut self,) -> u16 {
        self.addresses.pop().unwrap()
    }
}
