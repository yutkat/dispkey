use chrono::naive::NaiveDateTime;

#[derive(Debug)]
pub struct KeyLogs {
    keylogs: Vec<KeyLog>,
}

#[derive(Debug)]
struct KeyLog {
    key: String,
    input_datetime: NaiveDateTime,
}

impl KeyLogs {
    pub fn new() -> Self {
        KeyLogs {
            keylogs: Vec::<KeyLog>::new(),
        }
    }

    pub fn push<S: Into<String>>(&mut self, key: S) {
        self.keylogs.retain(|x| {
            chrono::offset::Utc::now().naive_utc().timestamp() - x.input_datetime.timestamp() <= 1
        });
        println!("{:?}", self.keylogs);

        if self.keylogs.len() >= 4 {
            self.keylogs.remove(0);
        }
        self.keylogs.push(KeyLog {
            key: key.into(),
            input_datetime: chrono::offset::Utc::now().naive_utc(),
        })
    }

    pub fn refresh(&mut self) {
        self.keylogs.retain(|x| {
            chrono::offset::Utc::now().naive_utc().timestamp() - x.input_datetime.timestamp() <= 1
        });
        println!("{:?}", self.keylogs);
    }

    pub fn get_keys(&self) -> Vec<String> {
        self.keylogs.iter().map(|k| k.key.clone()).collect()
    }
}
