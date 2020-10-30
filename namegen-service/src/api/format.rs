use rocket_contrib::json::Json;
use rocket::State;
use rocket::response::status::{Custom, NoContent, NotFound};
use namegen::{Name, NameFormat};

use crate::store::{Store, NameData, NameHeader};
use rocket::http::Status;

#[get("/<id>/formats")]
pub fn list_formats(store: State<Store>, id: String) -> Result<Json<Vec<NameFormat>>, NotFound<String>> {
    match store.find(&id) {
        Some(name) => Ok(Json(name.name.formats().cloned().collect())),
        None => Err(NotFound("Name not found".to_owned()))
    }
}

#[post("/<id>/formats", format = "json", data = "<input>")]
pub fn add_format(store: State<Store>, id: String, input: Json<FormatCreateInput>) -> Result<Json<NameFormat>, Custom<String>> {
    match store.find(&id) {
        Some(mut name_data) => {
            if name_data.name.formats().find(|f| f.name() == input.name).is_some() {
                return Err(Custom(Status::Conflict, "Format already exists".to_owned()))
            }

            name_data.name.add_format(&input.name, &input.format);
            name_data.version += 1;
            store.save(&id, name_data.clone());

            Ok(Json(name_data.name.formats().find(|f| f.name() == input.name).cloned().unwrap()))
        },
        None => Err(Custom(Status::NotFound, "Name not found".to_owned()))
    }
}

#[derive(Serialize, Deserialize)]
pub struct FormatCreateInput {
    pub name: String,
    pub format: String,
}