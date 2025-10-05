use std::collections::HashMap;

#[derive(Clone)]
pub struct IOMap {
    io: HashMap<(usize, usize), (usize, usize)>,
}

impl IOMap {
    pub fn new(conns: &Vec<crate::graph::Connection>) -> Self {
        let mut io = HashMap::new();
        for conn in conns {
            io.insert(conn.to, conn.from);
        }
        Self { io }
    }

    pub fn get(&self, from: (usize, usize)) -> Option<&(usize, usize)> {
        self.io.get(&from)
    }
}
