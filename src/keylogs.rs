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
        self.refresh();

        if chrono::offset::Utc::now().naive_utc().timestamp_millis()
            - self
                .keylogs
                .last()
                .unwrap_or(&KeyLog {
                    key: "".to_string(),
                    input_datetime: chrono::NaiveDateTime::from_timestamp(0, 0),
                })
                .input_datetime
                .timestamp_millis()
            <= 500
        {
            *self.keylogs.last_mut().unwrap() = KeyLog {
                key: format!("{}{}", self.keylogs.last().unwrap().key, key.into()),
                input_datetime: chrono::offset::Utc::now().naive_utc(),
            };
        } else {
            self.keylogs.push(KeyLog {
                key: key.into(),
                input_datetime: chrono::offset::Utc::now().naive_utc(),
            })
        }
    }

    pub fn refresh(&mut self) {
        self.keylogs.retain(|x| {
            chrono::offset::Utc::now().naive_utc().timestamp() - x.input_datetime.timestamp() <= 2
        });
        println!("{:?}", self.keylogs);
    }

    pub fn get_keys(&mut self) -> Vec<String> {
        self.refresh();
        self.keylogs.iter().map(|k| k.key.clone()).collect()
    }

    pub fn get_keys_from_last(&mut self, num: usize) -> Vec<String> {
        self.refresh();
        let len = self.keylogs.len();
        let start_index = if len > num { len - num } else { 0 };

        self.keylogs[start_index..len]
            .iter()
            .map(|k| k.key.clone())
            .collect()
    }
}
