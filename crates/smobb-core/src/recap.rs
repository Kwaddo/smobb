pub trait Storage {
    fn read(&self) -> Vec<String>;
    fn write(&mut self, lines: &[String]);
}

#[derive(Default)]
pub struct MemoryStorage {
    pub lines: Vec<String>,
}

impl Storage for MemoryStorage {
    fn read(&self) -> Vec<String> {
        self.lines.clone()
    }
    fn write(&mut self, lines: &[String]) {
        self.lines = lines.to_vec();
    }
}

#[derive(Default)]
pub struct Recap {
    entries: Vec<String>,
}

impl Recap {
    pub fn add(&mut self, line: &str) -> bool {
        if self.entries.iter().any(|e| e == line) {
            return false;
        }
        self.entries.push(line.to_string());
        true
    }

    pub fn entries(&self) -> &[String] {
        &self.entries
    }

    pub fn load(storage: &dyn Storage) -> Recap {
        let mut r = Recap::default();
        for l in storage.read() {
            r.add(&l);
        }
        r
    }

    pub fn persist(&self, storage: &mut dyn Storage) {
        storage.write(&self.entries);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedups() {
        let mut r = Recap::default();
        assert!(r.add("a"));
        assert!(r.add("b"));
        assert!(!r.add("a")); // duplicate
        assert_eq!(r.entries(), &["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn round_trips_through_storage() {
        let mut s = MemoryStorage::default();
        let mut r = Recap::default();
        r.add("x");
        r.add("y");
        r.persist(&mut s);
        let r2 = Recap::load(&s);
        assert_eq!(r2.entries(), &["x".to_string(), "y".to_string()]);
    }
}
