use std::sync::{Mutex, Arc};
use std::collections::HashMap;
use namegen::Name;
use rocket::request::{FromRequest, Outcome};
use rocket::{Request, State};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Store (Arc<Mutex<HashMap<String, NameData>>>);

#[derive(Serialize, Deserialize, Clone)]
pub struct NameData {
    pub id: String,
    pub version: u64,
    pub name: Name,
    pub mtime: u128,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NameHeader {
    pub id: String,
    pub version: u64,
    pub mtime: u128,
}

impl NameData {
    pub fn new(id: &str, version: u64, name: &Name) -> NameData {
        NameData{
            id: id.to_owned(),
            name: name.clone(),
            mtime: SystemTime::now().duration_since(UNIX_EPOCH).expect("time went backwards?").as_millis(),

            version,
        }
    }
}

impl Store {
    pub fn list(&self) -> Vec<NameHeader> {
        if let Ok(map) = self.0.lock() {
            map.values().map(|v| NameHeader{
                id: v.id.clone(),
                version: v.version,
                mtime: v.mtime
            }).collect()
        } else {
            Vec::new()
        }
    }

    pub fn find(&self, id: &str) -> Option<NameData> {
        if let Ok(map) = self.0.lock() {
            map.get(&id.to_owned()).map(|n| (*n).clone())
        } else {
            None
        }
    }

    pub fn save(&self, id: &str, mut name_data: NameData) -> bool {
        if let Ok(mut map) = self.0.lock() {
            name_data.mtime = SystemTime::now().duration_since(UNIX_EPOCH).expect("time went backwards?").as_millis();
            map.insert(id.to_owned(), name_data).is_some()
        } else {
            false
        }
    }

    pub fn delete(&self, id: &str) -> Option<NameData> {
        if let Ok(mut map) = self.0.lock() {
            map.remove(&id.to_owned())
        } else {
            None
        }
    }

    pub fn new() -> Store {
        Store(Arc::new(Mutex::new(HashMap::with_capacity(64))))
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Store {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> Outcome<Store, ()> {
        let map: State<Arc<Mutex<HashMap<String, NameData>>>> = request.guard()?;
        Outcome::Success(Store(map.clone()))
    }
}
